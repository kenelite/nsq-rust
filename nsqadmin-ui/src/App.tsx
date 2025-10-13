import { useEffect } from 'react'
import { Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Dashboard } from './components/Dashboard'
import { Topics } from './components/Topics'
import { Nodes } from './components/Nodes'
import { Settings } from './components/Settings'
import { Channels } from './components/Channels'
import { Performance } from './components/Performance'
import { useAppStore } from './stores/useAppStore'

function App() {
  const { isDarkMode } = useAppStore()

  useEffect(() => {
    if (isDarkMode) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDarkMode])

  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Dashboard />} />
        <Route path="topics" element={<Topics />} />
        <Route path="nodes" element={<Nodes />} />
        <Route path="channels" element={<Channels />} />
        <Route path="performance" element={<Performance />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  )
}

export default App
