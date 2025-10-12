import { Server, CheckCircle } from 'lucide-react'
import { useStats } from '../hooks/useStats'

export function NodeStatus() {
  const { stats, lookupdStats } = useStats()

  const nodes = [
    ...(stats?.producers || []),
    ...(lookupdStats?.producers || [])
  ]

  if (!nodes.length) {
    return (
      <div className="text-center py-8 text-gray-500 dark:text-gray-400">
        No nodes found
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {nodes.map((node, index) => (
        <div key={`${node.hostname}-${index}`} className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center space-x-3">
            <Server className="h-5 w-5 text-gray-400" />
            <div>
              <div className="text-sm font-medium text-gray-900 dark:text-white">
                {node.hostname}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {node.broadcast_address}:{node.http_port}
              </div>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <CheckCircle className="h-4 w-4 text-green-500" />
            <span className="text-xs text-gray-500 dark:text-gray-400">
              Online
            </span>
          </div>
        </div>
      ))}
      
      <div className="text-xs text-gray-500 dark:text-gray-400 text-center pt-2">
        Last updated: {new Date().toLocaleTimeString()}
      </div>
    </div>
  )
}
