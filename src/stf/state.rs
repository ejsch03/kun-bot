use rspotify::model::TrackId;
use songbird::input::{Parsed, core::probe::Hint};
use symphonia::core::{codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream};

use super::prelude::*;

struct LibreSpotify {
    cred: Credentials,
    sess: AsyncMutex<Session>,
}

impl LibreSpotify {
    pub async fn session(&self) -> Result<Session> {
        let sess = self.sess.lock().await;
        if sess.is_invalid() {
            Self::refresh_session(&sess).await?;
        }
        Ok(sess.clone())
    }

    async fn refresh_session(sess: &AsyncMutexGuard<'_, Session>) -> Result<()> {
        let creds = Cache::new(Some("."), None, None, None)?
            .credentials()
            .ok_or(anyhow!("No cached credentials"))?;
        sess.connect(creds, true).await?;
        Ok(())
    }
}

pub struct Spotify {
    rspot: RSpotify,                                 // spotify dev api
    lspot: LibreSpotify,                             // librespot config
    http: HttpClient,                                // reqwests client
    song_cache: AsyncMutex<HashMap<String, Song>>,   // song metadata cache
    cover_cache: AsyncMutex<HashMap<String, Bytes>>, // cover-art metadata cache
}

impl Spotify {
    pub async fn new(cred: Credentials) -> Result<Self> {
        let rspot_cred = rspotify::Credentials::new(cred.client_id(), cred.client_secret());
        let rspot = RSpotify::new(rspot_cred);
        rspot.request_token().await?;

        let sess = AsyncMutex::new(super::auth::create_session().await?);

        let lspot = LibreSpotify { cred, sess };

        let app_state = Self {
            rspot,
            lspot,
            http: Default::default(),
            song_cache: Default::default(),
            cover_cache: Default::default(),
        };
        Ok(app_state)
    }

    pub async fn get_cover_art(&self, id: String) -> Result<Bytes> {
        let image_bytes = if let Some(bytes) = self.cover_cache.lock().await.get(&id) {
            bytes.clone()
        } else {
            let album_id = AlbumId::from_id(&id)?;

            let album = self.rspot.album(album_id, None).await?;

            // spotify returns images sorted largest first
            let image_url = album
                .images
                .first()
                .ok_or_else(|| anyhow!("no available album cover."))?;

            let bytes = self.http.get(&image_url.url).send().await?.bytes().await?;

            self.cover_cache
                .lock()
                .await
                .insert(id.clone(), bytes.clone());

            bytes
        };
        Ok(image_bytes)
    }

    async fn parse_track(&self, track: &FullTrack) -> Result<Song> {
        let song = Song::from_spotify(track)?;
        self.song_cache
            .lock()
            .await
            .insert(song.id.clone(), song.clone());
        Ok(song)
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
        let buf: Arc<Mutex<VecDeque<u8>>> = Default::default();
        let (tx, rx) = waitx::pair();
        let sink = StreamingSink::new(AudioFormat::F32, buf.clone(), tx);
        let player = Player::new(Default::default(), sess, Box::new(NoOpVolume), {
            let sink = sink.clone();
            move || Box::new(sink)
        });
        let header = write_wav_header(2, 44100, 32);
        buf.lock().extend(header);

        player.load(uri, true, 0);

        let pcm_stream = PcmStream::new(buf, rx, player);

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
}
