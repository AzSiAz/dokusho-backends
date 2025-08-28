use std::net::SocketAddr;

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use maud::{DOCTYPE, PreEscaped, html};
use reqwest::Client;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    api_base: String,
    http: Client,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    host: String,
    port: u16,
    api_base: String,
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
        Ok(Self {
            host,
            port,
            api_base,
        })
    }
}

// no embedded assets; all HTML is rendered with maud

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cfg = Config::from_env()?;

    let state = AppState {
        api_base: cfg.api_base.clone(),
        http: Client::new(),
    };

    // Router: static assets, /auth/callback, and /graphql proxy
    let app = Router::new()
        .route("/", get(index))
        .route("/auth/callback", get(auth_callback))
        .route("/graphql", post(graphql_proxy))
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

                  const GQL = async (query, variables={}) => {
                    const jwt = localStorage.getItem('jwt');
                    const headers = { 'content-type': 'application/json' };
                    if (jwt) headers['authorization'] = 'Bearer ' + jwt;
                    const r = await fetch('/graphql', { method:'POST', headers, body: JSON.stringify({ query, variables }) });
                    const j = await r.json();
                    if (j.errors) throw new Error(j.errors.map(e => e.message).join('\n'));
                    return j.data;
                  };

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

                  const updateAuthButtons = () => {
                    const jwt = localStorage.getItem('jwt');
                    elSignIn.style.display = jwt ? 'none' : '';
                    elSignOut.style.display = jwt ? '' : 'none';
                  };

                  elSignIn.addEventListener('click', async () => {
                    try {
                      const redirect_uri = window.location.origin + '/auth/callback';
                      const q = `mutation($redirect_uri: String!) { initiate_authentication(redirect_uri: $redirect_uri) { authorization_url state } }`;
                      const data = await GQL(q, { redirect_uri });
                      sessionStorage.setItem('post_auth_redirect', '/');
                      window.location.href = data.initiate_authentication.authorization_url;
                    } catch (e) { toast('Sign-in failed: ' + e.message, 'error'); }
                  });

                  elSignOut.addEventListener('click', async () => {
                    try {
                      const q = `mutation { logout }`;
                      await GQL(q);
                    } catch (_) {}
                    localStorage.removeItem('jwt');
                    updateAuthButtons();
                  });

                  const loadSources = async () => {
                    elSource.innerHTML = '<option>Loading…</option>';
                    try {
                      const q = `query { sources { id name filters { query order sort artists authors types genres { include exclude accepted_values } status } } }`;
                      const data = await GQL(q);
                      elSource.innerHTML = data.sources.map(s => `<option value="${s.id}">${s.name}</option>`).join('');
                      sourcesMeta = Object.fromEntries(data.sources.map(s => [s.id, s]));
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
                    const opt = (v) => `<option value="${v}">${v}</option>`;
                    const card = (title, body) => `
                      <div class="filter-card">
                        <div class="filter-title">${title}</div>
                        ${body}
                      </div>`;
                    let parts = [];
                    if (f.order && f.order.length) {
                      parts.push(card('Order', `<select id="f-order" class="w-full"><option value="">(any)</option>${f.order.map(opt).join('')}</select>`));
                    }
                    if (f.sort && f.sort.length) {
                      parts.push(card('Sort', `<select id="f-sort" class="w-full"><option value="">(any)</option>${f.sort.map(opt).join('')}</select>`));
                    }
                    if (f.artists) {
                      parts.push(card('Artists', `<input id="f-artists" class="w-full" placeholder="Comma separated" />`));
                    }
                    if (f.authors) {
                      parts.push(card('Authors', `<input id="f-authors" class="w-full" placeholder="Comma separated" />`));
                    }
                    if (f.types && f.types.length) {
                      const chips = f.types.map(v => `<label class='chip'><input type='checkbox' value='${v}'/><span>${v}</span></label>`).join('');
                      parts.push(card('Types', `<div id="f-types" class="chips">${chips}</div>`));
                    }
                    if (f.status && f.status.length) {
                      const chips = f.status.map(v => `<label class='chip'><input type='checkbox' value='${v}'/><span>${v}</span></label>`).join('');
                      parts.push(card('Status', `<div id="f-status" class="chips">${chips}</div>`));
                    }
                    if (f.genres && f.genres.accepted_values && f.genres.accepted_values.length) {
                      const opts = f.genres.accepted_values.map(opt).join('');
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
                    const pref = ['en','fr','jp','jp_ro','ko','zh_hk','zh'];
                    for (const k of pref) {
                      const v = ml[k];
                      if (Array.isArray(v) && v.length) return v[0];
                    }
                    for (const k of Object.keys(ml)) {
                      const v = ml[k];
                      if (Array.isArray(v) && v.length) return v[0];
                    }
                    return fallback;
                  };

                  const card = (sourceId, s) => `
                    <div class="card" data-serie="${s.id}">
                      <img class="cover" src="${s.cover}" alt="cover" />
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
                          const q = `mutation($sid: String!, $id: String!) { create_serie_from_source(sourceId: $sid, serieId: $id) }`;
                          const data = await GQL(q, { sid: sourceId, id: serieId });
                          if (data.create_serie_from_source) {
                            if (card) card.classList.add('exists');
                            el.textContent = 'Already in DB';
                            toast('Serie created successfully', 'success');
                          } else {
                            el.disabled = false;
                            el.textContent = prevText;
                            toast('Serie creation failed', 'error');
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
                      const q = `query($sid: String!, $page: Int!, $f: GraphQLFetchSearchSerieFilter!) {
                        source_search_series(source_id: $sid, page: $page, filters: $f) {
                          has_next_page
                          series {
                            id
                            title { en jp jp_ro fr ko zh_hk zh }
                            cover
                          }
                        }
                      }`;
                      // Build filters object from UI
                      const filters = { query };
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

                      const data = await GQL(q, { sid: sourceId, page, f: filters });
                      const result = data.source_search_series;
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
                          const q2 = `query($sid: String!, $ids: [String!]!) { existing_series_for_source(source_id: $sid, external_ids: $ids) { external_id serie_id } }`;
                          const r2 = await GQL(q2, { sid: sourceId, ids });
                          const existing = new Set(r2.existing_series_for_source.map(x => x.external_id));
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
                      const q = `query($page: Int!, $per: Int!) { series_list(page: $page, per_page: $per) { has_next_page series { id cover title { en fr jp jp_ro ko zh_hk zh } } } }`;
                      const data = await GQL(q, { page, per: 24 });
                      const result = data.series_list;
                      hasNextPageAdded = result.has_next_page;
                      elPage.textContent = String(page);
                      elPrev.disabled = page <= 1;
                      elNext.disabled = !hasNextPageAdded;
                      const series = result.series;
                      elResults.innerHTML = series.map(s => `
                        <div class="card" data-serie="${s.id}">
                          <img class="cover" src="${s.cover}" alt="cover" />
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

async fn graphql_proxy(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request<Body>,
) -> impl IntoResponse {
    let target = format!("{}/graphql", state.api_base.trim_end_matches('/'));

    // Copy JSON body
    let bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let mut builder = state
        .http
        .post(&target)
        .header("content-type", "application/json");
    if let Some(auth) = headers.get(axum::http::header::AUTHORIZATION) {
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
                // filter hop-by-hop headers implicitly
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

// Minimal callback page to store JWT from `?token=` and redirect
async fn auth_callback(_uri: Uri) -> impl IntoResponse {
    let page = html! {
        (DOCTYPE)
        html {
            head { meta charset="utf-8"; meta name="viewport" content="width=device-width, initial-scale=1"; title { "Auth Callback" } }
            body style="font-family:system-ui;padding:1rem" {
                script { (PreEscaped(r#"
                    (function(){
                        var u = new URL(window.location.href);
                        var token = u.searchParams.get('token');
                        if (token) { try { localStorage.setItem('jwt', token); } catch(e) {} }
                        var to = sessionStorage.getItem('post_auth_redirect') || '/';
                        sessionStorage.removeItem('post_auth_redirect');
                        window.location.replace(to);
                    })();
                "#)) }
                p { "Finishing authentication…" }
            }
        }
    };
    Html(page.into_string())
}
