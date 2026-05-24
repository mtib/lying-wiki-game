const KEY = (code: string) => `lwg-token-${code}`
const NAME_KEY = (code: string) => `lwg-name-${code}`

export function saveDeviceToken(code: string, token: string, name: string) {
  localStorage.setItem(KEY(code), token)
  localStorage.setItem(NAME_KEY(code), name)
}

export function loadDeviceToken(code: string): { token: string; name: string } | null {
  const token = localStorage.getItem(KEY(code))
  const name = localStorage.getItem(NAME_KEY(code))
  if (!token || !name) return null
  return { token, name }
}

export function clearDeviceToken(code: string) {
  localStorage.removeItem(KEY(code))
  localStorage.removeItem(NAME_KEY(code))
}
