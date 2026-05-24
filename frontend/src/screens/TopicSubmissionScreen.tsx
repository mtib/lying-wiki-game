import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitTopic, startRound } from '../api'
import { WikiArticleSheet } from '../components/WikiArticleSheet'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
  onLeave: () => void
}

export function TopicSubmissionScreen({ room, token, myName, onError, onLeave: _onLeave }: Props) {
  const [title, setTitle] = useState('')
  const [showWiki, setShowWiki] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [starting, setStarting] = useState(false)

  const allSubmitted = room.players.every((p) => p.submitted_this_round)

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

  const handleStart = async () => {
    setStarting(true)
    try {
      await startRound(room.code, token)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setStarting(false)
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
        <h2 className="text-xl font-bold text-white">Submit Your Topic</h2>

        <div className="space-y-3">
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
              🎲 Random Article
            </button>
            <button
              className="flex-1 bg-blue-600 text-white font-semibold py-2.5 rounded-xl text-sm disabled:opacity-50"
              disabled={!title.trim() || submitting}
              onClick={handleSubmit}
            >
              Submit
            </button>
          </div>
        </div>

        <div className="space-y-2">
          <p className="text-slate-400 text-sm">Players</p>
          {room.players.map((p) => (
            <div key={p.name} className="flex items-center justify-between bg-slate-800 rounded-xl px-4 py-3">
              <span className="text-white">{p.name}{p.name === myName ? ' (you)' : ''}</span>
              <span className={p.submitted_this_round ? 'text-green-400 text-lg' : 'text-slate-600 text-lg'}>
                {p.submitted_this_round ? '✓' : '…'}
              </span>
            </div>
          ))}
        </div>

        <button
          className="bg-green-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-40 w-full mt-auto"
          disabled={!allSubmitted || starting}
          onClick={handleStart}
        >
          {allSubmitted ? 'Start Round' : 'Waiting for everyone…'}
        </button>

        <ScoreDrawer room={room} />
      </div>
    </>
  )
}
