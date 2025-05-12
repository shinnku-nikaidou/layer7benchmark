use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short = 'c', long = "concurrent-count", default_value_t = 2)]
    pub concurrent_count: u8,

    #[arg(short = 'u', long = "url", default_value = "https://www.google.com")]
    pub url: String,

    #[arg(short = 't', long = "time", default_value_t = 60)]
    pub time: u64,

    #[arg(short='i',long="ip", default_value = "")]
    pub ip: String,

    #[arg(short = 'h', long = "help", action = ArgAction::Help)]
    pub help: bool,

    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    pub version: bool,

    #[arg(long = "header", default_value = "")]
    pub header: String,
}
