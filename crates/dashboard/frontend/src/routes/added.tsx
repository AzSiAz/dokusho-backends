import * as React from 'react'
import { useQuery } from '@tanstack/react-query'
import { authHeaders } from '../lib/auth'
export default Added

type Serie = { id: string; title?: any; cover?: string }
type List = { series: Serie[]; has_next_page: boolean }

function Added() {
  const [page, setPage] = React.useState<number>(1)
  const listQ = useQuery({
    queryKey: ['added', page],
    queryFn: async (): Promise<List> => {
      const r = await fetch(`/api/v1/admin/series?page=${page}&per_page=24`, { headers: authHeaders() })
      if (!r.ok) throw new Error('Failed to load')
      return r.json()
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
    for (const k of pref) { const r = pick(ml[k]); if (r) return r }
    const vals = Object.values(ml).map(pick).filter(Boolean) as string[]
    return vals[0] || fallback
  }

  if (listQ.isLoading) return <div>Loading…</div>
  if (listQ.error) return <div>Failed to load</div>
  return (
    <div>
      <section id="controls-pager" className="row" style={{ marginTop: 8 }}>
        <button id="prev" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</button>
        <span>
          Page: <span id="page">{page}</span>
        </span>
        <button id="next" disabled={!listQ.data?.has_next_page} onClick={() => setPage((p) => p + 1)}>Next</button>
      </section>
      <div id="results" className="grid">
        {listQ.data?.series?.map((s) => (
          <div key={s.id} className="card" data-serie={s.id}>
            <img className="cover" src={`/img?u=${encodeURIComponent(s.cover ?? '')}`} alt="cover" />
            <h3>{pickTitle(s.title, s.id)}</h3>
          </div>
        ))}
      </div>
    </div>
  )
}
