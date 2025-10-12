import { 
  Database, 
  Users, 
  Activity, 
  TrendingUp,
  AlertCircle
} from 'lucide-react'
import { useStats } from '../hooks/useStats'
import { StatCard } from './StatCard'
import { MessageRateChart } from './MessageRateChart'
import { TopicList } from './TopicList'
import { NodeStatus } from './NodeStatus'

export function Dashboard() {
  const { stats, loading, error } = useStats()

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            Connection Error
          </h3>
          <p className="text-gray-500 dark:text-gray-400">{error}</p>
        </div>
      </div>
    )
  }

  const totalTopics = stats?.topics?.length || 0
  const totalChannels = stats?.topics?.reduce((sum, topic) => sum + topic.channels.length, 0) || 0
  const totalMessages = stats?.topics?.reduce((sum, topic) => sum + topic.message_count, 0) || 0
  const totalClients = stats?.topics?.reduce((sum, topic) => 
    sum + topic.channels.reduce((channelSum, channel) => channelSum + channel.clients.length, 0), 0) || 0

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          Overview of your NSQ cluster
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Topics"
          value={totalTopics}
          icon={Database}
          color="blue"
        />
        <StatCard
          title="Channels"
          value={totalChannels}
          icon={Users}
          color="green"
        />
        <StatCard
          title="Messages"
          value={totalMessages.toLocaleString()}
          icon={Activity}
          color="purple"
        />
        <StatCard
          title="Clients"
          value={totalClients}
          icon={TrendingUp}
          color="orange"
        />
      </div>

      {/* Charts and Tables */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Message Rate
          </h3>
          <MessageRateChart />
        </div>
        
        <div className="card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Node Status
          </h3>
          <NodeStatus />
        </div>
      </div>

      {/* Topics Table */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Topics Overview
        </h3>
        <TopicList />
      </div>
    </div>
  )
}
