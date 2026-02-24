mod app;
mod cfg;
mod cmds;
mod handlers;
mod keys;
mod link;
mod prelude;
mod stf;
mod util;

#[tokio::main]
async fn main() -> prelude::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("kun_bot", log::LevelFilter::Debug)
        .filter(Some("tracing::span"), log::LevelFilter::Error)
        .filter(Some("serenity::gateway"), log::LevelFilter::Error)
        .filter(Some("serenity::http"), log::LevelFilter::Error)
        .init();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");
    let data = prelude::Data::new().await?;
    let token = std::env::var("KUN_BOT_TOKEN")?;
    app::run(data, token.trim()).await
}
