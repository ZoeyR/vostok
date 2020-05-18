use url::Url;
use vostok_core::request::Request;

#[derive(Debug)]
pub struct GeminiRequest {
    url: Url,
}

impl Request for GeminiRequest {
    fn path(&self) -> &str {
        match self.url.path() {
            "" => "/",
            s => s,
        }
    }
}

impl GeminiRequest {
    pub fn from_line(line: &str) -> Option<Self> {
        let url = Url::parse(line).ok()?;

        Some(GeminiRequest { url })
    }
}
