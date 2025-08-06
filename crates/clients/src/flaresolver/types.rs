use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlareSolverRequest {
    pub cmd: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_ttl_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<FlareSolverProxy>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlareSolverProxy {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlareSolverResponse {
    pub status: String,
    pub message: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub version: String,
    pub solution: Option<FlareSolverSolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlareSolverSolution {
    pub url: String,
    pub status: u16,
    pub headers: serde_json::Value,
    pub response: String,
    pub cookies: Vec<FlareSolverCookie>,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlareSolverCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<f64>,
    pub size: u32,
    pub http_only: bool,
    pub secure: bool,
    pub session: bool,
    pub same_site: Option<String>,
}

impl FlareSolverRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            cmd: "request.get".to_string(),
            url: url.into(),
            max_timeout: Some(60000), // 60 seconds default
            session: None,
            session_ttl_minutes: None,
            proxy: None,
        }
    }

    pub fn with_session(mut self, session: String) -> Self {
        self.session = Some(session);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.max_timeout = Some(timeout_ms);
        self
    }

    pub fn with_proxy(mut self, proxy: FlareSolverProxy) -> Self {
        self.proxy = Some(proxy);
        self
    }
}
