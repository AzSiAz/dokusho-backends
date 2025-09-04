use axum::{
    Json, Router,
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use openidconnect::IssuerUrl;
use openidconnect::core::CoreProviderMetadata;
use reqwest::Client;
use serde::Deserialize;
use vite_rs_axum_0_8::ViteServe;

#[derive(vite_rs::Embed)]
#[root = "./frontend"]
#[dev_server_port = "5173"]
struct Assets;

#[derive(Clone)]
pub struct DashboardState {
    http: Client,
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    redirect_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardConfig {
    pub issuer_url: String,
    pub public_client_id: String,
    pub redirect_url: String,
}

pub async fn router(cfg: DashboardConfig) -> anyhow::Result<Router> {
    let issuer = IssuerUrl::new(cfg.issuer_url.clone())?;
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let provider = CoreProviderMetadata::discover_async(issuer, &http_client).await?;

    let state = DashboardState {
        http: Client::new(),
        authorization_endpoint: provider.authorization_endpoint().url().to_string(),
        token_endpoint: provider
            .token_endpoint()
            .expect("token endpoint available")
            .url()
            .to_string(),
        client_id: cfg.public_client_id,
        redirect_url: cfg.redirect_url,
    };

    let app = Router::new()
        .route("/dashboard/oidc.json", get(oidc_json))
        .route("/img", get(image_proxy))
        .with_state(state)
        .route_service("/", ViteServe::new(Assets::boxed()))
        .route_service("/{*path}", ViteServe::new(Assets::boxed()));

    Ok(app)
}

#[derive(Deserialize)]
struct ImgQuery {
    u: String,
}

async fn image_proxy(
    State(state): State<DashboardState>,
    Query(q): Query<ImgQuery>,
) -> impl IntoResponse {
    let Ok(url) = reqwest::Url::parse(&q.u) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match url.scheme() {
        "http" | "https" => {}
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let mut req = state
        .http
        .get(url.clone())
        .header("User-Agent", "dokusho-adminboard/1.0");
    if let Some(host) = url.host_str()
        && host.ends_with("mangadex.org")
    {
        req = req.header("Referer", "https://mangadex.org/");
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let ct = resp
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .cloned()
                .unwrap_or_else(|| axum::http::HeaderValue::from_static("image/jpeg"));
            let bytes = resp.bytes().await.unwrap_or_default();
            Response::builder()
                .status(status)
                .header(axum::http::header::CONTENT_TYPE, ct)
                .body(Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
    }
}

async fn oidc_json(State(state): State<DashboardState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "authorization_endpoint": state.authorization_endpoint,
        "token_endpoint": state.token_endpoint,
        "client_id": state.client_id,
        "redirect_uri": state.redirect_url,
    }))
}
