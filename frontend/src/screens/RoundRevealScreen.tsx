import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitTopic } from '../api'
import { WikiArticleSheet } from '../components/WikiArticleSheet'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
  onLeave: () => void
}

export function RoundRevealScreen({ room, token, myName, onError, onLeave: _onLeave }: Props) {
  const [title, setTitle] = useState('')
  const [showWiki, setShowWiki] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  if (!room.reveal) return null
  const reveal = room.reveal
  const isOwner = reveal.owner_name === myName

  const handleSubmit = async () => {
    if (!title.trim()) return
    setSubmitting(true)
    try {
      await submitTopic(room.code, token, title.trim())
      setTitle('')
    } catch (e: any) {
      onError(e.message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <>
      {showWiki && (
        <WikiArticleSheet
          onSelect={(t) => { setTitle(t); setShowWiki(false) }}
          onClose={() => setShowWiki(false)}
        />
      )}

      <div className="min-h-full flex flex-col px-6 py-8 gap-6">
        <div className="text-center space-y-1">
          <p className="text-slate-400 text-sm uppercase tracking-wide">Round {room.round_number} Reveal</p>
          <h2 className="text-2xl font-black text-white">{reveal.topic}</h2>
          <p className="text-slate-300">
            owned by <span className="text-yellow-400 font-semibold">{reveal.owner_name}</span>
          </p>
        </div>

        <div className={`rounded-xl px-5 py-4 text-center ${reveal.correct ? 'bg-green-900/60' : 'bg-red-900/60'}`}>
          <p className="text-lg font-bold text-white">
            {reveal.correct ? '✓ Correct guess!' : '✗ Wrong guess!'}
          </p>
          <p className="text-slate-300 text-sm mt-1">
            {reveal.guesser_name} guessed <span className="text-white font-medium">{reveal.guessed_name}</span>
          </p>
          <p className="text-slate-300 text-sm">
            {reveal.correct
              ? `${reveal.guesser_name} gets ${reveal.points} points`
              : `Everyone else gets ${reveal.points} point each`}
          </p>
        </div>

        <div className="bg-slate-800 rounded-xl px-4 py-3 space-y-2">
          <p className="text-slate-400 text-xs uppercase tracking-wide">Scores</p>
          {[...room.players].sort((a, b) => b.score - a.score).map((p) => (
            <div key={p.name} className="flex justify-between">
              <span className="text-white">{p.name}</span>
              <span className="text-yellow-400 font-semibold">{p.score} pts</span>
            </div>
          ))}
        </div>

        {isOwner ? (
          <div className="space-y-3 mt-auto">
            <p className="text-white font-semibold">Your topic was revealed. Submit a new one to continue:</p>
            <input
              className="w-full bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Wikipedia article title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
            <div className="flex gap-3">
              <button
                className="flex-1 bg-slate-700 text-white font-medium py-2.5 rounded-xl text-sm"
                onClick={() => setShowWiki(true)}
              >
                🎲 Random
              </button>
              <button
                className="flex-1 bg-blue-600 text-white font-semibold py-2.5 rounded-xl text-sm disabled:opacity-50"
                disabled={!title.trim() || submitting}
                onClick={handleSubmit}
              >
                Submit & Continue
              </button>
            </div>
          </div>
        ) : (
          <p className="text-slate-500 text-center mt-auto">
            Waiting for <span className="text-white font-medium">{reveal.owner_name}</span> to submit a new topic…
          </p>
        )}

        <ScoreDrawer room={room} />
      </div>
    </>
  )
}
