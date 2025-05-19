mod args;
mod components;
mod statistic;

use crate::components::client::benchmark;
use crate::components::controlled_mode::server::connect_to_server;
use args::Args;
use clap::Parser;
use log::{error, info};
use tokio::runtime::Runtime;

const DEFAULT_LOG_LEVEL: &str = "info";

fn main() {
    let args = Args::parse();
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe { std::env::set_var("RUST_LOG", DEFAULT_LOG_LEVEL) };
    }

    if args.log_level != DEFAULT_LOG_LEVEL {
        unsafe { std::env::set_var("RUST_LOG", args.log_level.as_str()) };
    }

    env_logger::init();
    info!("l7_flood started");
    let runtime = Runtime::new().expect("Could not build the tokio runtime");

    let result = if let Some(server_url) = args.server {
        runtime.block_on(connect_to_server(server_url))
    } else {
        runtime.block_on(benchmark::run(args))
    };

    match result {
        Ok(_) => info!("Finished."),
        Err(error) => error!("Exited with error: {}", error),
    }
}
