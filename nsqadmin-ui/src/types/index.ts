// NSQ Types
export interface NSQNode {
  remote_address: string
  hostname: string
  broadcast_address: string
  tcp_port: number
  http_port: number
  version: string
  last_update: string
}

export interface Topic {
  topic_name: string
  channels: Channel[]
  depth: number
  backend_depth: number
  message_count: number
  paused: boolean
}

export interface Channel {
  channel_name: string
  depth: number
  backend_depth: number
  in_flight_count: number
  deferred_count: number
  message_count: number
  requeue_count: number
  timeout_count: number
  clients: Client[]
  paused: boolean
}

export interface Client {
  client_id: string
  hostname: string
  version: string
  remote_address: string
  state: number
  ready_count: number
  in_flight_count: number
  message_count: number
  finish_count: number
  requeue_count: number
  connect_ts: number
  sample_rate: number
  deflate: boolean
  snappy: boolean
  user_agent: string
  tls: boolean
  tls_cipher_suite: string
  tls_version: string
  tls_negotiated_protocol: string
  tls_negotiated_protocol_is_mutual: boolean
}

export interface Stats {
  version: string
  health: string
  start_time: number
  uptime: string
  uptime_seconds: number
  producers: NSQNode[]
  topics: Topic[]
}

export interface LookupdStats {
  version: string
  health: string
  start_time: number
  uptime: string
  uptime_seconds: number
  topics: string[]
  channels: string[]
  producers: NSQNode[]
}

export interface MessageRate {
  timestamp: number
  rate: number
  count: number
}
