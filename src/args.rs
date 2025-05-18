use std::net::IpAddr;

use crate::header::HeadersPair;
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
    #[arg(short = 'H', long = "header")]
    pub header: Vec<HeadersPair>,

    /// Request body content
    #[arg(long = "body", default_value = "")]
    pub body: String,

    /// HTTP method to use (GET, POST, PUT, DELETE, OPTIONS)
    #[arg(short = 'X', long = "method", default_value = "GET")]
    pub method: Method,

    /// Test mode - only send one request for testing or debugging
    #[arg(long = "test", default_value_t = false)]
    pub test: bool,

    /// Timeout for each request in seconds
    /// If the request takes longer than this time, it will be considered a timeout
    /// It is different from the get stream timeout, which is applied to the full request body,
    /// which in that case the time out is set to 60 seconds, different from the request timeout
    #[arg(long = "timeout", default_value_t = 10)]
    pub timeout: u64,

    /// ⚠️ If you use this option, the --url option grammar will be changed.
    /// In summary, this program will now randomly generate URLs based on your --url option.
    /// For example, if you set --url to https://www.example.com/[a-z0-9]{10},
    /// the program will randomly generate URLs like https://www.example.com/abc123xyz0,
    /// https://www.example.com/xyz789abc1 , etc., and send requests to these random URLs.
    /// If you want to use this option, please make sure you understand the grammar of the URL you set.
    /// This option can be combined with the --test option. And the --test option will only send one request to a randomly generated URL, also the --test option will print the URL which is randomly generated.
    /// Full grammar is down below and you can keep reading the following text.
    #[arg(long = "random", default_value_t = false)]
    pub random: bool,

    /// Four log levels are available: error, warn, info, and debug.
    #[arg(long = "log-level", default_value = "info")]
    pub log_level: String,

    #[arg(long = "normal-output", default_value_t = false)]
    pub normal_output: bool,

    #[arg(long = "ip-files", default_value = "")]
    pub ip_files: String,

    // if this option is set, the program will be a slave and connect to the master server
    // this program will use websocket to connect to the master server
    // and master server will send the order back to the slave
    #[arg(long = "server", default_value = "")]
    pub server: String,
}
