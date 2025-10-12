import { useState, useEffect } from 'react'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'
import { format } from 'date-fns'

interface MessageRateData {
  timestamp: number
  rate: number
  count: number
}

export function MessageRateChart() {
  const [data, setData] = useState<MessageRateData[]>([])

  useEffect(() => {
    // Generate mock data for demonstration
    const generateData = () => {
      const now = Date.now()
      const points: MessageRateData[] = []
      
      for (let i = 29; i >= 0; i--) {
        const timestamp = now - (i * 1000) // 1 second intervals
        const rate = Math.random() * 1000 + 500 // Random rate between 500-1500
        const count = Math.floor(rate * 0.1) // Count is proportional to rate
        
        points.push({
          timestamp,
          rate: Math.round(rate),
          count,
        })
      }
      
      return points
    }

    setData(generateData())
    
    // Update data every second
    const interval = setInterval(() => {
      setData(prevData => {
        const newData = [...prevData.slice(1)] // Remove oldest point
        const now = Date.now()
        const rate = Math.random() * 1000 + 500
        const count = Math.floor(rate * 0.1)
        
        newData.push({
          timestamp: now,
          rate: Math.round(rate),
          count,
        })
        
        return newData
      })
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  const formatTime = (timestamp: number) => {
    return format(new Date(timestamp), 'HH:mm:ss')
  }

  return (
    <div className="h-64">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
          <XAxis 
            dataKey="timestamp" 
            tickFormatter={formatTime}
            tick={{ fontSize: 12 }}
          />
          <YAxis tick={{ fontSize: 12 }} />
          <Tooltip 
            labelFormatter={(value) => formatTime(value as number)}
            formatter={(value: number, name: string) => [
              value.toLocaleString(),
              name === 'rate' ? 'Messages/sec' : 'Total Messages'
            ]}
          />
          <Line 
            type="monotone" 
            dataKey="rate" 
            stroke="#3b82f6" 
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
