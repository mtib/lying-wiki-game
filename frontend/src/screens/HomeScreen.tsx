import { useState } from 'react'
import { createRoom, joinRoom } from '../api'
import { saveDeviceToken } from '../hooks/useDeviceToken'

interface Props {
  onJoined: (code: string, token: string, name: string) => void
}

export function HomeScreen({ onJoined }: Props) {
  const [name, setName] = useState('')
  const [code, setCode] = useState('')
  const [mode, setMode] = useState<'home' | 'join'>('home')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const handle = async (action: () => Promise<{ code: string; token: string }>) => {
    if (!name.trim()) { setError('Enter your name first'); return }
    setLoading(true)
    setError(null)
    try {
      const { code: roomCode, token } = await action()
      saveDeviceToken(roomCode, token, name.trim())
      onJoined(roomCode, token, name.trim())
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-full flex flex-col items-center justify-center px-6 gap-6">
      <h1 className="text-3xl font-bold text-center text-white">
        2 of These People<br />Are Lying
      </h1>

      <input
        className="w-full max-w-sm bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="Your name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        maxLength={24}
      />

      {mode === 'home' && (
        <div className="flex flex-col w-full max-w-sm gap-3">
          <button
            className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50"
            disabled={loading}
            onClick={() => handle(() => createRoom(name.trim()))}
          >
            Create Room
          </button>
          <button
            className="bg-slate-700 text-white font-semibold py-3 rounded-xl text-base"
            onClick={() => setMode('join')}
          >
            Join Room
          </button>
        </div>
      )}

      {mode === 'join' && (
        <div className="flex flex-col w-full max-w-sm gap-3">
          <input
            className="w-full bg-slate-800 text-white rounded-xl px-4 py-3 text-base placeholder-slate-500 outline-none focus:ring-2 focus:ring-blue-500 uppercase tracking-widest text-center"
            placeholder="Room code"
            value={code}
            onChange={(e) => setCode(e.target.value.toUpperCase().slice(0, 6))}
            maxLength={6}
          />
          <button
            className="bg-blue-600 text-white font-semibold py-3 rounded-xl text-base disabled:opacity-50"
            disabled={loading || code.length !== 6}
            onClick={() => handle(async () => {
              const { token } = await joinRoom(code, name.trim())
              return { code, token }
            })}
          >
            Join
          </button>
          <button
            className="text-slate-400 text-sm"
            onClick={() => setMode('home')}
          >
            Back
          </button>
        </div>
      )}

      {error && <p className="text-red-400 text-sm text-center">{error}</p>}
    </div>
  )
}
