import { useState } from 'react'
import { 
  Database, 
  Plus, 
  Search, 
  Filter,
  Pause,
  Play,
  Trash2,
  MoreHorizontal
} from 'lucide-react'
import { useStats } from '../hooks/useStats'
import { cn } from '../utils/cn'
import toast from 'react-hot-toast'

export function Topics() {
  const { stats } = useStats()
  const [searchTerm, setSearchTerm] = useState('')
  const [showPausedOnly, setShowPausedOnly] = useState(false)

  const filteredTopics = stats?.topics?.filter(topic => {
    const matchesSearch = topic.topic_name.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesFilter = !showPausedOnly || topic.paused
    return matchesSearch && matchesFilter
  }) || []

  const handlePauseTopic = async (topicName: string) => {
    try {
      // TODO: Implement actual API call
      toast.success(`Topic ${topicName} paused`)
    } catch (error) {
      toast.error('Failed to pause topic')
    }
  }

  const handleUnpauseTopic = async (topicName: string) => {
    try {
      // TODO: Implement actual API call
      toast.success(`Topic ${topicName} unpaused`)
    } catch (error) {
      toast.error('Failed to unpause topic')
    }
  }

  const handleDeleteTopic = async (topicName: string) => {
    if (window.confirm(`Are you sure you want to delete topic "${topicName}"?`)) {
      try {
        // TODO: Implement actual API call
        toast.success(`Topic ${topicName} deleted`)
      } catch (error) {
        toast.error('Failed to delete topic')
      }
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Topics
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Manage your NSQ topics
          </p>
        </div>
        <button className="btn-primary">
          <Plus className="h-4 w-4 mr-2" />
          Create Topic
        </button>
      </div>

      {/* Filters */}
      <div className="card p-4">
        <div className="flex items-center space-x-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search topics..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="input pl-10"
            />
          </div>
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

      {/* Topics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredTopics.map((topic) => (
          <div key={topic.topic_name} className="card p-6 hover:shadow-md transition-shadow">
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center space-x-3">
                <Database className="h-6 w-6 text-primary-600 dark:text-primary-400" />
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                    {topic.topic_name}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {topic.channels.length} channels
                  </p>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                <button className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
                  <MoreHorizontal className="h-4 w-4" />
                </button>
              </div>
            </div>

            <div className="space-y-3 mb-4">
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Messages:</span>
                <span className="text-gray-900 dark:text-white font-medium">
                  {topic.message_count.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Depth:</span>
                <span className="text-gray-900 dark:text-white font-medium">
                  {topic.depth.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-500 dark:text-gray-400">Status:</span>
                <span className={cn(
                  "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                  topic.paused
                    ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                    : "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                )}>
                  {topic.paused ? 'Paused' : 'Active'}
                </span>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              {topic.paused ? (
                <button
                  onClick={() => handleUnpauseTopic(topic.topic_name)}
                  className="btn-secondary flex-1"
                >
                  <Play className="h-4 w-4 mr-2" />
                  Unpause
                </button>
              ) : (
                <button
                  onClick={() => handlePauseTopic(topic.topic_name)}
                  className="btn-secondary flex-1"
                >
                  <Pause className="h-4 w-4 mr-2" />
                  Pause
                </button>
              )}
              <button
                onClick={() => handleDeleteTopic(topic.topic_name)}
                className="btn-danger"
                title="Delete topic"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {filteredTopics.length === 0 && (
        <div className="text-center py-12">
          <Database className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            No topics found
          </h3>
          <p className="text-gray-500 dark:text-gray-400">
            {searchTerm || showPausedOnly 
              ? 'Try adjusting your search or filters'
              : 'Create your first topic to get started'
            }
          </p>
        </div>
      )}
    </div>
  )
}
