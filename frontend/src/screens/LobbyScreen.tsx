import { useState } from 'react'
import type { RoomSnapshot } from '../types'
import { startGame } from '../api'
import { ScoreDrawer } from '../components/ScoreDrawer'

interface Props {
  room: RoomSnapshot
  token: string
  myName: string
  onError: (msg: string) => void
}

export function LobbyScreen({ room, token, myName, onError }: Props) {
  const [loading, setLoading] = useState(false)

  const handleStart = async () => {
    setLoading(true)
    try {
      await startGame(room.code, token)
    } catch (e: any) {
      onError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-full flex flex-col px-6 py-8 gap-6">
      <div className="text-center">
        <p className="text-slate-400 text-sm mb-1">Room code</p>
        <p className="text-5xl font-bold tracking-widest text-white">{room.code}</p>
        <p className="text-slate-500 text-xs mt-2">Share this with your friends</p>
      </div>

      <div className="flex-1 space-y-2">
        <p className="text-slate-400 text-sm">Players ({room.players.length})</p>
        {room.players.map((p) => (
          <div
            key={p.name}
            className="flex items-center gap-3 bg-slate-800 rounded-xl px-4 py-3"
          >
            <span
              className={`w-2 h-2 rounded-full shrink-0 ${p.connected ? 'bg-green-400' : 'bg-slate-600'}`}
            />
            <span className="text-white">{p.name}{p.name === myName ? ' (you)' : ''}</span>
          </div>
        ))}
      </div>

      <button
        className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50 w-full"
        disabled={loading || room.players.length < 3}
        onClick={handleStart}
      >
        {room.players.length < 3
          ? `Need ${3 - room.players.length} more player(s)`
          : 'Start Game'}
      </button>

      <ScoreDrawer room={room} />
    </div>
  )
}
