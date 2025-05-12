use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};


#[derive(Debug, Clone, Default)]
pub struct SpecialHeaders {
    pub user_agent: Option<String>,
    pub gzip: bool,
    pub deflate: bool,
    pub cookie: Option<String>,
}

pub fn parse_header(headers: Vec<String>) -> anyhow::Result<(HeaderMap, SpecialHeaders)> {
    let mut header_map = HeaderMap::new();
    let mut special_headers = SpecialHeaders::default();

    for header in headers {
        if let Some(pos) = header.find(':') {
            let (name, value) = header.split_at(pos);
            let name = name.trim().to_lowercase();
            let value = value[1..].trim();

            match name.as_str() {
                "user-agent" => {
                    special_headers.user_agent = Some(value.to_string());
                }
                "accept-encoding" => {
                    special_headers.gzip = value.contains("gzip");
                    special_headers.deflate = value.contains("deflate");
                }
                "cookie" => {
                    special_headers.cookie = Some(value.to_string());
                }
                _ => {
                    if let Ok(header_name) = HeaderName::from_str(&name) {
                        if let Ok(header_value) = HeaderValue::from_str(value) {
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
            }
        } else {
            eprintln!(
                "Invalid header format: '{}'. Expected 'Name: Value'.",
                header
            );
            return Err(anyhow::anyhow!("Invalid header format"));
        }
    }
    Ok((header_map, special_headers))
}
