import { useEffect, useState } from 'react'
import type { RoomSnapshot } from '../types'

interface Props {
  room: RoomSnapshot
}

export function CountdownScreen({ room }: Props) {
  const [count, setCount] = useState(3)

  useEffect(() => {
    if (!room.countdown_started_at_ms) return
    const tick = () => {
      const elapsed = Date.now() - room.countdown_started_at_ms!
      const remaining = Math.max(0, 3 - Math.floor(elapsed / 1000))
      setCount(remaining)
    }
    tick()
    const id = setInterval(tick, 200)
    return () => clearInterval(id)
  }, [room.countdown_started_at_ms])

  return (
    <div className="min-h-full flex flex-col items-center justify-center gap-6">
      <p className="text-slate-400 text-lg">Get ready…</p>
      <div className="text-9xl font-black text-white tabular-nums">{count || '🎲'}</div>
      {room.current_topic && (
        <p className="text-slate-500 text-sm">Topic incoming</p>
      )}
    </div>
  )
}
