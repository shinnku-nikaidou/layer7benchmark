use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn parse_header(header: Vec<String>) -> anyhow::Result<HeaderMap> {
    let mut parsed_headers = HeaderMap::new();
    for h in header {
        let parts: Vec<&str> = h.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid header format: {}", h));
        }
        let key = parts[0].trim();
        let value = parts[1].trim();
        parsed_headers.insert(key.parse::<HeaderName>()?, HeaderValue::from_str(value)?);
    }
    Ok(parsed_headers)
}
