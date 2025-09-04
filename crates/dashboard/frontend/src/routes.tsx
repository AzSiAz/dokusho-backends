import * as React from 'react'
import { Outlet, Link, useRouterState, createRootRoute, createRoute, useNavigate } from '@tanstack/react-router'
import Sources from './routes/sources'
import Added from './routes/added'
import { authHeaders, completeLoginIfNeeded, fetchOIDC, getToken, startLogin } from './lib/auth'

function Root() {
  const [loggedIn, setLoggedIn] = React.useState<boolean>(!!getToken())

  React.useEffect(() => {
    completeLoginIfNeeded()
      .then(async () => {
        try {
          const r = await fetch('/api/v1/users/me', { headers: authHeaders() })
          setLoggedIn(r.ok)
        } catch {
          setLoggedIn(false)
        }
      })
      .catch(() => {})
  }, [])

  const onSignIn = async () => {
    const cfg = await fetchOIDC()
    await startLogin(cfg)
  }
  const onSignOut = async () => {
    try {
      localStorage.removeItem('jwt')
    } catch {}
    setLoggedIn(false)
  }

  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const isSources = pathname.startsWith('/sources') || pathname === '/'
  const isAdded = pathname.startsWith('/added')
  return (
    <div>
      <header>
        <strong>Adminboard</strong>
        <Link to="/" className={`tab ${isSources ? 'active' : ''}`}>Sources</Link>
        <Link to="/added" className={`tab ${isAdded ? 'active' : ''}`}>Added</Link>
        <span className="topbar-spacer" />
        {!loggedIn ? <button onClick={onSignIn}>Sign in</button> : <button onClick={onSignOut}>Sign out</button>}
      </header>
      <main>
        <Outlet />
      </main>
    </div>
  )
}

export const rootRoute = createRootRoute({ component: Root })

export const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: Sources,
})

export const sourcesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sources',
  component: Sources,
})

export const addedRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/added',
  component: Added,
})

function Callback() {
  const navigate = useNavigate()
  React.useEffect(() => {
    (async () => {
      try {
        await completeLoginIfNeeded()
      } catch (e) {
        // optional: report error
      } finally {
        const dest = sessionStorage.getItem('post_auth_redirect') || '/'
        navigate({ to: dest, replace: true })
      }
    })()
  }, [navigate])
  return <div>Completing sign-in…</div>
}

export const callbackRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/auth/callback',
  component: Callback,
})

export const routeTree = rootRoute.addChildren([indexRoute, sourcesRoute, addedRoute, callbackRoute])
