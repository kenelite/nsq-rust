import { useState, useEffect, useCallback } from 'react'
import { useAppStore } from '../stores/useAppStore'
import { nsqdApi, lookupdApi, healthCheck } from '../utils/api'
import type { Stats, LookupdStats } from '../types'

export function useStats() {
  const { nsqdAddress, lookupdAddress, refreshInterval, setIsConnected } = useAppStore()
  const [stats, setStats] = useState<Stats | null>(null)
  const [lookupdStats, setLookupdStats] = useState<LookupdStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchStats = useCallback(async () => {
    try {
      setError(null)
      
      // Check health first
      const nsqdHealthy = await healthCheck(nsqdAddress)
      const lookupdHealthy = await healthCheck(lookupdAddress)
      
      if (!nsqdHealthy && !lookupdHealthy) {
        throw new Error('No NSQ services are available')
      }
      
      setIsConnected(true)
      
      // Fetch stats from available services
      const promises: Promise<any>[] = []
      
      if (nsqdHealthy) {
        promises.push(nsqdApi.getStats(nsqdAddress))
      }
      
      if (lookupdHealthy) {
        promises.push(lookupdApi.getStats(lookupdAddress))
      }
      
      const results = await Promise.allSettled(promises)
      
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          if (index === 0 && nsqdHealthy) {
            setStats(result.value)
          } else if (index === 1 && lookupdHealthy) {
            setLookupdStats(result.value)
          }
        }
      })
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      setError(errorMessage)
      setIsConnected(false)
    } finally {
      setLoading(false)
    }
  }, [nsqdAddress, lookupdAddress, setIsConnected])

  useEffect(() => {
    fetchStats()
    
    const interval = setInterval(fetchStats, refreshInterval)
    return () => clearInterval(interval)
  }, [fetchStats, refreshInterval])

  return {
    stats,
    lookupdStats,
    loading,
    error,
    refetch: fetchStats,
  }
}
