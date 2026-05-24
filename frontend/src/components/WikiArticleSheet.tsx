import { useEffect, useRef, useState } from 'react'
import type { WikiArticle } from '../types'
import { fetchRandomWikiArticle } from '../api'

interface Props {
  onSelect: (title: string) => void
  onClose: () => void
}

export function WikiArticleSheet({ onSelect, onClose }: Props) {
  const [article, setArticle] = useState<WikiArticle | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const load = () => {
    setLoading(true)
    setError(null)
    setArticle(null)
    fetchRandomWikiArticle()
      .then(setArticle)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }

  useEffect(() => { load() }, [])

  useEffect(() => {
    if (article && iframeRef.current) {
      const doc = iframeRef.current.contentDocument
      if (doc) {
        doc.open()
        doc.write(article.html)
        doc.close()
      }
    }
  }, [article])

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-white text-slate-900">
      <div className="flex items-center justify-between px-4 py-3 bg-slate-100 border-b border-slate-200 shrink-0">
        <button className="text-blue-600 text-sm font-medium" onClick={onClose}>
          Cancel
        </button>
        <span className="text-sm font-semibold truncate max-w-[55vw]">
          {article?.title ?? 'Loading…'}
        </span>
        <button className="text-sm text-slate-500 font-medium" onClick={load}>
          🎲 New
        </button>
      </div>

      <div className="flex-1 overflow-hidden">
        {loading && (
          <div className="flex items-center justify-center h-full text-slate-500">
            Loading article…
          </div>
        )}
        {error && (
          <div className="flex flex-col items-center justify-center h-full gap-4 text-red-500 px-6 text-center">
            <p>{error}</p>
            <button className="text-blue-600 underline" onClick={load}>Retry</button>
          </div>
        )}
        {article && !loading && (
          <iframe
            ref={iframeRef}
            className="w-full h-full border-none"
            sandbox="allow-same-origin"
            title="Wikipedia article"
          />
        )}
      </div>

      {article && (
        <div className="px-4 py-4 bg-slate-100 border-t border-slate-200 shrink-0">
          <button
            className="w-full bg-blue-600 text-white font-semibold py-3 rounded-xl text-base"
            onClick={() => onSelect(article.title)}
          >
            Use "{article.title}"
          </button>
        </div>
      )}
    </div>
  )
}
