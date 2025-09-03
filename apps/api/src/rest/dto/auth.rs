use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct InitiateAuthRequest {
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InitiateAuthResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub success: bool,
}
