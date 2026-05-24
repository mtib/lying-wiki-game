import { useEffect } from 'react'

interface Props {
  message: string | null
  onDismiss: () => void
}

export function Toast({ message, onDismiss }: Props) {
  useEffect(() => {
    if (!message) return
    const t = setTimeout(onDismiss, 4000)
    return () => clearTimeout(t)
  }, [message, onDismiss])

  if (!message) return null

  return (
    <div
      className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 bg-red-600 text-white px-5 py-3 rounded-xl shadow-lg text-sm font-medium max-w-xs text-center"
      onClick={onDismiss}
    >
      {message}
    </div>
  )
}
