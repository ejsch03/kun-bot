use crate::prelude::*;

pub async fn run() -> Result<()> {
    // init crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("failed to install rustls crypto provider"))?;

    // init data
    let data = Data::new().await?;
    let token =
        std::env::var("KUN_BOT_TOKEN").context("KUN_BOT_TOKEN environment variable not set")?;

    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;

    // any shared data fields
    let links = data.links.clone();

    // configure framework
    let framework = Framework::builder()
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(data.prefix.clone()),
                ..Default::default()
            },
            on_error: |e| Box::pin(on_error(e)),
            commands: vec![
                a(),
                w(),
                join(),
                leave(),
                play(),
                playnext(),
                skip(),
                pause(),
                resume(),
                clear(),
                queue(),
                remove(),
                shuffle(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    // instantiate client
    let mut client = songbird::SerenityInit::register_songbird(
        serenity::Client::builder(token, intents)
            .framework(framework)
            .event_handler(Handler),
    )
    .await?;

    client.data.write().await.insert::<MessageLink>(links);

    tracing::info!("Bot is running!");

    // run the client
    client.start().await.map_err(Into::into)
}
