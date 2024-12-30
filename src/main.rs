use autodok::config::Config;
use autodok::run;

#[tokio::main]
async fn main() {
    let config = Config::new();

    let format = tracing_subscriber::fmt::format().with_target(false);
    tracing_subscriber::fmt().event_format(format).init();

    ctrlc::set_handler(|| {
        log::info!("Stopping autodok...");
        std::process::exit(0);
    })
    .expect("Error setting exit handler");

    log::info!("Starting autodok...");

    run(&config).await.unwrap();
}
