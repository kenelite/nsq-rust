import { useMemo, useState } from 'react'
import { MessageRateChart } from './MessageRateChart'
import { StatCard } from './StatCard'
import { 
  Gauge, 
  Activity, 
  TrendingUp, 
  AlertTriangle,
  CheckCircle,
  Clock,
  Zap,
  Database,
  Server
} from 'lucide-react'
import { useStats } from '../hooks/useStats'
import { cn } from '../utils/cn'

export function Performance() {
  const { stats } = useStats()
  const [timeRange, setTimeRange] = useState<'1m' | '5m' | '15m' | '1h'>('5m')

  const metrics = useMemo(() => {
    const topics = stats?.topics || []
    let totalMessages = 0
    let totalDepth = 0
    let totalBackendDepth = 0
    let totalInFlight = 0
    let totalDeferred = 0
    let totalRequeue = 0
    let totalTimeout = 0
    let channelCount = 0
    let pausedTopics = 0

    for (const t of topics) {
      totalMessages += t.message_count
      totalDepth += t.depth
      totalBackendDepth += t.backend_depth || 0
      if (t.paused) pausedTopics++
      
      for (const c of t.channels) {
        channelCount++
        totalInFlight += c.in_flight_count
        totalDeferred += c.deferred_count
        totalRequeue += c.requeue_count || 0
        totalTimeout += c.timeout_count || 0
      }
    }

    // Calculate rates and health
    const requeueRate = totalMessages > 0 ? (totalRequeue / totalMessages) * 100 : 0
    const timeoutRate = totalMessages > 0 ? (totalTimeout / totalMessages) * 100 : 0
    const depthRate = totalMessages > 0 ? (totalDepth / totalMessages) * 100 : 0

    return {
      totalMessages,
      totalDepth,
      totalBackendDepth,
      totalInFlight,
      totalDeferred,
      totalRequeue,
      totalTimeout,
      channelCount,
      topicCount: topics.length,
      pausedTopics,
      requeueRate,
      timeoutRate,
      depthRate,
    }
  }, [stats])

  // Health status calculation
  const healthStatus = useMemo(() => {
    if (metrics.requeueRate > 10) return { status: 'critical', label: 'Critical', color: 'red' }
    if (metrics.requeueRate > 5 || metrics.timeoutRate > 5) return { status: 'warning', label: 'Warning', color: 'yellow' }
    if (metrics.depthRate > 20) return { status: 'warning', label: 'High Load', color: 'yellow' }
    return { status: 'healthy', label: 'Healthy', color: 'green' }
  }, [metrics])

  // Performance recommendations
  const recommendations = useMemo(() => {
    const recs: Array<{ type: 'warning' | 'info'; message: string }> = []
    
    if (metrics.requeueRate > 5) {
      recs.push({
        type: 'warning',
        message: `High requeue rate (${metrics.requeueRate.toFixed(1)}%). Consider optimizing consumer processing logic.`
      })
    }
    
    if (metrics.timeoutRate > 3) {
      recs.push({
        type: 'warning',
        message: `High timeout rate (${metrics.timeoutRate.toFixed(1)}%). Increase message timeout or improve consumer performance.`
      })
    }
    
    if (metrics.depthRate > 20) {
      recs.push({
        type: 'warning',
        message: `Queue depth is high (${metrics.depthRate.toFixed(1)}% of total messages). Consider adding more consumers.`
      })
    }
    
    if (metrics.pausedTopics > 0) {
      recs.push({
        type: 'info',
        message: `${metrics.pausedTopics} topic(s) are paused. Resume them to continue message processing.`
      })
    }

    if (recs.length === 0) {
      recs.push({
        type: 'info',
        message: 'System is performing well. All metrics are within normal ranges.'
      })
    }
    
    return recs
  }, [metrics])

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Performance</h1>
          <p className="text-gray-500 dark:text-gray-400">Real-time throughput and performance metrics</p>
        </div>
        <div className="flex items-center space-x-2">
          <span className={cn(
            "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium",
            healthStatus.color === 'green' && "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400",
            healthStatus.color === 'yellow' && "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400",
            healthStatus.color === 'red' && "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
          )}>
            {healthStatus.status === 'healthy' && <CheckCircle className="h-4 w-4 mr-1" />}
            {healthStatus.status === 'warning' && <AlertTriangle className="h-4 w-4 mr-1" />}
            {healthStatus.status === 'critical' && <AlertTriangle className="h-4 w-4 mr-1" />}
            {healthStatus.label}
          </span>
        </div>
      </div>

      {/* Overview Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <StatCard 
          title="Total Messages" 
          value={metrics.totalMessages.toLocaleString()} 
          icon={Activity} 
          color="blue"
        />
        <StatCard 
          title="Queue Depth" 
          value={metrics.totalDepth.toLocaleString()} 
          icon={Database} 
          color={metrics.depthRate > 20 ? 'red' : metrics.depthRate > 10 ? 'orange' : 'green'}
        />
        <StatCard 
          title="In-Flight" 
          value={metrics.totalInFlight.toLocaleString()} 
          icon={Zap} 
          color="purple"
        />
        <StatCard 
          title="Deferred" 
          value={metrics.totalDeferred.toLocaleString()} 
          icon={Clock} 
          color="orange"
        />
      </div>

      {/* Performance Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">Requeue Rate</h3>
            <TrendingUp className={cn(
              "h-4 w-4",
              metrics.requeueRate > 5 ? "text-red-500" : "text-green-500"
            )} />
          </div>
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {metrics.requeueRate.toFixed(2)}%
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {metrics.totalRequeue.toLocaleString()} requeued
          </p>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">Timeout Rate</h3>
            <Clock className={cn(
              "h-4 w-4",
              metrics.timeoutRate > 3 ? "text-red-500" : "text-green-500"
            )} />
          </div>
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {metrics.timeoutRate.toFixed(2)}%
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {metrics.totalTimeout.toLocaleString()} timeouts
          </p>
        </div>

        <div className="card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">Backend Depth</h3>
            <Database className="h-4 w-4 text-primary-500" />
          </div>
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {metrics.totalBackendDepth.toLocaleString()}
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            On disk
          </p>
        </div>
      </div>

      {/* Message Rate Chart */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Message Throughput
          </h2>
          <div className="flex items-center space-x-2">
            {(['1m', '5m', '15m', '1h'] as const).map((range) => (
              <button
                key={range}
                onClick={() => setTimeRange(range)}
                className={cn(
                  "px-3 py-1 text-xs font-medium rounded-md transition-colors",
                  timeRange === range
                    ? "bg-primary-600 text-white"
                    : "bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
                )}
              >
                {range}
              </button>
            ))}
          </div>
        </div>
        <MessageRateChart />
      </div>

      {/* Performance Recommendations */}
      <div className="card p-6">
        <div className="flex items-center mb-4">
          <Gauge className="h-5 w-5 text-primary-600 dark:text-primary-400 mr-2" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Performance Analysis
          </h2>
        </div>
        <div className="space-y-3">
          {recommendations.map((rec, index) => (
            <div
              key={index}
              className={cn(
                "flex items-start p-4 rounded-lg",
                rec.type === 'warning' 
                  ? "bg-yellow-50 dark:bg-yellow-900/10" 
                  : "bg-blue-50 dark:bg-blue-900/10"
              )}
            >
              {rec.type === 'warning' ? (
                <AlertTriangle className="h-5 w-5 text-yellow-600 dark:text-yellow-400 mr-3 mt-0.5 flex-shrink-0" />
              ) : (
                <CheckCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 mr-3 mt-0.5 flex-shrink-0" />
              )}
              <p className={cn(
                "text-sm",
                rec.type === 'warning'
                  ? "text-yellow-800 dark:text-yellow-200"
                  : "text-blue-800 dark:text-blue-200"
              )}>
                {rec.message}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Cluster Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="card p-6">
          <div className="flex items-center mb-4">
            <Server className="h-5 w-5 text-primary-600 dark:text-primary-400 mr-2" />
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Cluster Stats</h3>
          </div>
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-500 dark:text-gray-400">Topics</span>
              <span className="text-gray-900 dark:text-white font-medium">{metrics.topicCount}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500 dark:text-gray-400">Channels</span>
              <span className="text-gray-900 dark:text-white font-medium">{metrics.channelCount}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500 dark:text-gray-400">Paused Topics</span>
              <span className={cn(
                "font-medium",
                metrics.pausedTopics > 0 
                  ? "text-yellow-600 dark:text-yellow-400" 
                  : "text-green-600 dark:text-green-400"
              )}>
                {metrics.pausedTopics}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500 dark:text-gray-400">Producers</span>
              <span className="text-gray-900 dark:text-white font-medium">
                {stats?.producers?.length || 0}
              </span>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <div className="flex items-center mb-4">
            <Activity className="h-5 w-5 text-primary-600 dark:text-primary-400 mr-2" />
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">System Health</h3>
          </div>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-gray-500 dark:text-gray-400">Overall Status</span>
              <span className={cn(
                "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                healthStatus.color === 'green' && "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400",
                healthStatus.color === 'yellow' && "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400",
                healthStatus.color === 'red' && "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
              )}>
                {healthStatus.label}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-500 dark:text-gray-400">Message Flow</span>
              <span className={cn(
                "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                metrics.totalInFlight > 0 
                  ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                  : "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400"
              )}>
                {metrics.totalInFlight > 0 ? 'Active' : 'Idle'}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-500 dark:text-gray-400">Queue Status</span>
              <span className={cn(
                "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                metrics.depthRate < 10 
                  ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                  : metrics.depthRate < 20
                  ? "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400"
                  : "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
              )}>
                {metrics.depthRate < 10 ? 'Normal' : metrics.depthRate < 20 ? 'Elevated' : 'High'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}


