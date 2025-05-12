use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short = 'c', long = "concurrent-count", default_value_t = 2)]
    pub concurrent_count: u16,

    #[arg(short = 'u', long = "url", default_value = "https://www.google.com")]
    pub url: String,

    #[arg(short = 't', long = "time", default_value_t = 60)]
    pub time: u64,

    #[arg(short, long)]
    pub ip: Option<IpAddr>,

    #[arg(short = 'H', long = "header", default_values_t = Vec::<String>::new())]
    pub header: Vec<String>,

    #[arg(long = "body", default_value = "")]
    pub body: String,

    #[arg(short = 'X', long = "method", default_value = "GET")]
    pub method: String,

    #[arg(long = "test", default_value_t = false)]
    pub test: bool,
}
