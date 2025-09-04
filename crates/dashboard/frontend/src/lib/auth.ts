export type OIDCConfig = {
  authorization_endpoint: string
  token_endpoint: string
  client_id: string
  redirect_uri: string
}

export async function fetchOIDC(): Promise<OIDCConfig> {
  const r = await fetch('/dashboard/oidc.json')
  if (!r.ok) throw new Error('Failed to fetch OIDC config')
  return r.json()
}

export function getToken(): string | null {
  try {
    return localStorage.getItem('jwt')
  } catch {
    return null
  }
}

export function authHeaders(): Record<string, string> {
  const t = getToken()
  return t ? { authorization: 'Bearer ' + t } : {}
}

export async function sha256base64url(input: string): Promise<string> {
  const data = new TextEncoder().encode(input)
  const hash = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(hash)
  let str = ''
  for (let i = 0; i < bytes.byteLength; i++) str += String.fromCharCode(bytes[i])
  return btoa(str).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

export function randomString(len = 64): string {
  const arr = new Uint8Array(len)
  crypto.getRandomValues(arr)
  return Array.from(arr)
    .map((b) => ('0' + b.toString(16)).slice(-2))
    .join('')
}

export async function startLogin(oidc?: OIDCConfig) {
  const cfg = oidc ?? (await fetchOIDC())
  const code_verifier = randomString(64)
  const code_challenge = await sha256base64url(code_verifier)
  const state = randomString(32)
  sessionStorage.setItem('pkce_verifier', code_verifier)
  sessionStorage.setItem('oauth_state', state)
  sessionStorage.setItem('post_auth_redirect', '/')
  const url = new URL(cfg.authorization_endpoint)
  url.searchParams.set('response_type', 'code')
  url.searchParams.set('client_id', cfg.client_id)
  url.searchParams.set('redirect_uri', cfg.redirect_uri)
  url.searchParams.set('scope', 'openid profile email groups')
  url.searchParams.set('code_challenge_method', 'S256')
  url.searchParams.set('code_challenge', code_challenge)
  url.searchParams.set('state', state)
  window.location.assign(url.toString())
}

export async function completeLoginIfNeeded(oidc?: OIDCConfig) {
  const cfg = oidc ?? (await fetchOIDC())
  const u = new URL(window.location.href)
  const code = u.searchParams.get('code')
  const state = u.searchParams.get('state')
  if (!code) return
  const saved = sessionStorage.getItem('oauth_state')
  const verifier = sessionStorage.getItem('pkce_verifier')
  if (!saved || !verifier || saved !== state) throw new Error('Invalid state')
  const body = new URLSearchParams()
  body.set('grant_type', 'authorization_code')
  body.set('client_id', cfg.client_id)
  body.set('code', code)
  body.set('code_verifier', verifier)
  body.set('redirect_uri', cfg.redirect_uri)
  const r = await fetch(cfg.token_endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body,
  })
  const j = await r.json()
  if (!r.ok || !j.access_token) throw new Error('Token exchange failed')
  localStorage.setItem('jwt', j.access_token)
  sessionStorage.removeItem('oauth_state')
  sessionStorage.removeItem('pkce_verifier')
  u.searchParams.delete('code')
  u.searchParams.delete('state')
  history.replaceState(null, '', u.pathname + (u.search ? '?' + u.search : '') + u.hash)
}

