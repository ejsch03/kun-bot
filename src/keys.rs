use crate::prelude::*;

pub struct Whitelist {
    data: Vec<GuildId>,
    path: PathBuf,
}

impl Whitelist {
    pub const fn new(data: Vec<GuildId>, path: PathBuf) -> Self {
        Self { data, path }
    }

    pub fn data(&self) -> &[GuildId] {
        self.data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut Vec<GuildId> {
        self.data.as_mut()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

#[derive(Default)]
pub struct MessageLink;

impl TypeMapKey for MessageLink {
    type Value = Arc<Mutex<HashMap<MessageId, MessageId>>>;
}

#[derive(Clone, Debug)]
pub struct TrackInfo {
    inner: Song,
}

impl TrackInfo {
    pub const fn new(song: Song) -> Self {
        Self { inner: song }
    }

    pub fn into_inner(self) -> Song {
        self.inner
    }
}

impl std::ops::Deref for TrackInfo {
    type Target = Song;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
