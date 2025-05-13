use std::net::IpAddr;

use clap::Parser;
use reqwest::Method;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Concurrent thread count for download
    #[arg(short = 'c', long = "concurrent-count", default_value_t = 2)]
    pub concurrent_count: u16,

    /// URL to download
    #[arg(short = 'u', long = "url", default_value = "https://www.google.com")]
    pub url: String,

    /// Time in seconds to run the benchmark
    #[arg(short = 't', long = "time", default_value_t = 60)]
    pub time: u64,

    /// IP address to send the request to (default is automatically resolved from the URL)
    /// If you have already found the original ip address, 
    /// you can use this option to bypass the CDN or some random WAF
    #[arg(short, long)]
    pub ip: Option<IpAddr>,

    /// HTTP headers to send (same format as curl's -H option)
    #[arg(short = 'H', long = "header", default_values_t = Vec::<String>::new())]
    pub header: Vec<String>,

    /// Request body content
    #[arg(long = "body", default_value = "")]
    pub body: String,

    /// HTTP method to use (GET, POST, PUT, DELETE, OPTIONS)
    #[arg(short = 'X', long = "method", default_value = "GET")]
    pub method: Method,

    /// Test mode - only send one request for testing or debugging
    #[arg(long = "test", default_value_t = false)]
    pub test: bool,
}
