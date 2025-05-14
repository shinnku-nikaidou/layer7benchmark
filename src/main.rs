mod args;
mod build_client;
mod core_request;
mod http_benchmark;
mod parse_header;
mod randomization;
mod shutdown;
mod statistic;
mod terminal;

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
    env_logger::init();
    info!("l7_flood started");
    let runtime = Runtime::new().expect("Could not build the tokio runtime");

    if let Err(error) = runtime.block_on(http_benchmark::run(args)) {
        error!("Exited with error: {}", error);
    } else {
        info!("Finished.");
    }
}
