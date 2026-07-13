use super::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum SpotifyUriType {
    Track,
    Playlist,
}

#[derive(Debug)]
pub enum FullSearchResult {
    Track(Box<Song>),
    Playlist { title: String, songs: Vec<Song> },
}

#[derive(Clone)]
pub struct LibreSpotify {
    cache: Cache,
    creds: LibreCreds,
    sess: Arc<Mutex<Session>>,
}

impl LibreSpotify {
    pub async fn new() -> Result<Self> {
        let (cache, creds, sess) = authenticate().await?;

        Ok(Self {
            cache,
            creds,
            sess: Arc::new(Mutex::new(sess)),
        })
    }

    pub async fn session(&self) -> Result<Session> {
        let mut sess = self.sess.lock().await;
        if sess.is_invalid() {
            *sess = create_session(self.cache.clone(), self.creds.clone()).await?;
        }
        Ok(sess.clone())
    }

    pub async fn source(&self, uri: SpotifyUri) -> Result<AudioStream<Box<dyn MediaSource>>> {
        let sess = self.session().await?;

        let rb = HeapRb::<u8>::new(BUFFER_CAPACITY);
        let (mut prod, cons) = rb.split();

        // write WAV header for F32 stereo at 44100hz
        let header = write_wav_header(2, 44100, 32);
        prod.push_slice(&header);

        let (data_tx, data_rx) = waitx::pair();
        let (space_tx, space_rx) = waitx::pair();
        let status = Arc::new(StreamStatus::default());

        let sink = StreamingSink::new(
            AudioFormat::F32,
            prod,
            data_tx.clone(),
            space_rx,
            status.clone(),
        );
        let player = Player::new(Default::default(), sess, Box::new(NoOpVolume), move || {
            Box::new(sink)
        });

        // watch for the track finishing (or failing): librespot never calls
        // `Sink::stop` at a natural end-of-track, so without this the reader
        // would block forever and the songbird queue would never advance
        let mut events = player.get_player_event_channel();
        {
            let status = status.clone();
            tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        PlayerEvent::EndOfTrack { .. }
                        | PlayerEvent::Stopped { .. }
                        | PlayerEvent::Unavailable { .. }
                        | PlayerEvent::SessionDisconnected { .. } => break,
                        _ => {}
                    }
                }
                // also reached if the channel closes (player dropped)
                status.finish();
                data_tx.signal();
            });
        }

        player.load(uri, true, 0);

        let pcm_stream = PcmStream::new(cons, space_tx, data_rx, status, player);

        Ok(AudioStream {
            input: Box::new(pcm_stream),
        })
    }
}

pub struct Spotify {
    rspot: RSpotify,                                       // spotify dev api
    lspot: LibreSpotify,                                   // librespot config
    song_cache: Mutex<HashMap<String, Song>>,              // song metadata cache
    cover_cache: Mutex<HashMap<AlbumId<'static>, String>>, // cover-art metadata cache
}

impl Spotify {
    pub async fn new(cred: Credentials) -> Result<Self> {
        let rspot_cred = rspotify::Credentials::new(cred.client_id(), cred.client_secret());
        let rspot = RSpotify::new(rspot_cred);
        rspot.request_token().await?;

        let app_state = Self {
            rspot,
            lspot: LibreSpotify::new().await?,
            song_cache: Default::default(),
            cover_cache: Default::default(),
        };
        Ok(app_state)
    }

    pub fn source(&self, uri: SpotifyUri) -> Source {
        Source::new(self.lspot.clone(), uri)
    }

    pub async fn get_cover_url(&self, id: AlbumId<'static>) -> Result<String> {
        let url = if let Some(url) = self.cover_cache.lock().await.get(&id) {
            url.clone()
        } else {
            let album = self.rspot.album(id.clone(), None).await?;

            // spotify returns images sorted largest first
            let url = album
                .images
                .first()
                .ok_or_else(|| anyhow!("no available album cover."))?
                .url
                .clone();

            let mut cache = self.cover_cache.lock().await;
            cache.insert(id.clone(), url.clone());

            url
        };
        Ok(url)
    }

    pub async fn search(&self, query: &str) -> Result<FullSearchResult> {
        // url
        if let Some((.., query)) = query.split_once(SPOTIFY_URL) {
            let (ty_str, path) = query
                .split_once('/')
                .ok_or_else(|| anyhow!("invalid spotify url"))?;

            let ty = match ty_str {
                "track" => SpotifyUriType::Track,
                "playlist" => SpotifyUriType::Playlist,
                _ => anyhow::bail!("unsupported spotify uri type"),
            };

            let s = if let Some((uri_s, ..)) = path.split_once("?") {
                uri_s
            } else {
                path
            };

            let id = SpotifyId::from_base62(s)?;

            return match ty {
                SpotifyUriType::Track => {
                    let uri = SpotifyUri::Track { id }.to_string();
                    let track_id = TrackId::from_uri(uri.as_str())?;
                    let track = self.rspot.track(track_id, None).await?;
                    let song = self.parse_track(&track).await?;
                    Ok(FullSearchResult::Track(Box::new(song)))
                }
                SpotifyUriType::Playlist => {
                    let uri = SpotifyUri::Playlist { user: None, id }.to_string();
                    let playlist_id = PlaylistId::from_uri(uri.as_str())?;
                    let playlist = self.rspot.playlist(playlist_id, None, None).await?;
                    let tracks = playlist
                        .items
                        .items
                        .into_iter()
                        .filter_map(|i| {
                            if let Some(PlayableItem::Track(t)) = i.item {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<FullTrack>>();

                    // parse every track
                    let results =
                        futures::future::join_all(tracks.iter().map(|t| self.parse_track(t))).await;
                    let songs = results.into_iter().filter_map(|r| r.ok()).collect();

                    Ok(FullSearchResult::Playlist {
                        title: playlist.name,
                        songs,
                    })
                }
            };
        }

        // search term
        let results = self
            .rspot
            .search(query, SearchType::Track, None, None, Some(1), None)
            .await?;

        if let SearchResult::Tracks(tracks) = results {
            let track = tracks
                .items
                .first()
                .ok_or_else(|| anyhow!("failed to find song."))?;
            let song = self.parse_track(track).await?;
            Ok(FullSearchResult::Track(Box::new(song)))
        } else {
            bail!("no available song(s)")
        }
    }

    async fn parse_track(&self, track: &FullTrack) -> Result<Song> {
        let cover_url = if let Some(id) = track.album.id.as_ref() {
            self.get_cover_url(id.clone()).await.ok()
        } else {
            None
        };
        let song = Song::from_spotify(track, cover_url).await?;
        self.song_cache
            .lock()
            .await
            .insert(song.id.clone(), song.clone());
        Ok(song)
    }
}
