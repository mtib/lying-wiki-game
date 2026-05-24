import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { submitGuess } from '../api'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function RoundActiveScreen({ room, token, myName, onError }: Props) {
  const [loading, setLoading] = useState(false)
  const isGuesser = room.guesser_name === myName

  const handleGuess = async (guessedName: string) => {
    setLoading(true)
    try {
      await submitGuess(room.code, token, guessedName)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setLoading(false)
    }
  }

  // Other players to guess from (not the guesser themselves)
  const guessTargets = room.players.filter((p) => p.name !== room.guesser_name)

  return (
    <div className="min-h-full flex flex-col px-6 py-8 gap-6">
      <div className="text-center space-y-2">
        <p className="text-slate-400 text-sm uppercase tracking-wide">Round {room.round_number} — The topic is</p>
        <h2 className="text-3xl font-black text-white leading-tight">{room.current_topic}</h2>
      </div>

      <div className="bg-slate-800 rounded-xl px-4 py-3 text-center">
        <p className="text-slate-400 text-sm">Guesser</p>
        <p className="text-xl font-bold text-yellow-400">
          {room.guesser_name}{isGuesser ? ' (you)' : ''}
        </p>
      </div>

      {isGuesser ? (
        <div className="space-y-3">
          <p className="text-slate-400 text-sm">Whose topic is it?</p>
          {guessTargets.map((p) => (
            <button
              key={p.name}
              disabled={loading}
              onClick={() => handleGuess(p.name)}
              className="w-full bg-slate-700 hover:bg-slate-600 text-white font-semibold py-4 rounded-xl text-base disabled:opacity-50 flex items-center justify-between px-5"
            >
              <span>{p.name}</span>
              <span className="text-slate-400">→</span>
            </button>
          ))}
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-slate-500 text-center px-8">
          <p>Waiting for <span className="text-white font-semibold">{room.guesser_name}</span> to guess…</p>
        </div>
      )}

      <ScoreDrawer room={room} />
    </div>
  )
}
