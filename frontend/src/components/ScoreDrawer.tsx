import { useState } from 'react'
import type { RoomSnapshot } from '../types'

interface Props {
  room: RoomSnapshot
}

export function ScoreDrawer({ room }: Props) {
  const [open, setOpen] = useState(false)
  const sorted = [...room.players].sort((a, b) => b.score - a.score)

  return (
    <>
      <button
        className="fixed bottom-4 right-4 z-40 bg-slate-700 text-white text-xs px-3 py-2 rounded-full shadow"
        onClick={() => setOpen(true)}
      >
        Scores
      </button>

      {open && (
        <div className="fixed inset-0 z-50 flex flex-col bg-slate-900/95">
          <div className="flex items-center justify-between px-4 py-4 border-b border-slate-700">
            <h2 className="text-lg font-bold">Scoreboard</h2>
            <button className="text-slate-400 text-2xl leading-none" onClick={() => setOpen(false)}>
              ✕
            </button>
          </div>

          <div className="overflow-y-auto flex-1 px-4 py-3 space-y-2">
            {sorted.map((p) => (
              <div key={p.name} className="flex justify-between items-center bg-slate-800 rounded-lg px-4 py-2">
                <span className={p.connected ? 'text-white' : 'text-slate-500 line-through'}>
                  {p.name}
                </span>
                <span className="font-bold text-yellow-400">{p.score} pts</span>
              </div>
            ))}

            {room.log.length > 0 && (
              <>
                <h3 className="text-slate-400 text-xs uppercase tracking-wide pt-4">Round Log</h3>
                {[...room.log].reverse().map((entry, i) => (
                  <div key={i} className="bg-slate-800 rounded-lg px-4 py-3 text-sm space-y-1">
                    <div className="font-semibold">{entry.topic}</div>
                    <div className="text-slate-400">
                      Owner: <span className="text-white">{entry.owner_name}</span>
                    </div>
                    <div className="text-slate-400">
                      {entry.guesser_name} guessed{' '}
                      <span className={entry.correct ? 'text-green-400' : 'text-red-400'}>
                        {entry.guessed_name}
                      </span>
                      {' '}— {entry.correct ? `+${entry.points} pts` : `everyone else +${entry.points}`}
                    </div>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>
      )}
    </>
  )
}
