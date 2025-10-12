import { useState } from 'react'
import { 
  Save, 
  RefreshCw,
  Server,
  Globe,
  Clock
} from 'lucide-react'
import { useAppStore } from '../stores/useAppStore'
import toast from 'react-hot-toast'

export function Settings() {
  const {
    nsqdAddress,
    lookupdAddress,
    refreshInterval,
    setNsqdAddress,
    setLookupdAddress,
    setRefreshInterval,
  } = useAppStore()

  const [formData, setFormData] = useState({
    nsqdAddress,
    lookupdAddress,
    refreshInterval,
  })

  const handleSave = () => {
    setNsqdAddress(formData.nsqdAddress)
    setLookupdAddress(formData.lookupdAddress)
    setRefreshInterval(formData.refreshInterval)
    toast.success('Settings saved successfully')
  }

  const handleReset = () => {
    setFormData({
      nsqdAddress: 'http://localhost:4151',
      lookupdAddress: 'http://localhost:4161',
      refreshInterval: 5000,
    })
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Settings
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Configure your NSQ Admin interface
          </p>
        </div>
      </div>

      {/* Settings Form */}
      <div className="card p-6">
        <div className="space-y-6">
          {/* Connection Settings */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Connection Settings
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  NSQd Address
                </label>
                <div className="relative">
                  <Server className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <input
                    type="text"
                    value={formData.nsqdAddress}
                    onChange={(e) => setFormData(prev => ({ ...prev, nsqdAddress: e.target.value }))}
                    placeholder="http://localhost:4151"
                    className="input pl-10"
                  />
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  HTTP address of your NSQd instance
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  NSQLookupd Address
                </label>
                <div className="relative">
                  <Globe className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <input
                    type="text"
                    value={formData.lookupdAddress}
                    onChange={(e) => setFormData(prev => ({ ...prev, lookupdAddress: e.target.value }))}
                    placeholder="http://localhost:4161"
                    className="input pl-10"
                  />
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  HTTP address of your NSQLookupd instance
                </p>
              </div>
            </div>
          </div>

          {/* Refresh Settings */}
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Refresh Settings
            </h3>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Refresh Interval
              </label>
              <div className="relative">
                <Clock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  type="number"
                  value={formData.refreshInterval}
                  onChange={(e) => setFormData(prev => ({ ...prev, refreshInterval: parseInt(e.target.value) || 5000 }))}
                  min="1000"
                  max="60000"
                  step="1000"
                  className="input pl-10"
                />
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                How often to refresh data (in milliseconds)
              </p>
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center justify-end space-x-4 pt-6 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={handleReset}
              className="btn-secondary"
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              Reset
            </button>
            <button
              onClick={handleSave}
              className="btn-primary"
            >
              <Save className="h-4 w-4 mr-2" />
              Save Settings
            </button>
          </div>
        </div>
      </div>

      {/* About */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          About NSQ Admin
        </h3>
        <div className="space-y-3 text-sm text-gray-600 dark:text-gray-400">
          <div className="flex justify-between">
            <span>Version:</span>
            <span className="font-medium">1.3.0</span>
          </div>
          <div className="flex justify-between">
            <span>Implementation:</span>
            <span className="font-medium">Rust</span>
          </div>
          <div className="flex justify-between">
            <span>UI Framework:</span>
            <span className="font-medium">React + TypeScript</span>
          </div>
          <div className="flex justify-between">
            <span>Styling:</span>
            <span className="font-medium">Tailwind CSS</span>
          </div>
        </div>
      </div>
    </div>
  )
}
