use log::info;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HeadersConfig {
    pub user_agent: Option<String>,
    pub gzip: bool,
    pub deflate: bool,
    pub cookie: Option<String>,
    pub other_headers: HeaderMap,
}

impl HeadersConfig {
    pub fn log_detail(&self) {
        info!("gzip enabled: {}", self.gzip);
        info!("deflate enabled: {}", self.deflate);
        info!("cookie: {:?}", self.cookie);
        info!("user agent: {:?}", self.user_agent);
        info!("headers: {:?}", self.other_headers);
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadersPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum HeadersParseError {
    #[error("Invalid header format: {0}\nExample: \"Accept: application/json\"")]
    InvalidFormat(String),
}

impl FromStr for HeadersPair {
    type Err = HeadersParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s
            .split_once(':')
            .ok_or_else(|| HeadersParseError::InvalidFormat(s.to_string()))?;
        Ok(Self {
            key: key.trim().to_string(),
            value: value.trim().to_string(),
        })
    }
}

impl From<Vec<HeadersPair>> for HeadersConfig {
    fn from(headers: Vec<HeadersPair>) -> Self {
        let mut headers_map = HeaderMap::new();
        let mut user_agent = None;
        let mut gzip = false;
        let mut deflate = false;
        let mut cookie = None;

        for header in headers {
            match header.key.as_str() {
                "user-agent" => {
                    user_agent = Some(header.value);
                }
                "accept-encoding" => {
                    gzip = header.value.contains("gzip");
                    deflate = header.value.contains("deflate");
                }
                "cookie" => {
                    cookie = Some(header.value);
                }
                _ => {
                    if let Ok(header_name) = HeaderName::from_str(&header.key) {
                        if let Ok(header_value) = HeaderValue::from_str(&header.value) {
                            headers_map.insert(header_name, header_value);
                        }
                    }
                }
            }
        }

        Self {
            user_agent,
            gzip,
            deflate,
            cookie,
            other_headers: headers_map,
        }
    }
}
