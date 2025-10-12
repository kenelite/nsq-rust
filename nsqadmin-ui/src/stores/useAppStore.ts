import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface AppState {
  // Theme
  isDarkMode: boolean
  toggleDarkMode: () => void
  
  // Settings
  nsqdAddress: string
  lookupdAddress: string
  refreshInterval: number
  setNsqdAddress: (address: string) => void
  setLookupdAddress: (address: string) => void
  setRefreshInterval: (interval: number) => void
  
  // Connection status
  isConnected: boolean
  setIsConnected: (connected: boolean) => void
  
  // Selected items
  selectedTopic: string | null
  selectedChannel: string | null
  setSelectedTopic: (topic: string | null) => void
  setSelectedChannel: (channel: string | null) => void
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      // Theme
      isDarkMode: false,
      toggleDarkMode: () => set((state) => ({ isDarkMode: !state.isDarkMode })),
      
      // Settings
      nsqdAddress: 'http://localhost:4151',
      lookupdAddress: 'http://localhost:4161',
      refreshInterval: 5000,
      setNsqdAddress: (address) => set({ nsqdAddress: address }),
      setLookupdAddress: (address) => set({ lookupdAddress: address }),
      setRefreshInterval: (interval) => set({ refreshInterval: interval }),
      
      // Connection status
      isConnected: false,
      setIsConnected: (connected) => set({ isConnected: connected }),
      
      // Selected items
      selectedTopic: null,
      selectedChannel: null,
      setSelectedTopic: (topic) => set({ selectedTopic: topic }),
      setSelectedChannel: (channel) => set({ selectedChannel: channel }),
    }),
    {
      name: 'nsqadmin-settings',
      partialize: (state) => ({
        isDarkMode: state.isDarkMode,
        nsqdAddress: state.nsqdAddress,
        lookupdAddress: state.lookupdAddress,
        refreshInterval: state.refreshInterval,
      }),
    }
  )
)
