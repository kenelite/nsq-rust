import { Moon, Sun, Settings, RefreshCw } from 'lucide-react'
import { useAppStore } from '../stores/useAppStore'
import { cn } from '../utils/cn'

export function Header() {
  const { isDarkMode, toggleDarkMode, refreshInterval } = useAppStore()

  return (
    <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
            NSQ Admin
          </h1>
          <span className="text-sm text-gray-500 dark:text-gray-400">
            v1.3.0
          </span>
        </div>
        
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
            <RefreshCw className="h-4 w-4" />
            <span>Refresh: {refreshInterval / 1000}s</span>
          </div>
          
          <button
            onClick={toggleDarkMode}
            className={cn(
              "p-2 rounded-md transition-colors",
              "hover:bg-gray-100 dark:hover:bg-gray-700",
              "text-gray-500 dark:text-gray-400"
            )}
            title={isDarkMode ? "Switch to light mode" : "Switch to dark mode"}
          >
            {isDarkMode ? (
              <Sun className="h-5 w-5" />
            ) : (
              <Moon className="h-5 w-5" />
            )}
          </button>
          
          <button
            className={cn(
              "p-2 rounded-md transition-colors",
              "hover:bg-gray-100 dark:hover:bg-gray-700",
              "text-gray-500 dark:text-gray-400"
            )}
            title="Settings"
          >
            <Settings className="h-5 w-5" />
          </button>
        </div>
      </div>
    </header>
  )
}
