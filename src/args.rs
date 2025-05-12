use std::net::IpAddr;

use anyhow::{bail, Result};
use clap::Parser;
use http::{HeaderName, HeaderValue, Method};
use url::Url;

fn parse_header(s: &str) -> Result<(HeaderName, HeaderValue)> {
    let parts: Vec<_> = s.split(':').collect();

    if parts.len() != 2 {
        bail!("Invalid header format: {s}");
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    Ok((key.parse()?, value.parse()?))
}

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// concurrent thread count for download
    #[arg(short, long = "concurrent-count", default_value_t = 2)]
    pub concurrent_count: u16,

    /// url to download
    #[arg(short, long, default_value = "https://www.google.com")]
    pub url: Url,

    /// time to download
    #[arg(short, long, default_value_t = 60)]
    pub time: u64,

    /// ip to send the request to [default is automatically resolved from the url]
    #[arg(short, long)]
    pub ip: Option<IpAddr>,

    /// http header to send
    #[arg(short = 'H', long, value_parser = parse_header)]
    pub header: Vec<(HeaderName, HeaderValue)>,

    /// http method to use
    #[arg(short, long, default_value_t = Method::GET)]
    pub method: Method,
}
