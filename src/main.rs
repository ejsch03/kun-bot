mod app;
mod cfg;
mod cmds;
mod handlers;
mod hooks;
mod keys;
mod link;
mod logger;
mod prelude;
mod stf;
mod util;

#[tokio::main]
async fn main() {
    // init logger
    let (_guard, _cleaner) = logger::setup();

    if let Err(e) = app::run().await {
        tracing::error!(error = ?e)
    }
}
