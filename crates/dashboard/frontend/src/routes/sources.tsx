import * as React from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
export default Sources
import { authHeaders } from '../lib/auth'

type SourceMeta = { id: string; name: string; filters?: any }
type Serie = { id: string; title?: any; cover?: string }
type SearchResult = { series: Serie[]; has_next_page: boolean }

function Sources() {
  const qc = useQueryClient()
  const [sourceId, setSourceId] = React.useState<string>('')
  const [query, setQuery] = React.useState<string>('')
  const [page, setPage] = React.useState<number>(1)
  const [criteriaKey, setCriteriaKey] = React.useState<number>(0)

  const sourcesQ = useQuery({
    queryKey: ['sources'],
    queryFn: async (): Promise<SourceMeta[]> => {
      const r = await fetch('/api/v1/sources', { headers: authHeaders() })
      if (!r.ok) throw new Error('Failed to load sources')
      return r.json()
    },
  })

  React.useEffect(() => {
    if (sourcesQ.data && sourcesQ.data.length && !sourceId) {
      setSourceId(sourcesQ.data[0].id)
    }
  }, [sourcesQ.data, sourceId])

  const filtersMeta = React.useMemo(() => {
    return sourcesQ.data?.find((s) => s.id === sourceId)?.filters ?? {}
  }, [sourcesQ.data, sourceId])

  // Controlled filter state
  const [order, setOrder] = React.useState<string>('')
  const [sort, setSort] = React.useState<string>('')
  const [artists, setArtists] = React.useState<string>('')
  const [authors, setAuthors] = React.useState<string>('')
  const [types, setTypes] = React.useState<string[]>([])
  const [status, setStatus] = React.useState<string[]>([])
  const [genresInc, setGenresInc] = React.useState<string[]>([])
  const [genresExc, setGenresExc] = React.useState<string[]>([])

  const toArr = (v: any): string[] => {
    if (!v) return []
    if (Array.isArray(v)) return v
    if (typeof v === 'object') return Object.values(v)
    return [String(v)]
  }

  const buildFilters = React.useCallback(() => {
    const obj: any = { query, page }
    if (filtersMeta.order && order) obj.order = order
    if (filtersMeta.sort && sort) obj.sort = sort
    if (filtersMeta.artists && artists.trim()) obj.artists = artists.split(',').map((s) => s.trim()).filter(Boolean)
    if (filtersMeta.authors && authors.trim()) obj.authors = authors.split(',').map((s) => s.trim()).filter(Boolean)
    if (filtersMeta.types && types.length) obj.types = types
    if (filtersMeta.status && status.length) obj.status = status
    if (filtersMeta.genres && (genresInc.length || genresExc.length)) obj.genres = { includes: genresInc, excludes: genresExc }
    return obj
  }, [query, page, filtersMeta, order, sort, artists, authors, types, status, genresInc, genresExc])

  const searchQ = useQuery({
    queryKey: ['search', sourceId, criteriaKey, page],
    enabled: !!sourceId,
    queryFn: async (): Promise<SearchResult> => {
      const body = buildFilters()
      const r = await fetch(`/api/v1/sources/${encodeURIComponent(sourceId)}/series/search`, {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...authHeaders() },
        body: JSON.stringify(body),
      })
      if (!r.ok) throw new Error('Search failed')
      return r.json()
    },
  })

  const ids = React.useMemo(() => searchQ.data?.series?.map((s) => s.id) ?? [], [searchQ.data])
  const existingQ = useQuery({
    queryKey: ['existing', sourceId, ids],
    enabled: !!sourceId && ids.length > 0,
    queryFn: async (): Promise<Set<string>> => {
      const r = await fetch('/api/v1/admin/series/existing', {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...authHeaders() },
        body: JSON.stringify({ source_id: sourceId, external_ids: ids }),
      })
      if (!r.ok) return new Set<string>()
      const arr: Array<{ external_id: string }> = await r.json()
      return new Set(arr.map((x) => x.external_id))
    },
  })

  const createMutation = useMutation({
    mutationFn: async (serie_id: string) => {
      const r = await fetch('/api/v1/admin/series/from-source', {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...authHeaders() },
        body: JSON.stringify({ source_id: sourceId, serie_id }),
      })
      if (!r.ok) throw new Error('Create failed')
      return true
    },
    onSuccess: (_data, serie_id) => {
      // Optimistically mark as existing
      qc.setQueryData<Set<string>>(['existing', sourceId, ids], (prev) => new Set([...(prev ?? new Set()), serie_id]))
    },
  })

  const pickTitle = (ml: any, fallback = ''): string => {
    if (!ml || typeof ml !== 'object') return fallback
    const pref = ['En', 'Fr', 'Jp', 'JpRo', 'Ko', 'ZhHk', 'Zh']
    const pick = (v: any) => {
      if (!v) return undefined
      if (typeof v === 'string') return v
      if (Array.isArray(v) && v.length) return v[0]
      return undefined
    }
    for (const k of pref) {
      const r = pick(ml[k])
      if (r) return r
    }
    const vals = Object.values(ml).map(pick).filter(Boolean) as string[]
    return vals[0] || fallback
  }

  const onSearch = () => {
    setPage(1)
    setCriteriaKey((k) => k + 1)
  }

  const onToggle = (arr: string[], setArr: (v: string[]) => void, v: string) => {
    setArr(arr.includes(v) ? arr.filter((x) => x !== v) : [...arr, v])
  }

  if (sourcesQ.isLoading) return <div>Loading…</div>
  if (sourcesQ.error) return <div>Failed to load sources</div>
  const sources = sourcesQ.data ?? []
  const f = filtersMeta

  return (
    <div>
      <section id="controls-sources" className="row">
        <label>Source</label>
        <select id="source" value={sourceId} onChange={(e) => setSourceId(e.target.value)}>
          {sources.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
        <input id="query" placeholder="Search query" value={query} onChange={(e) => setQuery(e.target.value)} />
        <button id="search" onClick={onSearch}>
          Search
        </button>
      </section>
      <div id="filters">
        <div className="filters-grid">
          {!!toArr(f?.order).length && (
            <div className="filter-card">
              <div className="filter-title">Order</div>
              <select id="f-order" className="w-full" value={order} onChange={(e) => setOrder(e.target.value)}>
                <option value="">(any)</option>
                {toArr(f.order).map((v) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
            </div>
          )}
          {!!toArr(f?.sort).length && (
            <div className="filter-card">
              <div className="filter-title">Sort</div>
              <select id="f-sort" className="w-full" value={sort} onChange={(e) => setSort(e.target.value)}>
                <option value="">(any)</option>
                {toArr(f.sort).map((v) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
            </div>
          )}
          {f?.artists && (
            <div className="filter-card">
              <div className="filter-title">Artists</div>
              <input id="f-artists" className="w-full" placeholder="Comma separated" value={artists} onChange={(e) => setArtists(e.target.value)} />
            </div>
          )}
          {f?.authors && (
            <div className="filter-card">
              <div className="filter-title">Authors</div>
              <input id="f-authors" className="w-full" placeholder="Comma separated" value={authors} onChange={(e) => setAuthors(e.target.value)} />
            </div>
          )}
          {!!toArr(f?.types).length && (
            <div className="filter-card">
              <div className="filter-title">Types</div>
              <div id="f-types" className="chips">
                {toArr(f.types).map((v) => (
                  <label className="chip" key={v}>
                    <input type="checkbox" checked={types.includes(v)} onChange={() => onToggle(types, setTypes, v)} />
                    <span>{v}</span>
                  </label>
                ))}
              </div>
            </div>
          )}
          {!!toArr(f?.status).length && (
            <div className="filter-card">
              <div className="filter-title">Status</div>
              <div id="f-status" className="chips">
                {toArr(f.status).map((v) => (
                  <label className="chip" key={v}>
                    <input type="checkbox" checked={status.includes(v)} onChange={() => onToggle(status, setStatus, v)} />
                    <span>{v}</span>
                  </label>
                ))}
              </div>
            </div>
          )}
          {!!toArr(f?.genres?.accepted_values).length && (
            <>
              {f.genres?.include && (
                <div className="filter-card">
                  <div className="filter-title">Genres: Include</div>
                  <select id="f-genres-inc" multiple size={8} className="w-full" value={genresInc} onChange={(e) => setGenresInc(Array.from(e.target.selectedOptions).map((o) => o.value))}>
                    {toArr(f.genres.accepted_values).map((v) => (
                      <option key={v} value={v}>
                        {v}
                      </option>
                    ))}
                  </select>
                </div>
              )}
              {f.genres?.exclude && (
                <div className="filter-card">
                  <div className="filter-title">Genres: Exclude</div>
                  <select id="f-genres-exc" multiple size={8} className="w-full" value={genresExc} onChange={(e) => setGenresExc(Array.from(e.target.selectedOptions).map((o) => o.value))}>
                    {toArr(f.genres.accepted_values).map((v) => (
                      <option key={v} value={v}>
                        {v}
                      </option>
                    ))}
                  </select>
                </div>
              )}
            </>
          )}
        </div>
      </div>

      <section id="controls-pager" className="row" style={{ marginTop: 8 }}>
        <button id="prev" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>
          Prev
        </button>
        <span>
          Page: <span id="page">{page}</span>
        </span>
        <button id="next" disabled={!searchQ.data?.has_next_page} onClick={() => setPage((p) => p + 1)}>
          Next
        </button>
      </section>

      <div id="results" className="grid">
        {searchQ.isLoading && <div>Loading…</div>}
        {searchQ.error && <div>Search failed</div>}
        {searchQ.data?.series?.map((s) => {
          const exists = existingQ.data?.has(s.id) ?? false
          return (
            <div key={s.id} className={`card ${exists ? 'exists' : ''}`} data-serie={s.id}>
              <img className="cover" src={`/img?u=${encodeURIComponent(s.cover ?? '')}`} alt="cover" />
              <h3>{pickTitle(s.title, s.id)}</h3>
              <div className="row">
                <button className="create" disabled={exists || createMutation.isPending} onClick={() => createMutation.mutate(s.id)}>
                  {exists ? 'Already in DB' : createMutation.isPending ? 'Creating…' : 'Create serie'}
                </button>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
