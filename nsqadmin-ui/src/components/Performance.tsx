import { useMemo } from 'react'
import { MessageRateChart } from './MessageRateChart'
import { StatCard } from './StatCard'
import { Gauge } from 'lucide-react'
import { useStats } from '../hooks/useStats'

export function Performance() {
  const { stats } = useStats()

  const totals = useMemo(() => {
    const topics = stats?.topics || []
    let totalMessages = 0
    let totalDepth = 0
    let totalInFlight = 0
    let totalDeferred = 0

    for (const t of topics) {
      totalMessages += t.message_count
      totalDepth += t.depth
      for (const c of t.channels) {
        totalInFlight += c.in_flight_count
        totalDeferred += c.deferred_count
      }
    }

    return { totalMessages, totalDepth, totalInFlight, totalDeferred }
  }, [stats])

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Performance</h1>
          <p className="text-gray-500 dark:text-gray-400">Throughput and backlog overview</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <StatCard title="Total Messages" value={totals.totalMessages.toLocaleString()} icon={Gauge} trend={{ label: 'Since start', value: '+0%' }} />
        <StatCard title="Queue Depth" value={totals.totalDepth.toLocaleString()} icon={Gauge} trend={{ label: 'Current', value: '' }} />
        <StatCard title="In-Flight" value={totals.totalInFlight.toLocaleString()} icon={Gauge} trend={{ label: 'Consumers', value: '' }} />
        <StatCard title="Deferred" value={totals.totalDeferred.toLocaleString()} icon={Gauge} trend={{ label: 'Delayed', value: '' }} />
      </div>

      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Message Rate (msgs/sec)</h2>
        </div>
        <MessageRateChart />
      </div>
    </div>
  )
}


