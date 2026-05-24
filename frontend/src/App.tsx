import { useState, useCallback } from 'react'
import { useRoom } from './hooks/useRoom'
import { loadDeviceToken, saveDeviceToken, clearDeviceToken } from './hooks/useDeviceToken'
import { Toast } from './components/Toast'
import { HomeScreen } from './screens/HomeScreen'
import { LobbyScreen } from './screens/LobbyScreen'
import { TopicSubmissionScreen } from './screens/TopicSubmissionScreen'
import { CountdownScreen } from './screens/CountdownScreen'
import { RoundActiveScreen } from './screens/RoundActiveScreen'
import { RoundRevealScreen } from './screens/RoundRevealScreen'

export default function App() {
  const [session, setSession] = useState<{ code: string; token: string; name: string } | null>(() => {
    // Attempt to restore from localStorage (latest room code)
    const lastCode = localStorage.getItem('lwg-last-code')
    if (!lastCode) return null
    const saved = loadDeviceToken(lastCode)
    if (!saved) return null
    return { code: lastCode, ...saved }
  })

  const [toast, setToast] = useState<string | null>(null)
  const onError = useCallback((msg: string) => setToast(msg), [])

  const room = useRoom(session?.code ?? null, session?.token ?? null)

  const onJoined = (code: string, token: string, name: string) => {
    localStorage.setItem('lwg-last-code', code)
    saveDeviceToken(code, token, name)
    setSession({ code, token, name })
  }

  const onLeave = () => {
    if (session) clearDeviceToken(session.code)
    localStorage.removeItem('lwg-last-code')
    setSession(null)
  }

  if (!session) {
    return (
      <div className="min-h-full">
        <HomeScreen onJoined={onJoined} />
      </div>
    )
  }

  if (!room) {
    return (
      <div className="min-h-full flex items-center justify-center text-slate-400 flex-col gap-4">
        <span>Connecting…</span>
        <button onClick={onLeave} className="text-slate-500 text-sm underline">Leave room</button>
      </div>
    )
  }

  const props = { room, token: session.token, myName: session.name, onError, onLeave }

  return (
    <div className="min-h-full">
      {room.state === 'lobby' && <LobbyScreen {...props} />}
      {room.state === 'topic_submission' && <TopicSubmissionScreen {...props} />}
      {room.state === 'countdown' && <CountdownScreen room={room} />}
      {room.state === 'round_active' && <RoundActiveScreen {...props} />}
      {room.state === 'round_reveal' && <RoundRevealScreen {...props} />}
      <Toast message={toast} onDismiss={() => setToast(null)} />
    </div>
  )
}
