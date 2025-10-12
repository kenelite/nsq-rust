import { useState } from 'react'
import { 
  Server, 
  Search, 
  CheckCircle, 
  XCircle, 
  Database,
  Users
} from 'lucide-react'
import { useStats } from '../hooks/useStats'
import { cn } from '../utils/cn'

export function Nodes() {
  const { stats, lookupdStats } = useStats()
  const [searchTerm, setSearchTerm] = useState('')

  const nodes = [
    ...(stats?.producers || []),
    ...(lookupdStats?.producers || [])
  ]

  const filteredNodes = nodes.filter(node =>
    node.hostname.toLowerCase().includes(searchTerm.toLowerCase()) ||
    node.broadcast_address.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const getNodeStatus = (_node: any) => {
    // Mock status - in real implementation, this would check actual connectivity
    return 'online'
  }

  const getNodeType = (node: any) => {
    // Determine if it's nsqd or nsqlookupd based on available data
    return node.tcp_port ? 'nsqd' : 'nsqlookupd'
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Nodes
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Monitor your NSQ cluster nodes
          </p>
        </div>
      </div>

      {/* Search */}
      <div className="card p-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search nodes..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="input pl-10"
          />
        </div>
      </div>

      {/* Nodes Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredNodes.map((node, index) => {
          const status = getNodeStatus(node)
          const type = getNodeType(node)
          
          return (
            <div key={`${node.hostname}-${index}`} className="card p-6">
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center space-x-3">
                  <div className={cn(
                    "p-2 rounded-lg",
                    type === 'nsqd' 
                      ? "bg-blue-50 dark:bg-blue-900/20" 
                      : "bg-green-50 dark:bg-green-900/20"
                  )}>
                    <Server className={cn(
                      "h-5 w-5",
                      type === 'nsqd' 
                        ? "text-blue-600 dark:text-blue-400" 
                        : "text-green-600 dark:text-green-400"
                    )} />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                      {node.hostname}
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {type.toUpperCase()} Node
                    </p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  {status === 'online' ? (
                    <CheckCircle className="h-5 w-5 text-green-500" />
                  ) : (
                    <XCircle className="h-5 w-5 text-red-500" />
                  )}
                  <span className={cn(
                    "text-xs font-medium px-2 py-1 rounded-full",
                    status === 'online'
                      ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                      : "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                  )}>
                    {status}
                  </span>
                </div>
              </div>

              <div className="space-y-3 mb-4">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500 dark:text-gray-400">Address:</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {node.broadcast_address}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500 dark:text-gray-400">HTTP Port:</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {node.http_port}
                  </span>
                </div>
                {node.tcp_port && (
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">TCP Port:</span>
                    <span className="text-gray-900 dark:text-white font-medium">
                      {node.tcp_port}
                    </span>
                  </div>
                )}
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500 dark:text-gray-400">Version:</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {node.version}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500 dark:text-gray-400">Last Update:</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {new Date(node.last_update).toLocaleString()}
                  </span>
                </div>
              </div>

              <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
                <div className="flex items-center space-x-4 text-xs text-gray-500 dark:text-gray-400">
                  <div className="flex items-center space-x-1">
                    <Database className="h-3 w-3" />
                    <span>Topics: {stats?.topics?.length || 0}</span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <Users className="h-3 w-3" />
                    <span>Clients: {stats?.topics?.reduce((sum, topic) => 
                      sum + topic.channels.reduce((channelSum, channel) => 
                        channelSum + channel.clients.length, 0), 0) || 0}
                    </span>
                  </div>
                </div>
                <button className="text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 text-sm font-medium">
                  View Details
                </button>
              </div>
            </div>
          )
        })}
      </div>

      {filteredNodes.length === 0 && (
        <div className="text-center py-12">
          <Server className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            No nodes found
          </h3>
          <p className="text-gray-500 dark:text-gray-400">
            {searchTerm 
              ? 'Try adjusting your search'
              : 'No NSQ nodes are currently running'
            }
          </p>
        </div>
      )}
    </div>
  )
}
