mod args;
mod benchmark;
mod output;
mod components;
mod server;
mod shutdown;
mod statistic;

use args::Args;
use clap::Parser;
use log::{error, info};
use tokio::runtime::Runtime;

const DEFAULT_LOG_LEVEL: &str = "info";

fn main() {
    let args = Args::parse();
    let _ = statistic::STATISTIC.set(statistic::Statistic::default());
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", DEFAULT_LOG_LEVEL);
    }

    if args.log_level != DEFAULT_LOG_LEVEL {
        std::env::set_var("RUST_LOG", args.log_level.as_str());
    }

    env_logger::init();
    info!("l7_flood started");
    let runtime = Runtime::new().expect("Could not build the tokio runtime");

    let result = if args.server.is_empty() {
        runtime.block_on(benchmark::run(args))
    } else {
        runtime.block_on(server::connect_to_server())
    };

    match result {
        Ok(_) => info!("Finished."),
        Err(error) => error!("Exited with error: {}", error),
    }
}
