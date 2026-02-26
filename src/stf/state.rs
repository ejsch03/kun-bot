use super::prelude::*;

struct LibreSpotify {
    sess: Mutex<Session>,
}

impl LibreSpotify {
    pub async fn session(&self) -> Result<Session> {
        let mut sess = self.sess.lock().await;
        if sess.is_invalid() {
            *sess = Self::create_session().await?;
        }
        Ok(sess.clone())
    }

    async fn create_session() -> Result<Session> {
        let creds = Cache::new(Some("."), None, None, None)?
            .credentials()
            .ok_or(anyhow!("No cached credentials"))?;

        let session_config = SessionConfig::default();
        let session = Session::new(session_config, None);
        session.connect(creds, true).await?;
        Ok(session)
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

        let sess = Mutex::new(super::auth::create_session().await?);

        let lspot = LibreSpotify { sess };

        let app_state = Self {
            rspot,
            lspot,
            song_cache: Default::default(),
            cover_cache: Default::default(),
        };
        Ok(app_state)
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

    pub async fn search(&self, query: &str) -> Result<Song> {
        // url
        if let Some((.., path)) = query.split_once(SPOTIFY_TRACK_URL) {
            let s = if let Some((uri_s, ..)) = path.split_once("?") {
                uri_s
            } else {
                path
            };
            let uri = SpotifyId::from_base62(s).map(|id| SpotifyUri::Track { id }.to_string())?;
            let id = TrackId::from_uri(&uri)?;
            let track = self.rspot.track(id, None).await?;
            return self.parse_track(&track).await;
        }

        // id
        if let Ok(uri) =
            SpotifyId::from_base62(query).map(|id| SpotifyUri::Track { id }.to_string())
        {
            let id = TrackId::from_uri(&uri)?;
            let track = self.rspot.track(id, None).await?;
            return self.parse_track(&track).await;
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
            self.parse_track(track).await
        } else {
            bail!("no available song(s)")
        }
    }

    pub async fn stream(&self, uri: SpotifyUri) -> Result<Input> {
        let sess = self.lspot.session().await?;

        let rb = HeapRb::<u8>::new(BUFFER_CAPACITY);
        let (mut prod, cons) = rb.split();
        let header = write_wav_header(2, 44100, 32);
        prod.push_slice(&header);

        let (tx, rx) = waitx::pair();
        let sink = StreamingSink::new(AudioFormat::F32, prod, tx);
        let player = Player::new(Default::default(), sess, Box::new(NoOpVolume), {
            move || Box::new(sink)
        });

        player.load(uri, true, 0);

        let pcm_stream = PcmStream::new(cons, rx, player);

        // build the MSS from your PcmStream
        let mss = MediaSourceStream::new(Box::new(pcm_stream), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("wav");

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss, // only used here
            &FormatOptions::default(),
            &symphonia::core::meta::MetadataOptions::default(),
        )?;

        let format = probed.format;
        let meta = probed.metadata;
        let track = format.default_track().ok_or_else(|| anyhow!("no track"))?;
        let track_id = track.id;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;

        let input = Input::Live(
            LiveInput::Parsed(Parsed {
                format,
                decoder,
                track_id,
                meta,
                supports_backseek: false,
            }),
            None,
        );
        Ok(input)
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
