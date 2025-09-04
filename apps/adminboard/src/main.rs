use std::net::SocketAddr;

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Method, Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
};
use maud::{DOCTYPE, PreEscaped, html};
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{
    ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    api_base: String,
    http: Client,
    // OpenID provider metadata and config for login
    oidc_client_id: String,
    oidc_provider: CoreProviderMetadata,
    redirect_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    host: String,
    port: u16,
    api_base: String,
    auth_issuer_url: String,
    auth_public_client_id: String,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        let host = std::env::var("ADMINBOARD_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = std::env::var("ADMINBOARD_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8081);
        let api_base = std::env::var("ADMINBOARD_API_BASE")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let auth_issuer_url = std::env::var("AUTH_ISSUER_URL")
            .unwrap_or_else(|_| "http://localhost:4455/realms/example".to_string());
        let auth_public_client_id = std::env::var("AUTH_PUBLIC_CLIENT_ID")
            .unwrap_or_else(|_| "dokusho-adminboard".to_string());
        Ok(Self {
            host,
            port,
            api_base,
            auth_issuer_url,
            auth_public_client_id,
        })
    }
}

// no embedded assets; all HTML is rendered with maud

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cfg = Config::from_env()?;

    // Discover OpenID provider
    let issuer = IssuerUrl::new(cfg.auth_issuer_url.clone())?;
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let provider = CoreProviderMetadata::discover_async(issuer.clone(), &http_client).await?;

    // Compute redirect URL
    let redirect_url = format!("http://{}:{}/auth/callback", cfg.host, cfg.port);

    let state = AppState {
        api_base: cfg.api_base.clone(),
        http: Client::new(),
        oidc_client_id: cfg.auth_public_client_id.clone(),
        oidc_provider: provider,
        redirect_url: redirect_url.clone(),
    };

    // Router: UI, OpenID login/logout/callback, and REST proxy
    let app = Router::new()
        .route("/", get(index))
        .route("/auth/login", get(auth_login))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/callback", get(auth_callback))
        .route("/img", get(image_proxy))
        .route("/api/{*path}", any(rest_proxy))
        .route("/assets/{*path}", get(static_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], cfg.port)));
    tracing::info!("Adminboard listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

async fn index() -> impl IntoResponse {
    let page = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Dokusho Adminboard" }
                style { (PreEscaped(r#"
                  :root { color-scheme: light dark; }
                  *, *::before, *::after { box-sizing: border-box; }
                  body { margin: 0; font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Arial; }
                  header { padding: 12px 16px; border-bottom: 1px solid #2b3754; background: #0f152b; color: #e6edf3; display:flex; align-items:center; gap:12px; }
                  .tab { background: transparent; border-color: #2b3754; }
                  .tab.active { background: #223056; }
                  main { padding: 16px; }
                  input, select, button { padding: 8px 10px; border-radius: 8px; border: 1px solid #2b3754; background: #0b1020; color: #e6edf3; }
                  button { background: #1a2442; cursor: pointer; }
                  button:hover { background: #223056; }
                  button.secondary { background: #0b1020; }
                  button.secondary:hover { background: #131d39; }
                  button:disabled { opacity: .6; cursor: not-allowed; }
                  .row { display:flex; gap: 8px; flex-wrap: wrap; align-items: center; }
                  .grid { display:grid; grid-template-columns: repeat(auto-fill, minmax(240px,1fr)); gap: 12px; margin-top: 12px; }
                  .card { border: 1px solid #2b3754; border-radius: 12px; padding: 12px; background: #0f152b; }
                  .card.exists .cover { opacity: .4; filter: grayscale(20%); }
                  .muted { color: #9fb1d1; }
                  img.cover { width: 100%; height: 320px; object-fit: cover; border-radius: 8px; background: #223056; }
                  .topbar-spacer { flex:1; }
                  /* Filters UI */
                  #filters { margin-top: 8px; }
                  .filters-grid { display:grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }
                  .filter-card { background:#0f152b; border:1px solid #2b3754; border-radius: 12px; padding: 12px; }
                  .filter-title { font-size: .9rem; color:#9fb1d1; margin-bottom: 6px; }
                  .w-full { width: 100%; }
                  .filter-card input,
                  .filter-card select { width: 100%; max-width: 100%; display: block; box-sizing: border-box; }
                  .chips { display:flex; flex-wrap: wrap; gap: 6px; }
                  .chip { display:inline-flex; align-items:center; }
                  .chip input { display:none; }
                  .chip span { padding:6px 10px; border-radius: 999px; border:1px solid #2b3754; background:#0b1020; color:#e6edf3; font-size:.9rem; }
                  .chip input:checked + span { background:#223056; border-color:#3b4b7a; }
                  .stack { display:flex; flex-direction:column; gap:6px; }
                  .note { font-size:.85rem; color:#9fb1d1; }
                  #toast-container { position: fixed; right: 16px; bottom: 16px; display: flex; flex-direction: column; gap: 8px; z-index: 9999; }
                  .toast { padding: 10px 12px; border-radius: 8px; color: #e6edf3; background: #1a2442; border: 1px solid #2b3754; box-shadow: 0 6px 24px rgba(0,0,0,.2); opacity: 0; transform: translateY(8px); animation: toast-in .18s ease-out forwards; }
                  .toast.success { background: #184a2c; border-color: #2b7a46; }
                  .toast.error { background: #5a1d1d; border-color: #8a2f2f; }
                  .toast.info { background: #1a2442; border-color: #2b3754; }
                  @keyframes toast-in { to { opacity: 1; transform: translateY(0); } }
                "#)) }
            }
            body {
                header {
                    strong { "Adminboard" }
                    button id="tab-sources" class="tab active" { "Sources" }
                    button id="tab-added" class="tab" { "Added" }
                    span class="topbar-spacer" {}
                    button id="signin" { "Sign in" }
                    button id="signout" style="display:none" { "Sign out" }
                }
                main {
                    section id="controls-sources" class="row" {
                        label { "Source" }
                        select id="source" {}
                        input id="query" placeholder="Search query" {}
                        button id="search" { "Search" }
                    }
                    div id="filters" {}
                    section id="controls-pager" class="row" style="margin-top:8px" {
                        button id="prev" { "Prev" }
                        span { "Page: " span id="page" { "1" } }
                        button id="next" { "Next" }
                    }
                    div id="results" class="grid" {}
                }
                div id="toast-container" {}
                script { (PreEscaped(r#"
                  function toast(message, type='info', timeout=3200) {
                    try {
                      const c = document.getElementById('toast-container');
                      if (!c) return;
                      const el = document.createElement('div');
                      el.className = 'toast ' + type;
                      el.textContent = message;
                      el.title = 'Click to dismiss';
                      c.appendChild(el);
                      const dismiss = () => { try { el.style.transition = 'opacity .2s ease'; el.style.opacity = '0'; setTimeout(() => { if (el.parentNode) el.parentNode.removeChild(el); }, 220); } catch(_){} };
                      el.addEventListener('click', dismiss);
                      setTimeout(dismiss, timeout);
                    } catch(_) { /* no-op */ }
                  }

                  const qs = s => document.querySelector(s);
                  const elSource = qs('#source');
                  const elQuery = qs('#query');
                  const elSignIn = qs('#signin');
                  const elSignOut = qs('#signout');
                  const elResults = qs('#results');
                  const elFilters = qs('#filters');
                  const elPrev = qs('#prev');
                  const elNext = qs('#next');
                  const elPage = qs('#page');
                  const elTabSources = qs('#tab-sources');
                  const elTabAdded = qs('#tab-added');
                  const elControlsSources = qs('#controls-sources');
                  const elControlsPager = qs('#controls-pager');
                  let mode = 'sources';
                  let currentPage = 1;
                  let hasNextPage = false;
                  let currentPageAdded = 1;
                  let hasNextPageAdded = false;
                  let sourcesMeta = {};

                  const updateAuthButtons = async () => {
                    try {
                      const r = await fetch('/api/users/me');
                      if (r.ok) { elSignIn.style.display = 'none'; elSignOut.style.display = ''; }
                      else { elSignIn.style.display = ''; elSignOut.style.display = 'none'; }
                    } catch (_) { elSignIn.style.display = ''; elSignOut.style.display = 'none'; }
                  };

                  elSignIn.addEventListener('click', async () => {
                    window.location.href = '/auth/login';
                  });

                  elSignOut.addEventListener('click', async () => {
                    try { await fetch('/auth/logout', { method: 'POST' }); } catch(_) {}
                    updateAuthButtons();
                  });

                  const loadSources = async () => {
                    elSource.innerHTML = '<option>Loading…</option>';
                    try {
                      const r = await fetch('/api/sources');
                      if (!r.ok) throw new Error('Failed to load sources');
                      const data = await r.json();
                      elSource.innerHTML = data.map(s => `<option value="${s.id}">${s.name}</option>`).join('');
                      sourcesMeta = Object.fromEntries(data.map(s => [s.id, s]));
                      renderFilters();
                    } catch (e) {
                      elSource.innerHTML = '';
                      console.error(e);
                      toast('Failed to load sources. Are you signed in?', 'error');
                    }
                  };

                  const renderFilters = () => {
                    const sid = elSource.value;
                    const meta = sourcesMeta[sid];
                    if (!meta || !meta.filters) { elFilters.innerHTML = ''; return; }
                    const f = meta.filters;
                    const toArr = (v) => Array.isArray(v) ? v : (v && typeof v === 'object' ? Object.values(v) : (v ? [v] : []));
                    const opt = (v) => `<option value="${v}">${v}</option>`;
                    const card = (title, body) => `
                      <div class="filter-card">
                        <div class="filter-title">${title}</div>
                        ${body}
                      </div>`;
                    let parts = [];
                    const orders = toArr(f.order);
                    if (orders.length) {
                      parts.push(card('Order', `<select id="f-order" class="w-full"><option value="">(any)</option>${orders.map(opt).join('')}</select>`));
                    }
                    const sorts = toArr(f.sort);
                    if (sorts.length) {
                      parts.push(card('Sort', `<select id="f-sort" class="w-full"><option value="">(any)</option>${sorts.map(opt).join('')}</select>`));
                    }
                    if (f.artists) {
                      parts.push(card('Artists', `<input id="f-artists" class="w-full" placeholder="Comma separated" />`));
                    }
                    if (f.authors) {
                      parts.push(card('Authors', `<input id="f-authors" class="w-full" placeholder="Comma separated" />`));
                    }
                    const types = toArr(f.types);
                    if (types.length) {
                      const chips = types.map(v => `<label class='chip'><input type='checkbox' value='${v}'/><span>${v}</span></label>`).join('');
                      parts.push(card('Types', `<div id="f-types" class="chips">${chips}</div>`));
                    }
                    const status = toArr(f.status);
                    if (status.length) {
                      const chips = status.map(v => `<label class='chip'><input type='checkbox' value='${v}'/><span>${v}</span></label>`).join('');
                      parts.push(card('Status', `<div id="f-status" class="chips">${chips}</div>`));
                    }
                    if (f.genres && f.genres.accepted_values) {
                      const opts = toArr(f.genres.accepted_values).map(opt).join('');
                      if (f.genres.include) parts.push(card('Genres: Include', `<select id="f-genres-inc" class="w-full" multiple size="8">${opts}</select>`));
                      if (f.genres.exclude) parts.push(card('Genres: Exclude', `<select id="f-genres-exc" class="w-full" multiple size="8">${opts}</select>`));
                    }
                    elFilters.innerHTML = `<div class="filters-grid">${parts.join('')}</div><div class="row"><button id="filters-clear" class="secondary">Clear filters</button></div>`;
                    const clear = document.getElementById('filters-clear');
                    if (clear) {
                      clear.addEventListener('click', () => {
                        const wrap = elFilters;
                        wrap.querySelectorAll('input[type="text"]').forEach(i => i.value = '');
                        wrap.querySelectorAll('select').forEach(s => {
                          if (s.multiple) Array.from(s.options).forEach(o => o.selected = false); else s.value = '';
                        });
                        wrap.querySelectorAll('input[type="checkbox"]').forEach(c => c.checked = false);
                      });
                    }
                  };

                  const pickTitle = (ml, fallback='') => {
                    if (!ml || typeof ml !== 'object') return fallback;
                    // API returns keys like En, Fr, Jp, JpRo, Ko, ZhHk, Zh
                    const pref = ['En','Fr','Jp','JpRo','Ko','ZhHk','Zh'];
                    const pick = (v) => {
                      if (!v) return undefined;
                      if (typeof v === 'string') return v;
                      if (Array.isArray(v) && v.length) return v[0];
                      return undefined;
                    };
                    for (const k of pref) { const r = pick(ml[k]); if (r) return r; }
                    const vals = Object.values(ml).map(pick).filter(Boolean);
                    return vals[0] || fallback;
                  };

                  const card = (sourceId, s) => `
                    <div class="card" data-serie="${s.id}">
                      <img class="cover" src="/img?u=${encodeURIComponent(s.cover)}" alt="cover" />
                      <h3>${pickTitle(s.title, s.id)}</h3>
                      <div class="row">
                        <button class="create" data-source="${sourceId}" data-serie="${s.id}">Create serie</button>
                      </div>
                    </div>
                  `;

                  const bindCreateButtons = () => {
                    document.querySelectorAll('button.create').forEach(btn => {
                      if (btn.__bound) return; // avoid double-binding on rerenders
                      btn.__bound = true;
                      btn.addEventListener('click', async (ev) => {
                        const el = ev.currentTarget;
                        const sourceId = el.getAttribute('data-source');
                        const serieId = el.getAttribute('data-serie');
                        const card = el.closest('.card');
                        const prevText = el.textContent;
                        el.disabled = true;
                        el.textContent = 'Creating…';
                        try {
                          const r = await fetch('/api/admin/series/from-source', {
                            method: 'POST',
                            headers: { 'content-type': 'application/json' },
                            body: JSON.stringify({ source_id: sourceId, serie_id: serieId })
                          });
                          if (r.ok) {
                            if (card) card.classList.add('exists');
                            el.textContent = 'Already in DB';
                            toast('Serie created successfully', 'success');
                          } else {
                            el.disabled = false;
                            el.textContent = prevText;
                            toast('Serie creation failed: ' + (await r.text()), 'error');
                          }
                        } catch (e) {
                          el.disabled = false;
                          el.textContent = prevText;
                          toast('Creation failed: ' + e.message, 'error');
                        }
                      });
                    });
                  };

                  const search = async (page = 1) => {
                    const sourceId = elSource.value;
                    const query = elQuery.value || '';
                    elResults.innerHTML = '';
                    try {
                      // Build filters object from UI (query + page)
                      const filters = { query, page };
                      const readCSV = (id) => {
                        const el = document.getElementById(id); if (!el) return undefined;
                        const v = el.value.trim(); if (!v) return undefined;
                        return v.split(',').map(s => s.trim()).filter(Boolean);
                      };
                      const readSelect = (id) => {
                        const el = document.getElementById(id); if (!el) return undefined;
                        const v = el.value; if (!v) return undefined; return v;
                      };
                      const readChecks = (id) => {
                        const wrap = document.getElementById(id); if (!wrap) return undefined;
                        const vals = Array.from(wrap.querySelectorAll('input[type="checkbox"]:checked')).map(i => i.value);
                        return vals.length ? vals : undefined;
                      };
                      const readMulti = (id) => {
                        const el = document.getElementById(id); if (!el) return undefined;
                        const vals = Array.from(el.selectedOptions).map(o => o.value);
                        return vals.length ? vals : undefined;
                      };
                      const meta = sourcesMeta[sourceId]?.filters || {};
                      if (meta.order) { const v = readSelect('f-order'); if (v) filters.order = v; }
                      if (meta.sort) { const v = readSelect('f-sort'); if (v) filters.sort = v; }
                      if (meta.artists) { const v = readCSV('f-artists'); if (v) filters.artists = v; }
                      if (meta.authors) { const v = readCSV('f-authors'); if (v) filters.authors = v; }
                      if (meta.types) { const v = readChecks('f-types'); if (v) filters.types = v; }
                      if (meta.status) { const v = readChecks('f-status'); if (v) filters.status = v; }
                      if (meta.genres) {
                        const includes = readMulti('f-genres-inc');
                        const excludes = readMulti('f-genres-exc');
                        if ((includes && includes.length) || (excludes && excludes.length)) {
                          filters.genres = { includes, excludes };
                        }
                      }

                      const r = await fetch(`/api/sources/${encodeURIComponent(sourceId)}/series/search`, {
                        method: 'POST', headers: { 'content-type': 'application/json' },
                        body: JSON.stringify(filters)
                      });
                      if (!r.ok) throw new Error('Search failed');
                      const result = await r.json();
                      hasNextPage = result.has_next_page;
                      elPage.textContent = String(page);
                      elPrev.disabled = page <= 1;
                      elNext.disabled = !hasNextPage;
                      const series = result.series;
                      elResults.innerHTML = series.map(s => card(sourceId, s)).join('');

                      // Check existing series in DB
                      const ids = series.map(s => s.id);
                      if (ids.length) {
                        try {
                          const r2 = await fetch('/api/admin/series/existing', {
                            method: 'POST', headers: { 'content-type': 'application/json' },
                            body: JSON.stringify({ source_id: sourceId, external_ids: ids })
                          });
                          const arr = r2.ok ? await r2.json() : [];
                          const existing = new Set(arr.map(x => x.external_id));
                          // Mark cards
                          document.querySelectorAll('#results .card').forEach(c => {
                            const id = c.getAttribute('data-serie');
                            const btn = c.querySelector('button.create');
                            if (existing.has(id)) {
                              c.classList.add('exists');
                              if (btn) { btn.disabled = true; btn.textContent = 'Already in DB'; }
                            }
                          });
                        } catch (e) {
                          console.warn('existence check failed', e);
                        }
                      }

                      bindCreateButtons();
                    } catch (e) {
                      console.error(e);
                      toast('Search failed: ' + e.message, 'error');
                    }
                  };

                  const loadAdded = async (page = 1) => {
                    elResults.innerHTML = '';
                    try {
                      const r = await fetch(`/api/admin/series?page=${page}&per_page=24`);
                      if (!r.ok) throw new Error('Failed to load');
                      const result = await r.json();
                      hasNextPageAdded = result.has_next_page;
                      elPage.textContent = String(page);
                      elPrev.disabled = page <= 1;
                      elNext.disabled = !hasNextPageAdded;
                      const series = result.series;
                      elResults.innerHTML = series.map(s => `
                        <div class="card" data-serie="${s.id}">
                          <img class="cover" src="/img?u=${encodeURIComponent(s.cover)}" alt="cover" />
                          <h3>${pickTitle(s.title, s.id)}</h3>
                        </div>
                      `).join('');
                    } catch (e) {
                      console.error(e);
                      toast('Load added series failed: ' + e.message, 'error');
                    }
                  };

                  document.querySelector('#search').addEventListener('click', () => { currentPage = 1; search(currentPage); });
                  elPrev.addEventListener('click', () => {
                    if (mode === 'sources') {
                      if (currentPage > 1) { currentPage -= 1; search(currentPage); }
                    } else {
                      if (currentPageAdded > 1) { currentPageAdded -= 1; loadAdded(currentPageAdded); }
                    }
                  });
                  elNext.addEventListener('click', () => {
                    if (mode === 'sources') {
                      if (hasNextPage) { currentPage += 1; search(currentPage); }
                    } else {
                      if (hasNextPageAdded) { currentPageAdded += 1; loadAdded(currentPageAdded); }
                    }
                  });

                  const switchMode = (m) => {
                    mode = m;
                    elTabSources.classList.toggle('active', m === 'sources');
                    elTabAdded.classList.toggle('active', m === 'added');
                    elControlsSources.style.display = m === 'sources' ? '' : 'none';
                    elFilters.style.display = m === 'sources' ? '' : 'none';
                    elControlsPager.style.display = '';
                    if (m === 'sources') {
                      elPage.textContent = String(currentPage);
                      elPrev.disabled = currentPage <= 1;
                      elNext.disabled = !hasNextPage;
                      search(currentPage);
                    } else {
                      elPage.textContent = String(currentPageAdded);
                      elPrev.disabled = currentPageAdded <= 1;
                      elNext.disabled = !hasNextPageAdded;
                      loadAdded(currentPageAdded);
                    }
                  };

                  elTabSources.addEventListener('click', () => switchMode('sources'));
                  elTabAdded.addEventListener('click', () => switchMode('added'));
                  updateAuthButtons();
                  elSource.addEventListener('change', () => { renderFilters(); });
                  loadSources().then(() => search(currentPage)).catch(() => {});
                "#)) }
            }
        }
    };
    Html(page.into_string())
}

async fn static_handler(Path(_path): Path<String>) -> impl IntoResponse {
    StatusCode::NOT_FOUND
}

#[derive(Deserialize)]
struct ImgQuery {
    u: String,
}

// Simple image proxy to avoid hotlinking blocks (e.g., MangaDex)
async fn image_proxy(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<ImgQuery>,
) -> impl IntoResponse {
    let Ok(url) = reqwest::Url::parse(&q.u) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match url.scheme() {
        "http" | "https" => {}
        _ => return StatusCode::BAD_REQUEST.into_response(),
    }

    // Build request; add Referer for known hosts that block hotlinking
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
        Err(e) => {
            tracing::warn!("image proxy error: {}", e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn rest_proxy(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    uri: Uri,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    // Build target: api_base + /api/v1 + /{path} + query
    let mut target = format!("{}/api/v1/{}", state.api_base.trim_end_matches('/'), path);
    if let Some(q) = uri.query() {
        target.push('?');
        target.push_str(q);
    }

    // Read body bytes (for non-GET/HEAD)
    let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    let mut builder = state.http.request(method.clone(), &target);

    // Forward content-type if present
    if let Some(ct) = headers.get(axum::http::header::CONTENT_TYPE) {
        builder = builder.header(axum::http::header::CONTENT_TYPE, ct);
    }

    // Prefer Authorization from cookie; fallback to incoming header
    // Parse access_token from Cookie
    let mut auth_set = false;
    if let Some(cookie_hdr) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        && let Some(tok) = find_cookie(cookie_hdr, "access_token")
    {
        builder = builder.header(axum::http::header::AUTHORIZATION, format!("Bearer {}", tok));
        auth_set = true;
    }
    if !auth_set && let Some(auth) = headers.get(axum::http::header::AUTHORIZATION) {
        builder = builder.header(axum::http::header::AUTHORIZATION, auth);
    }

    match builder.body(bytes).send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>();
            let bytes = resp.bytes().await.unwrap_or_default();
            let mut out = Response::builder().status(status);
            for (k, v) in headers {
                if k != axum::http::header::CONTENT_LENGTH {
                    out = out.header(k, v);
                }
            }
            out.body(Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

fn find_cookie(all: &str, name: &str) -> Option<String> {
    for part in all.split(';') {
        let p = part.trim();
        if let Some(val) = p.strip_prefix(&format!("{}=", name)) {
            return Some(val.to_string());
        }
    }
    None
}

// Minimal callback page to store JWT from `?token=` and redirect
async fn auth_callback(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract state and code from query
    let full = uri.to_string();
    let url = url::Url::parse(&format!("http://dummy.local{}", full))
        .unwrap_or_else(|_| url::Url::parse("http://dummy.local/").unwrap());
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string());
    let state_param = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string());

    if code.is_none() || state_param.is_none() {
        return (StatusCode::BAD_REQUEST, "Missing code/state").into_response();
    }

    // Read pkce_verifier and auth_state from cookies
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let stored_state = find_cookie(&cookie_header, "auth_state");
    let pkce_verifier = find_cookie(&cookie_header, "pkce_verifier");

    if stored_state.as_deref() != state_param.as_deref() || pkce_verifier.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid auth state").into_response();
    }

    // Exchange code for tokens
    let client = CoreClient::from_provider_metadata(
        state.oidc_provider.clone(),
        ClientId::new(state.oidc_client_id.clone()),
        None,
    )
    .set_redirect_uri(RedirectUrl::new(state.redirect_url.clone()).unwrap());

    let token_response = client
        .exchange_code(openidconnect::AuthorizationCode::new(code.unwrap()))
        .expect("code exchange configuration failed")
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.unwrap()))
        .request_async(&state.http)
        .await;

    let Ok(token_response) = token_response else {
        return (StatusCode::BAD_GATEWAY, "Token exchange failed").into_response();
    };

    let access_token = token_response.access_token().secret().to_string();

    // Set HttpOnly cookie with access token and clear temporary cookies
    // Build response with cookies set

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8");
    // Cookies
    let cookies = vec![
        // access token cookie
        format!(
            "access_token={}; Path=/; HttpOnly; SameSite=Lax",
            access_token
        ),
        // clear pkce/state
        "pkce_verifier=; Path=/; Max-Age=0".to_string(),
        "auth_state=; Path=/; Max-Age=0".to_string(),
    ];
    for c in cookies {
        resp = resp.header(axum::http::header::SET_COOKIE, c);
    }

    let body = html! {
        (DOCTYPE)
        html { head { meta charset="utf-8"; title { "Auth Callback" } }
          body style="font-family:system-ui;padding:1rem" {
            script { (PreEscaped(r#"
              (function(){
                var to = sessionStorage.getItem('post_auth_redirect') || '/';
                sessionStorage.removeItem('post_auth_redirect');
                window.location.replace(to);
              })();
            "#)) }
            p { "Finishing authentication…" }
          }
        }
    };

    resp.body(Body::from(body.into_string()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn auth_login(State(state): State<AppState>) -> impl IntoResponse {
    // Build auth URL with PKCE and state
    let client = CoreClient::from_provider_metadata(
        state.oidc_provider.clone(),
        ClientId::new(state.oidc_client_id.clone()),
        None,
    )
    .set_redirect_uri(RedirectUrl::new(state.redirect_url.clone()).unwrap());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_state, _nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(openidconnect::Scope::new("openid".to_string()))
        .add_scope(openidconnect::Scope::new("profile".to_string()))
        .add_scope(openidconnect::Scope::new("email".to_string()))
        .add_scope(openidconnect::Scope::new("groups".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let mut resp = Response::builder()
        .status(StatusCode::FOUND)
        .header(axum::http::header::LOCATION, auth_url.to_string());

    // Persist PKCE verifier and state in short-lived cookies (10 minutes)
    let cookies = vec![
        format!(
            "pkce_verifier={}; Path=/; HttpOnly; SameSite=Lax",
            pkce_verifier.secret()
        ),
        format!(
            "auth_state={}; Path=/; HttpOnly; SameSite=Lax",
            csrf_state.secret()
        ),
    ];
    for c in cookies {
        resp = resp.header(axum::http::header::SET_COOKIE, c);
    }

    resp.body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn auth_logout() -> impl IntoResponse {
    let mut resp = Response::builder().status(StatusCode::NO_CONTENT);
    // Clear access token cookie
    resp = resp.header(
        axum::http::header::SET_COOKIE,
        "access_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
    );
    resp.body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
