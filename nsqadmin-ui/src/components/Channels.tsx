import { useMemo, useState } from 'react'
import { 
  Network,
  Search,
  Filter,
  Pause,
  Play,
  Trash2,
  RefreshCcw
} from 'lucide-react'
import { useStats } from '../hooks/useStats'
import { useAppStore } from '../stores/useAppStore'
import { cn } from '../utils/cn'
import toast from 'react-hot-toast'
import { nsqdApi } from '../utils/api'

export function Channels() {
  const { stats } = useStats()
  const { nsqdAddress } = useAppStore()
  const [searchTerm, setSearchTerm] = useState('')
  const [showPausedOnly, setShowPausedOnly] = useState(false)
  const [selectedTopic, setSelectedTopic] = useState<string>('')

  const allChannels = useMemo(() => {
    const pairs: { topic: string, channel: any }[] = []
    stats?.topics?.forEach(t => {
      t.channels.forEach(c => {
        pairs.push({ topic: t.topic_name, channel: c })
      })
    })
    return pairs
  }, [stats])

  const filtered = allChannels.filter(({ topic, channel }) => {
    const matchesTopic = selectedTopic ? topic === selectedTopic : true
    const matchesSearch = channel.channel_name.toLowerCase().includes(searchTerm.toLowerCase())
      || topic.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesFilter = !showPausedOnly || channel.paused
    return matchesTopic && matchesSearch && matchesFilter
  })

  const handlePause = async (topicName: string, channelName: string) => {
    try {
      await nsqdApi.pauseChannel(nsqdAddress, topicName, channelName)
      toast.success(`Channel ${channelName} paused`)
    } catch {
      toast.error('Failed to pause channel')
    }
  }

  const handleUnpause = async (topicName: string, channelName: string) => {
    try {
      await nsqdApi.unpauseChannel(nsqdAddress, topicName, channelName)
      toast.success(`Channel ${channelName} unpaused`)
    } catch {
      toast.error('Failed to unpause channel')
    }
  }

  const handleDelete = async (topicName: string, channelName: string) => {
    if (!window.confirm(`Delete channel "${channelName}" on topic "${topicName}"?`)) return
    try {
      await nsqdApi.deleteChannel(nsqdAddress, topicName, channelName)
      toast.success(`Channel ${channelName} deleted`)
    } catch {
      toast.error('Failed to delete channel')
    }
  }

  const handleEmpty = async (topicName: string, channelName: string) => {
    if (!window.confirm(`Empty all messages from channel "${channelName}"?`)) return
    try {
      await nsqdApi.emptyChannel(nsqdAddress, topicName, channelName)
      toast.success(`Channel ${channelName} emptied`)
    } catch {
      toast.error('Failed to empty channel')
    }
  }

  const topics = stats?.topics?.map(t => t.topic_name) || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Channels</h1>
          <p className="text-gray-500 dark:text-gray-400">Monitor and manage channels across topics</p>
        </div>
      </div>

      <div className="card p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 items-center">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search channels or topics..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="input pl-10"
            />
          </div>
          <div>
            <select
              className="input"
              value={selectedTopic}
              onChange={(e) => setSelectedTopic(e.target.value)}
            >
              <option value="">All topics</option>
              {topics.map(t => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>
          <div className="flex md:justify-end">
            <button
              onClick={() => setShowPausedOnly(!showPausedOnly)}
              className={cn(
                "btn-secondary",
                showPausedOnly && "bg-primary-100 text-primary-700 dark:bg-primary-900/20 dark:text-primary-400"
              )}
            >
              <Filter className="h-4 w-4 mr-2" />
              Paused Only
            </button>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filtered.map(({ topic, channel }) => (
          <div key={`${topic}:${channel.channel_name}`} className="card p-6 hover:shadow-md transition-shadow">
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center space-x-3">
                <Network className="h-6 w-6 text-primary-600 dark:text-primary-400" />
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    {channel.channel_name}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    Topic: {topic}
                  </p>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3 text-sm mb-4">
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Messages</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.message_count.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Depth</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.depth.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">In Flight</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.in_flight_count.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Deferred</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.deferred_count.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Requeue</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.requeue_count.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500 dark:text-gray-400">Timeout</span>
                <span className="text-gray-900 dark:text-white font-medium">{channel.timeout_count.toLocaleString()}</span>
              </div>
              <div className="col-span-2">
                <span className={cn(
                  "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                  channel.paused
                    ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                    : "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                )}>
                  {channel.paused ? 'Paused' : 'Active'}
                </span>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              {channel.paused ? (
                <button onClick={() => handleUnpause(topic, channel.channel_name)} className="btn-secondary flex-1">
                  <Play className="h-4 w-4 mr-2" />
                  Unpause
                </button>
              ) : (
                <button onClick={() => handlePause(topic, channel.channel_name)} className="btn-secondary flex-1">
                  <Pause className="h-4 w-4 mr-2" />
                  Pause
                </button>
              )}
              <button onClick={() => handleEmpty(topic, channel.channel_name)} className="btn-secondary" title="Empty channel">
                <RefreshCcw className="h-4 w-4" />
              </button>
              <button onClick={() => handleDelete(topic, channel.channel_name)} className="btn-danger" title="Delete channel">
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div className="text-center py-12">
          <Network className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No channels found</h3>
          <p className="text-gray-500 dark:text-gray-400">
            {searchTerm || showPausedOnly || selectedTopic
              ? 'Try adjusting your search or filters'
              : 'Channels will appear once topics and consumers are active'}
          </p>
        </div>
      )}
    </div>
  )
}


