use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use maud::{PreEscaped, html};
mod components;
use components::{layout, page_container, topbar};
use openidconnect::IssuerUrl;
use openidconnect::core::CoreProviderMetadata;
use reqwest::Client;
use serde::Deserialize;

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
        .route("/", get(index))
        .route("/auth/callback", get(index))
        .route("/img", get(image_proxy))
        .with_state(state);

    Ok(app)
}

async fn index(State(state): State<DashboardState>) -> impl IntoResponse {
    let oidc_str = serde_json::json!({
        "authorization_endpoint": state.authorization_endpoint,
        "token_endpoint": state.token_endpoint,
        "client_id": state.client_id,
        "redirect_uri": state.redirect_url,
    })
    .to_string();

    layout(
        "Dokusho Adminboard",
        html! {
            (topbar())
            (page_container(html! {
                section id="controls-sources" class="flex gap-2 flex-wrap items-center" {
                    label class="text-slate-300" { "Source" }
                    select id="source" class="px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" {}
                    input id="query" placeholder="Search query" class="px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" {}
                    button id="search" class="px-3 py-2 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors" { "Search" }
                }
                div id="filters" class="mt-2" {}
                section id="controls-pager" class="flex gap-2 flex-wrap items-center mt-2" {
                    button id="prev" class="px-3 py-1.5 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors" { "Prev" }
                    span { "Page: " span id="page" { "1" } }
                    button id="next" class="px-3 py-1.5 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors" { "Next" }
                }
                div id="results" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3 mt-3" {}
                div id="toast-container" class="fixed right-4 bottom-4 flex flex-col gap-2 z-[9999]" {}
            }))
            script { (PreEscaped(format!("const OIDC = {}\n", oidc_str))) (PreEscaped(r#"
                  const qs = s => document.querySelector(s);
                  const getToken = () => { try { return localStorage.getItem('jwt'); } catch(_) { return null; } };
                  const authHeaders = () => { const t = getToken(); return t ? { 'authorization': 'Bearer ' + t } : {}; };
                  async function sha256base64url(input) { const data = new TextEncoder().encode(input); const hash = await crypto.subtle.digest('SHA-256', data); const bytes = new Uint8Array(hash); let str = ''; for (let i = 0; i < bytes.byteLength; i++) str += String.fromCharCode(bytes[i]); return btoa(str).replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,''); }
                  function randomString(len=64) { const arr = new Uint8Array(len); crypto.getRandomValues(arr); return Array.from(arr).map(b=>('0'+b.toString(16)).slice(-2)).join(''); }
                  async function startLogin() { const code_verifier = randomString(64); const code_challenge = await sha256base64url(code_verifier); const state = randomString(32); sessionStorage.setItem('pkce_verifier', code_verifier); sessionStorage.setItem('oauth_state', state); sessionStorage.setItem('post_auth_redirect', '/'); const url = new URL(OIDC.authorization_endpoint); url.searchParams.set('response_type','code'); url.searchParams.set('client_id', OIDC.client_id); url.searchParams.set('redirect_uri', OIDC.redirect_uri); url.searchParams.set('scope','openid profile email groups'); url.searchParams.set('code_challenge_method','S256'); url.searchParams.set('code_challenge', code_challenge); url.searchParams.set('state', state); window.location = url.toString(); }
                  async function completeLoginIfNeeded() { const u = new URL(window.location.href); const code = u.searchParams.get('code'); const state = u.searchParams.get('state'); if (!code) return; try { const saved = sessionStorage.getItem('oauth_state'); const verifier = sessionStorage.getItem('pkce_verifier'); if (!saved || !verifier || saved !== state) throw new Error('Invalid state'); const body = new URLSearchParams(); body.set('grant_type','authorization_code'); body.set('client_id', OIDC.client_id); body.set('code', code); body.set('code_verifier', verifier); body.set('redirect_uri', OIDC.redirect_uri); const r = await fetch(OIDC.token_endpoint, { method:'POST', headers: { 'content-type':'application/x-www-form-urlencoded' }, body }); const j = await r.json(); if (!r.ok || !j.access_token) throw new Error('Token exchange failed'); localStorage.setItem('jwt', j.access_token); sessionStorage.removeItem('oauth_state'); sessionStorage.removeItem('pkce_verifier'); u.searchParams.delete('code'); u.searchParams.delete('state'); history.replaceState(null,'', u.pathname + (u.search?('?'+u.search):'') + u.hash); } catch(e) { console.error(e); } }
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
                    const t = getToken();
                    if (!t) { elSignIn.classList.remove('hidden'); elSignOut.classList.add('hidden'); return; }
                    try { const r = await fetch('/api/v1/users/me', { headers: authHeaders() }); const ok = r.ok; elSignIn.classList.toggle('hidden', ok); elSignOut.classList.toggle('hidden', !ok); } catch(_) { elSignIn.classList.remove('hidden'); elSignOut.classList.add('hidden'); }
                  };

                  elSignIn.addEventListener('click', async () => { await startLogin(); });
                  elSignOut.addEventListener('click', async () => { try { localStorage.removeItem('jwt'); } catch(_) {} ; updateAuthButtons(); });

                  const loadSources = async () => {
                    elSource.innerHTML = '<option>Loading…</option>';
                    try {
                      const r = await fetch('/api/v1/sources', { headers: authHeaders() });
                      if (!r.ok) throw new Error('Failed to load sources');
                      const data = await r.json();
                      elSource.innerHTML = data.map(s => `<option value="${s.id}">${s.name}</option>`).join('');
                      sourcesMeta = Object.fromEntries(data.map(s => [s.id, s]));
                      renderFilters();
                    } catch (e) {
                      elSource.innerHTML = '';
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
                      <div class="rounded-xl border border-slate-700 p-3 bg-slate-900">
                        <div class="text-sm text-slate-400 mb-1">${title}</div>
                        ${body}
                      </div>`;
                    let parts = [];
                    const orders = toArr(f.order);
                    if (orders.length) parts.push(card('Order', `<select id="f-order" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100"><option value="">(any)</option>${orders.map(opt).join('')}</select>`));
                    const sorts = toArr(f.sort);
                    if (sorts.length) parts.push(card('Sort', `<select id="f-sort" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100"><option value="">(any)</option>${sorts.map(opt).join('')}</select>`));
                    if (f.artists) parts.push(card('Artists', `<input id="f-artists" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" placeholder="Comma separated" />`));
                    if (f.authors) parts.push(card('Authors', `<input id="f-authors" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" placeholder="Comma separated" />`));
                    const types = toArr(f.types);
                    if (types.length) {
                      const chips = types.map(v => `<label class='inline-flex items-center'><input class='peer hidden' type='checkbox' value='${v}'/><span class="px-2.5 py-1.5 rounded-full border border-slate-700 bg-slate-900 text-slate-100 text-sm peer-checked:bg-slate-700 peer-checked:border-slate-600">${v}</span></label>`).join('');
                      parts.push(card('Types', `<div id="f-types" class="flex flex-wrap gap-1.5">${chips}</div>`));
                    }
                    const status = toArr(f.status);
                    if (status.length) {
                      const chips = status.map(v => `<label class='inline-flex items-center'><input class='peer hidden' type='checkbox' value='${v}'/><span class="px-2.5 py-1.5 rounded-full border border-slate-700 bg-slate-900 text-slate-100 text-sm peer-checked:bg-slate-700 peer-checked:border-slate-600">${v}</span></label>`).join('');
                      parts.push(card('Status', `<div id="f-status" class="flex flex-wrap gap-1.5">${chips}</div>`));
                    }
                    if (f.genres && f.genres.accepted_values) {
                      const opts = toArr(f.genres.accepted_values).map(opt).join('');
                      if (f.genres.include) parts.push(card('Genres: Include', `<select id="f-genres-inc" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" multiple size="8">${opts}</select>`));
                      if (f.genres.exclude) parts.push(card('Genres: Exclude', `<select id="f-genres-exc" class="w-full px-3 py-2 rounded-md border border-slate-700 bg-slate-900 text-slate-100" multiple size="8">${opts}</select>`));
                    }
                    elFilters.innerHTML = `<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">${parts.join('')}</div>`;
                  };

                  const pickTitle = (ml, fallback='') => {
                    if (!ml || typeof ml !== 'object') return fallback;
                    const pref = ['En','Fr','Jp','JpRo','Ko','ZhHk','Zh'];
                    const pick = (v) => { if (!v) return undefined; if (typeof v === 'string') return v; if (Array.isArray(v) && v.length) return v[0]; };
                    for (const k of pref) { const r = pick(ml[k]); if (r) return r; }
                    const vals = Object.values(ml).map(pick).filter(Boolean);
                    return vals[0] || fallback;
                  };

                  const card = (sourceId, s) => `
                    <div class="card rounded-xl border border-slate-700 p-3 bg-slate-900" data-serie="${s.id}">
                      <img class="cover w-full h-80 object-cover rounded-md bg-slate-800" src="/img?u=${encodeURIComponent(s.cover)}" alt="cover" />
                      <h3 class="mt-2 font-medium">${pickTitle(s.title, s.id)}</h3>
                      <div class="mt-2 flex gap-2">
                        <button class="create px-3 py-1.5 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors" data-source="${sourceId}" data-serie="${s.id}">Create serie</button>
                      </div>
                    </div>
                  `;

                  const bindCreateButtons = () => {
                    document.querySelectorAll('button.create').forEach(btn => {
                      if (btn.__bound) return; btn.__bound = true;
                      btn.addEventListener('click', async (ev) => {
                        const el = ev.currentTarget; const sourceId = el.getAttribute('data-source'); const serieId = el.getAttribute('data-serie'); const card = el.closest('.card'); const prevText = el.textContent; el.disabled = true; el.textContent = 'Creating…';
                        try {
                          const r = await fetch('/api/v1/admin/series/from-source', { method: 'POST', headers: { 'content-type': 'application/json', ...authHeaders() }, body: JSON.stringify({ source_id: sourceId, serie_id: serieId }) });
                          if (r.ok) { if (card) { card.classList.add('exists'); const cover = card.querySelector('.cover'); if (cover) cover.classList.add('opacity-40','grayscale'); } el.textContent = 'Already in DB'; }
                          else { el.disabled = false; el.textContent = prevText; }
                        } catch (e) { el.disabled = false; el.textContent = prevText; }
                      });
                    });
                  };

                  const search = async (page = 1) => {
                    const sourceId = elSource.value; const query = elQuery.value || ''; elResults.innerHTML = '';
                    try {
                      const filters = { query, page };
                      const readCSV = (id) => { const el = document.getElementById(id); if (!el) return undefined; const v = el.value.trim(); if (!v) return undefined; return v.split(',').map(s => s.trim()).filter(Boolean); };
                      const readSelect = (id) => { const el = document.getElementById(id); if (!el) return undefined; const v = el.value; if (!v) return undefined; return v; };
                      const readChecks = (id) => { const wrap = document.getElementById(id); if (!wrap) return undefined; const vals = Array.from(wrap.querySelectorAll('input[type="checkbox"]:checked')).map(i => i.value); return vals.length ? vals : undefined; };
                      const readMulti = (id) => { const el = document.getElementById(id); if (!el) return undefined; const vals = Array.from(el.selectedOptions).map(o => o.value); return vals.length ? vals : undefined; };
                      const meta = sourcesMeta[sourceId]?.filters || {};
                      if (meta.order) { const v = readSelect('f-order'); if (v) filters.order = v; }
                      if (meta.sort) { const v = readSelect('f-sort'); if (v) filters.sort = v; }
                      if (meta.artists) { const v = readCSV('f-artists'); if (v) filters.artists = v; }
                      if (meta.authors) { const v = readCSV('f-authors'); if (v) filters.authors = v; }
                      if (meta.types) { const v = readChecks('f-types'); if (v) filters.types = v; }
                      if (meta.status) { const v = readChecks('f-status'); if (v) filters.status = v; }
                      if (meta.genres) { const includes = readMulti('f-genres-inc'); const excludes = readMulti('f-genres-exc'); if ((includes && includes.length) || (excludes && excludes.length)) { filters.genres = { includes, excludes }; } }

                      const r = await fetch(`/api/v1/sources/${encodeURIComponent(sourceId)}/series/search`, { method: 'POST', headers: { 'content-type': 'application/json', ...authHeaders() }, body: JSON.stringify(filters) });
                      if (!r.ok) throw new Error('Search failed');
                      const result = await r.json();
                      hasNextPage = result.has_next_page; elPage.textContent = String(page); elPrev.disabled = page <= 1; elNext.disabled = !hasNextPage; const series = result.series; elResults.innerHTML = series.map(s => card(sourceId, s)).join('');

                      const ids = series.map(s => s.id);
                      if (ids.length) {
                        try { const r2 = await fetch('/api/v1/admin/series/existing', { method: 'POST', headers: { 'content-type': 'application/json', ...authHeaders() }, body: JSON.stringify({ source_id: sourceId, external_ids: ids }) }); const arr = r2.ok ? await r2.json() : []; const existing = new Set(arr.map(x => x.external_id)); document.querySelectorAll('#results .card').forEach(c => { const id = c.getAttribute('data-serie'); const btn = c.querySelector('button.create'); if (existing.has(id)) { c.classList.add('exists'); const cover = c.querySelector('.cover'); if (cover) cover.classList.add('opacity-40','grayscale'); if (btn) { btn.disabled = true; btn.textContent = 'Already in DB'; } } }); } catch (_) {}
                      }

                      bindCreateButtons();
                    } catch (_) {}
                  };

                  const loadAdded = async (page = 1) => {
                    elResults.innerHTML = '';
                    try { const r = await fetch(`/api/v1/admin/series?page=${page}&per_page=24`, { headers: authHeaders() }); if (!r.ok) throw new Error('Failed to load'); const result = await r.json(); hasNextPageAdded = result.has_next_page; elPage.textContent = String(page); elPrev.disabled = page <= 1; elNext.disabled = !hasNextPageAdded; const series = result.series; elResults.innerHTML = series.map(s => `<div class=\"card rounded-xl border border-slate-700 p-3 bg-slate-900\" data-serie=\"${s.id}\"><img class=\"cover w-full h-80 object-cover rounded-md bg-slate-800\" src=\"/img?u=${encodeURIComponent(s.cover)}\" alt=\"cover\" /><h3 class=\"mt-2 font-medium\">${pickTitle(s.title, s.id)}</h3></div>`).join(''); } catch (_) {}
                  };

                  document.querySelector('#search').addEventListener('click', () => { currentPage = 1; search(currentPage); });
                  elPrev.addEventListener('click', () => { if (mode === 'sources') { if (currentPage > 1) { currentPage -= 1; search(currentPage); } } else { if (currentPageAdded > 1) { currentPageAdded -= 1; loadAdded(currentPageAdded); } } });
                  elNext.addEventListener('click', () => { if (mode === 'sources') { if (hasNextPage) { currentPage += 1; search(currentPage); } } else { if (hasNextPageAdded) { currentPageAdded += 1; loadAdded(currentPageAdded); } } });

                  const switchMode = (m) => {
                    mode = m;
                    const srcActive = m === 'sources';
                    const addActive = m === 'added';
                    // Toggle visual styles for tabs using Tailwind classes
                    elTabSources.classList.toggle('bg-slate-700', srcActive);
                    elTabSources.classList.toggle('bg-slate-700/20', !srcActive);
                    elTabAdded.classList.toggle('bg-slate-700', addActive);
                    elTabAdded.classList.toggle('bg-slate-700/20', !addActive);

                    elControlsSources.style.display = srcActive ? '' : 'none';
                    elFilters.style.display = srcActive ? '' : 'none';
                    elControlsPager.style.display = '';
                    if (srcActive) { elPage.textContent = String(currentPage); elPrev.disabled = currentPage <= 1; elNext.disabled = !hasNextPage; search(currentPage); }
                    else { elPage.textContent = String(currentPageAdded); elPrev.disabled = currentPageAdded <= 1; elNext.disabled = !hasNextPageAdded; loadAdded(currentPageAdded); }
                  };

                  elTabSources.addEventListener('click', () => switchMode('sources'));
                  elTabAdded.addEventListener('click', () => switchMode('added'));
                  completeLoginIfNeeded().then(updateAuthButtons);
                  elSource.addEventListener('change', () => { renderFilters(); });
                  loadSources().then(() => search(currentPage)).catch(() => {});
            "#)) }
        },
    )
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
