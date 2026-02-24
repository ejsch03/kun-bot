use super::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub album: String,
    pub track: u32,
    pub duration: u64,
    pub is_dir: bool,
    pub r#type: &'static str,
    pub media_type: &'static str,
    pub suffix: &'static str,
    pub content_type: &'static str,
    pub bit_rate: u32,
    pub bit_depth: u32,
    pub sampling_rate: u32,
    pub channel_count: u32,
    pub transcoded_suffix: &'static str,
    pub transcoded_content_type: &'static str,
    pub disc_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_status: Option<&'static str>,
    #[serde(skip)]
    pub uri: SpotifyUri,
    #[serde(skip)]
    pub url: String,
}

impl Song {
    pub fn from_spotify(t: &FullTrack) -> Result<Self> {
        let track_id =
            t.id.as_ref()
                .ok_or_else(|| anyhow!("track doesn't have an ID."))?;
        let id = track_id.id().to_string();
        let dur = t.duration.to_std()?.as_secs();
        let uri = SpotifyUri::from_uri(&track_id.uri())?;
        let url = track_id.url();

        Ok(Self {
            id,
            title: t.name.clone(),
            album: t.album.name.clone(),
            track: t.track_number,
            duration: dur,
            is_dir: false,
            r#type: "music",
            media_type: "song",
            suffix: "ogg",
            content_type: "audio/ogg",
            bit_rate: 320,
            bit_depth: 16,
            sampling_rate: 44100,
            channel_count: 2,
            transcoded_suffix: "opus",
            transcoded_content_type: "audio/ogg; codecs=opus",
            disc_number: t.disc_number,
            artist: t.artists.first().map(|a| a.name.clone()),
            cover_art: t.album.id.as_ref().map(|id| id.id().to_string()),
            created: t.album.release_date.clone(),
            explicit_status: if t.explicit { Some("explicit") } else { None },
            uri,
            url,
        })
    }
}
