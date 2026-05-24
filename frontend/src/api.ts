const BASE = '/api'

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  const text = await res.text()
  const data = text ? JSON.parse(text) : {}
  if (!res.ok) throw new Error(data.error ?? 'Request failed')
  return data as T
}

export function createRoom(name: string) {
  return post<{ code: string; token: string }>('/rooms', { name })
}

export function joinRoom(code: string, name: string, token?: string) {
  return post<{ token: string }>(`/rooms/${code}/join`, { name, token })
}

export function startGame(code: string, token: string) {
  return post<void>(`/rooms/${code}/start-game`, { token })
}

export function submitTopic(code: string, token: string, title: string) {
  return post<void>(`/rooms/${code}/topic`, { token, title })
}

export function startRound(code: string, token: string) {
  return post<void>(`/rooms/${code}/start-round`, { token })
}

export function submitGuess(code: string, token: string, guessed_name: string) {
  return post<void>(`/rooms/${code}/guess`, { token, guessed_name })
}

export function fetchRandomWikiArticle() {
  return fetch(`${BASE}/wiki/random`).then(async (res) => {
    const data = await res.json()
    if (!res.ok) throw new Error(data.error ?? 'Failed to fetch article')
    return data as import('./types').WikiArticle
  })
}
