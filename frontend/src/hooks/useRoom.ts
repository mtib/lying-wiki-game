import { useEffect, useRef, useState, useCallback } from 'react'
import type { RoomSnapshot } from '../types'

const BASE = '/api'

export function useRoom(code: string | null, token: string | null) {
  const [room, setRoom] = useState<RoomSnapshot | null>(null)
  const esRef = useRef<EventSource | null>(null)
  const backoffRef = useRef(100)
  const unmountedRef = useRef(false)

  const connect = useCallback(() => {
    if (!code || !token || unmountedRef.current) return
    if (esRef.current) {
      esRef.current.close()
      esRef.current = null
    }

    const es = new EventSource(`${BASE}/rooms/${code}/events?token=${token}`)
    esRef.current = es

    es.addEventListener('message', (e) => {
      try {
        const parsed = JSON.parse(e.data)
        if (parsed.type === 'room_state') {
          setRoom(parsed.data as RoomSnapshot)
          backoffRef.current = 100 // reset backoff on successful message
        }
      } catch {}
    })

    es.onerror = () => {
      es.close()
      esRef.current = null
      if (unmountedRef.current) return
      const delay = Math.min(backoffRef.current, 10000)
      backoffRef.current = Math.min(backoffRef.current * 2, 10000)
      setTimeout(connect, delay)
    }
  }, [code, token])

  useEffect(() => {
    unmountedRef.current = false
    connect()
    const onVisible = () => {
      if (document.visibilityState === 'visible') connect()
    }
    document.addEventListener('visibilitychange', onVisible)
    return () => {
      unmountedRef.current = true
      esRef.current?.close()
      document.removeEventListener('visibilitychange', onVisible)
    }
  }, [connect])

  return room
}
