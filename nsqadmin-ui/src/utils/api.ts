import axios from 'axios'
import toast from 'react-hot-toast'
import type { Stats, LookupdStats, Topic, Channel } from '../types'

// Create axios instance
const api = axios.create({
  timeout: 10000,
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 404) {
      toast.error('Service not found. Please check your NSQ configuration.')
    } else if (error.code === 'ECONNREFUSED') {
      toast.error('Connection refused. Please check if NSQ services are running.')
    } else {
      toast.error(`API Error: ${error.message}`)
    }
    return Promise.reject(error)
  }
)

// NSQd API
export const nsqdApi = {
  getStats: async (address: string): Promise<Stats> => {
    const response = await api.get(`${address}/stats`)
    return response.data
  },
  
  getTopic: async (address: string, topic: string): Promise<Topic> => {
    const response = await api.get(`${address}/topic/stats?topic=${topic}`)
    return response.data
  },
  
  getChannel: async (address: string, topic: string, channel: string): Promise<Channel> => {
    const response = await api.get(`${address}/channel/stats?topic=${topic}&channel=${channel}`)
    return response.data
  },
  
  pauseTopic: async (address: string, topic: string): Promise<void> => {
    await api.post(`${address}/topic/pause?topic=${topic}`)
  },
  
  unpauseTopic: async (address: string, topic: string): Promise<void> => {
    await api.post(`${address}/topic/unpause?topic=${topic}`)
  },
  
  pauseChannel: async (address: string, topic: string, channel: string): Promise<void> => {
    await api.post(`${address}/channel/pause?topic=${topic}&channel=${channel}`)
  },
  
  unpauseChannel: async (address: string, topic: string, channel: string): Promise<void> => {
    await api.post(`${address}/channel/unpause?topic=${topic}&channel=${channel}`)
  },
  
  deleteTopic: async (address: string, topic: string): Promise<void> => {
    await api.post(`${address}/topic/delete?topic=${topic}`)
  },
  
  deleteChannel: async (address: string, topic: string, channel: string): Promise<void> => {
    await api.post(`${address}/channel/delete?topic=${topic}&channel=${channel}`)
  },
  
  emptyTopic: async (address: string, topic: string): Promise<void> => {
    await api.post(`${address}/topic/empty?topic=${topic}`)
  },
  
  emptyChannel: async (address: string, topic: string, channel: string): Promise<void> => {
    await api.post(`${address}/channel/empty?topic=${topic}&channel=${channel}`)
  },
}

// NSQLookupd API
export const lookupdApi = {
  getStats: async (address: string): Promise<LookupdStats> => {
    const response = await api.get(`${address}/stats`)
    return response.data
  },
  
  getTopics: async (address: string): Promise<string[]> => {
    const response = await api.get(`${address}/topics`)
    return response.data.topics || []
  },
  
  getChannels: async (address: string, topic: string): Promise<string[]> => {
    const response = await api.get(`${address}/channels?topic=${topic}`)
    return response.data.channels || []
  },
  
  getProducers: async (address: string): Promise<any[]> => {
    const response = await api.get(`${address}/nodes`)
    return response.data.producers || []
  },
  
  lookupTopic: async (address: string, topic: string): Promise<any[]> => {
    const response = await api.get(`${address}/lookup?topic=${topic}`)
    return response.data.producers || []
  },
}

// Health check
export const healthCheck = async (address: string): Promise<boolean> => {
  try {
    await api.get(`${address}/ping`)
    return true
  } catch {
    return false
  }
}
