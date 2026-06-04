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

pub async fn create_session(cache: Cache, creds: LSpotCreds) -> Result<Session> {
    // connect to Spotify session
    tracing::info!("Connecting librespot session...");
    let sess = Session::new(Default::default(), Some(cache.clone()));
    sess.connect(creds.clone(), true).await?;
    tracing::info!("Successfully connected librespot session.");
    Ok(sess)
}

pub async fn authenticate() -> Result<(Cache, LSpotCreds, Session)> {
    // credentials cache
    let cache = librespot::core::cache::Cache::new(Some("."), None, None, None)?;

    // obtain credentials
    let creds = if let Some(creds) = cache.credentials() {
        creds
    } else {
        get_creds().await?
    };

    let sess = create_session(cache.clone(), creds.clone()).await?;

    Ok((cache, creds, sess))
}
