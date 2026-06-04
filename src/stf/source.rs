use super::prelude::*;

pub struct Source {
    inner: super::LibreSpotify,
    uri: SpotifyUri,
}

impl Source {
    pub(super) fn new(inner: super::LibreSpotify, uri: SpotifyUri) -> Self {
        Self { inner, uri }
    }
}

#[async_trait::async_trait]
impl Compose for Source {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        unimplemented!() // sync version, not used
    }

    async fn create_async(
        &mut self,
    ) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        self.inner
            .source(self.uri.clone())
            .await
            .map_err(|e| AudioStreamError::Fail(e.into_boxed_dyn_error()))
    }

    fn should_create_async(&self) -> bool {
        true
    }
}
