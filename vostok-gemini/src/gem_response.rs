use vostok_core::response::IntoResponse;

pub struct GeminiResponse {
    pub status: StatusCode,
    pub meta: String,
    pub body: Option<String>,
}

impl GeminiResponse {}

pub enum StatusCode {
    Input(u8),
    Success(u8),
    Redirect(u8),
    TemporaryFailure(u8),
    PermanentFailure(u8),
    ClientCert(u8),
}

impl StatusCode {
    pub fn bytes(&self) -> [u8; 2] {
        match self {
            StatusCode::Input(_) => [b'1', b'0'],
            StatusCode::Success(_) => [b'2', b'0'],
            StatusCode::Redirect(_) => [b'3', b'0'],
            StatusCode::TemporaryFailure(_) => [b'4', b'0'],
            StatusCode::PermanentFailure(_) => [b'5', b'0'],
            StatusCode::ClientCert(_) => [b'6', b'0'],
        }
    }
}

impl<'a> IntoResponse<GeminiResponse> for &'a str {
    fn into_response(self) -> GeminiResponse {
        GeminiResponse {
            status: StatusCode::Success(0),
            meta: "text/plain".to_string(),
            body: Some(self.to_string()),
        }
    }
}

impl<'a> From<&'a str> for GeminiResponse {
    fn from(s: &'a str) -> Self {
        GeminiResponse {
            status: StatusCode::Success(0),
            meta: "text/plain".to_string(),
            body: Some(s.to_string()),
        }
    }
}
