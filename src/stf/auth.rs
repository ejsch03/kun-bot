use librespot::core::cache::Cache;

use super::prelude::*;

pub async fn get_creds() -> Result<LSpotCreds> {
    let scopes = vec!["streaming"];
    let client =
        librespot::oauth::OAuthClientBuilder::new(SPOTIFY_CLIENT_ID, SPOTIFY_REDIRECT_URI, scopes)
            .open_in_browser()
            .build()
            .map_err(|e| anyhow!("Failed to build OAuth client: {e}"))?;

    let token = client
        .get_access_token_async()
        .await
        .map_err(|e| anyhow!("Failed to get access token: {e}"))?;

    let creds = LSpotCreds::with_access_token(token.access_token.as_str());

    Ok(creds)
}

pub async fn create_session() -> Result<Session> {
    // credentials cache
    let cache = Cache::new(Some("."), None, None, None)?;

    // obtain credentials
    let creds = if let Some(creds) = cache.credentials() {
        creds
    } else {
        get_creds().await?
    };

    // connect to Spotify session
    log::trace!("Connecting...");
    let session = Session::new(Default::default(), Some(cache));
    session.connect(creds, true).await?;

    Ok(session)
}
