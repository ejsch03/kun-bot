use songbird::SerenityInit;

use crate::prelude::*;

pub async fn run(data: Data, token: &str) -> Result<()> {
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
    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .register_songbird()
        .await?;
    client.data.write().await.insert::<MessageLink>(links);

    // run the client
    client.start().await.map_err(Into::into)
}
