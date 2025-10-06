use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlareSolverError {
    #[error("FlareSolver request failed: {0}")]
    RequestFailed(String),

    #[error("FlareSolver returned error status: {0}")]
    ErrorStatus(String),

    #[error("FlareSolver solution not found")]
    NoSolution,

    #[error("Invalid response from FlareSolver")]
    InvalidResponse,

    #[error("Cloudflare challenge detected but not solved")]
    ChallengeNotSolved,

    #[error("Session expired")]
    SessionExpired,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

impl From<FlareSolverError> for dokusho_core::SourceError {
    fn from(err: FlareSolverError) -> Self {
        match err {
            FlareSolverError::ChallengeNotSolved => dokusho_core::SourceError::CloudflareProtection,
            FlareSolverError::Network(e) => dokusho_core::SourceError::Network(e.to_string()),
            FlareSolverError::Timeout(secs) => dokusho_core::SourceError::HTTPRequestFailed(
                format!("Request timed out after {} seconds", secs),
            ),
            _ => dokusho_core::SourceError::Other(err.into()),
        }
    }
}
