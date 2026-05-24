export type RoomStateName =
  | 'lobby'
  | 'topic_submission'
  | 'countdown'
  | 'round_active'
  | 'round_reveal'

export interface PlayerSnapshot {
  name: string
  score: number
  connected: boolean
  submitted_this_round: boolean
}

export interface RoundRevealData {
  topic: string
  owner_name: string
  guesser_name: string
  guessed_name: string
  correct: boolean
  points: number
}

export interface LogEntry {
  round: number
  topic: string
  owner_name: string
  guesser_name: string
  guessed_name: string
  correct: boolean
  points: number
}

export interface RoomSnapshot {
  code: string
  state: RoomStateName
  players: PlayerSnapshot[]
  current_topic: string | null
  reveal: RoundRevealData | null
  log: LogEntry[]
  round_number: number
  guesser_name: string | null
  countdown_started_at_ms: number | null
}

export interface WikiArticle {
  title: string
  url: string
  extract: string
  html: string
}
