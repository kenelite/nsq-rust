import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { 
  Server, 
  ArrowLeft, 
  Activity, 
  Database,
  HardDrive,
  Clock,
  Cpu,
  Network,
  CheckCircle,
  XCircle,
  RefreshCw
} from 'lucide-react'
import { nsqdApi, lookupdApi } from '../utils/api'
import { cn } from '../utils/cn'
import type { Stats, Topic, Channel } from '../types'
import toast from 'react-hot-toast'

export function NodeDetails() {
  const { hostname } = useParams<{ hostname: string }>()
  const navigate = useNavigate()
  const [loading, setLoading] = useState(true)
  const [stats, setStats] = useState<Stats | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [refreshing, setRefreshing] = useState(false)

  const fetchNodeStats = async () => {
    if (!hostname) return
    
    try {
      setError(null)
      // Try to fetch stats from the node
      // In a real implementation, we'd need to know the node's HTTP address
      // For now, we'll try common patterns
      const addresses = [
        `http://${hostname}:4151`,
        `http://${hostname}:4161`,
        `http://localhost:4151`,
        `http://localhost:4161`,
      ]

      let success = false
      for (const address of addresses) {
        try {
          const data = await nsqdApi.getStats(address)
          setStats(data)
          success = true
          break
        } catch (e) {
          // Try next address
          continue
        }
      }

      if (!success) {
        throw new Error('Could not connect to node')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch node stats')
      toast.error('Failed to load node details')
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }

  useEffect(() => {
    fetchNodeStats()
    // Auto refresh every 5 seconds
    const interval = setInterval(() => {
      if (!refreshing) {
        fetchNodeStats()
      }
    }, 5000)
    return () => clearInterval(interval)
  }, [hostname])

  const handleRefresh = () => {
    setRefreshing(true)
    fetchNodeStats()
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60

    const parts = []
    if (days > 0) parts.push(`${days}d`)
    if (hours > 0) parts.push(`${hours}h`)
    if (minutes > 0) parts.push(`${minutes}m`)
    if (secs > 0 || parts.length === 0) parts.push(`${secs}s`)

    return parts.join(' ')
  }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  const getTotalMessages = () => {
    if (!stats?.topics) return 0
    return stats.topics.reduce((sum, topic) => sum + (topic.message_count || 0), 0)
  }

  const getTotalDepth = () => {
    if (!stats?.topics) return 0
    return stats.topics.reduce((sum, topic) => sum + (topic.depth || 0), 0)
  }

  const getTotalClients = () => {
    if (!stats?.topics) return 0
    return stats.topics.reduce((sum, topic) => 
      sum + topic.channels.reduce((channelSum, channel) => 
        channelSum + (channel.clients?.length || 0), 0), 0)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center space-x-4">
          <button
            onClick={() => navigate('/nodes')}
            className="btn-secondary"
          >
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Nodes
          </button>
        </div>
        <div className="card p-12 text-center">
          <XCircle className="h-16 w-16 text-red-500 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
            Failed to Load Node Details
          </h3>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            {error}
          </p>
          <button onClick={handleRefresh} className="btn-primary">
            Try Again
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <button
            onClick={() => navigate('/nodes')}
            className="btn-secondary"
          >
            <ArrowLeft className="h-4 w-4" />
          </button>
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center space-x-3">
              <Server className="h-8 w-8" />
              <span>{hostname}</span>
            </h1>
            <p className="text-gray-500 dark:text-gray-400">
              Node Details
            </p>
          </div>
        </div>
        <button
          onClick={handleRefresh}
          disabled={refreshing}
          className="btn-secondary"
        >
          <RefreshCw className={cn("h-4 w-4 mr-2", refreshing && "animate-spin")} />
          Refresh
        </button>
      </div>

      {/* Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="card p-6">
          <div className="flex items-center justify-between mb-2">
            <div className="p-2 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
            </div>
            <span className="text-xs font-medium px-2 py-1 rounded-full bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400">
              {stats?.health || 'OK'}
            </span>
          </div>
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">
            Status
          </h3>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            Online
          </p>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between mb-2">
            <div className="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
              <Database className="h-6 w-6 text-blue-600 dark:text-blue-400" />
            </div>
          </div>
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">
            Topics
          </h3>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {stats?.topics?.length || 0}
          </p>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between mb-2">
            <div className="p-2 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
              <Activity className="h-6 w-6 text-purple-600 dark:text-purple-400" />
            </div>
          </div>
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">
            Total Messages
          </h3>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {getTotalMessages().toLocaleString()}
          </p>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between mb-2">
            <div className="p-2 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
              <HardDrive className="h-6 w-6 text-orange-600 dark:text-orange-400" />
            </div>
          </div>
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">
            Queue Depth
          </h3>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {getTotalDepth().toLocaleString()}
          </p>
        </div>
      </div>

      {/* Node Information */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card p-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center space-x-2">
            <Server className="h-5 w-5" />
            <span>System Information</span>
          </h2>
          <div className="space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">Version:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.version || 'N/A'}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400 flex items-center space-x-1">
                <Clock className="h-3 w-3" />
                <span>Uptime:</span>
              </span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.uptime_seconds ? formatUptime(stats.uptime_seconds) : 'N/A'}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">Start Time:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.start_time ? new Date(stats.start_time * 1000).toLocaleString() : 'N/A'}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400 flex items-center space-x-1">
                <Network className="h-3 w-3" />
                <span>Connected Clients:</span>
              </span>
              <span className="text-gray-900 dark:text-white font-medium">
                {getTotalClients()}
              </span>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center space-x-2">
            <Cpu className="h-5 w-5" />
            <span>Performance Metrics</span>
          </h2>
          <div className="space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">Total Channels:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.topics?.reduce((sum, topic) => sum + topic.channels.length, 0) || 0}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">In-Flight Messages:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.topics?.reduce((sum, topic) => 
                  sum + topic.channels.reduce((channelSum, channel) => 
                    channelSum + (channel.in_flight_count || 0), 0), 0) || 0}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">Deferred Messages:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.topics?.reduce((sum, topic) => 
                  sum + topic.channels.reduce((channelSum, channel) => 
                    channelSum + (channel.deferred_count || 0), 0), 0) || 0}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-500 dark:text-gray-400">Requeue Count:</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.topics?.reduce((sum, topic) => 
                  sum + topic.channels.reduce((channelSum, channel) => 
                    channelSum + (channel.requeue_count || 0), 0), 0) || 0}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Topics List */}
      <div className="card p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center space-x-2">
          <Database className="h-5 w-5" />
          <span>Topics on this Node</span>
        </h2>
        {stats?.topics && stats.topics.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead>
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Topic Name
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Channels
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Depth
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Messages
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {stats.topics.map((topic, index) => (
                  <tr key={index} className="hover:bg-gray-50 dark:hover:bg-gray-800">
                    <td className="px-4 py-3 text-sm font-medium text-gray-900 dark:text-white">
                      {topic.topic_name}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                      {topic.channels.length}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                      {topic.depth.toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                      {topic.message_count.toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-sm">
                      <span className={cn(
                        "px-2 py-1 rounded-full text-xs font-medium",
                        topic.paused
                          ? "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400"
                          : "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                      )}>
                        {topic.paused ? 'Paused' : 'Active'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-center py-8">
            <Database className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-500 dark:text-gray-400">
              No topics found on this node
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

