# NSQ Admin UI Demo

## Overview

The NSQ Admin UI is a modern, responsive web interface for managing NSQ message queues. Built with React, TypeScript, and Tailwind CSS, it provides a clean and intuitive interface for monitoring and managing your NSQ cluster.

## Features Demonstrated

### 🎨 Modern Design
- **Dark/Light Mode**: Toggle between themes with a single click
- **Responsive Layout**: Works seamlessly on desktop, tablet, and mobile
- **Clean UI**: Modern design with smooth animations and transitions
- **Accessible**: Built with accessibility best practices

### 📊 Real-time Dashboard
- **Live Statistics**: Real-time message rates and cluster statistics
- **Interactive Charts**: Message rate visualization with Recharts
- **Node Status**: Monitor NSQd and NSQLookupd nodes
- **Topic Overview**: Quick overview of all topics and their status

### 🔧 Topic Management
- **Topic Operations**: Create, pause, unpause, and delete topics
- **Channel Management**: Manage channels within topics
- **Search & Filter**: Find topics quickly with search and filters
- **Status Indicators**: Visual indicators for paused/active topics

### 🖥️ Node Monitoring
- **Node Status**: Real-time status of all NSQ nodes
- **Connection Info**: Detailed connection information
- **Health Checks**: Monitor node health and connectivity
- **Version Information**: Track NSQ versions across nodes

### ⚙️ Settings & Configuration
- **Connection Settings**: Configure NSQd and NSQLookupd addresses
- **Refresh Intervals**: Customize data refresh rates
- **Persistent Settings**: Settings saved across sessions
- **About Information**: Version and build information

## Technical Implementation

### Frontend Stack
- **React 18** with TypeScript for type safety
- **Vite** for fast development and optimized builds
- **Tailwind CSS** for utility-first styling
- **Zustand** for lightweight state management
- **React Router** for client-side routing
- **Axios** for HTTP API communication
- **Recharts** for data visualization
- **Lucide React** for consistent iconography

### Backend Integration
- **Rust Backend**: NSQAdmin server built with Axum
- **REST API**: Clean REST endpoints for all operations
- **Static File Serving**: Serves the built React app
- **Mock Data**: Demonstrates real-time data updates
- **Error Handling**: Comprehensive error handling and user feedback

### Key Components

#### Dashboard (`Dashboard.tsx`)
- Real-time statistics cards
- Message rate chart with live updates
- Topic overview table
- Node status monitoring

#### Topics (`Topics.tsx`)
- Grid layout for topic cards
- Search and filter functionality
- Topic management actions
- Status indicators and counts

#### Nodes (`Nodes.tsx`)
- Node status cards
- Connection information
- Health monitoring
- Version tracking

#### Settings (`Settings.tsx`)
- Connection configuration
- Refresh interval settings
- Theme preferences
- About information

## Getting Started

### Development
```bash
cd nsqadmin-ui
npm install
npm run dev
```

### Production Build
```bash
npm run build
```

### Backend Integration
The NSQAdmin Rust server serves the built React app and provides API endpoints for:
- Statistics and monitoring
- Topic and channel management
- Node status and health
- Configuration and settings

## UI Screenshots

### Dashboard View
- Real-time statistics overview
- Message rate visualization
- Topic and node status
- Clean, modern interface

### Topics Management
- Grid layout for easy browsing
- Search and filter capabilities
- Quick action buttons
- Status indicators

### Node Monitoring
- Node status cards
- Connection details
- Health indicators
- Version information

### Settings Panel
- Connection configuration
- Theme preferences
- Refresh settings
- About information

## Responsive Design

The UI is fully responsive and works on:
- **Desktop**: Full-featured interface with sidebar navigation
- **Tablet**: Optimized layout with collapsible sidebar
- **Mobile**: Mobile-first design with touch-friendly controls

## Dark Mode Support

- **Automatic Detection**: Respects system preferences
- **Manual Toggle**: User can override system settings
- **Persistent**: Theme preference saved across sessions
- **Smooth Transitions**: Animated theme switching

## Performance Features

- **Code Splitting**: Optimized bundle sizes
- **Lazy Loading**: Components loaded on demand
- **Efficient Updates**: Minimal re-renders with React 18
- **Optimized Assets**: Compressed images and fonts
- **Caching**: Intelligent API response caching

## Future Enhancements

- **Real-time WebSocket**: Live updates without polling
- **Advanced Filtering**: More sophisticated search and filter options
- **Export Features**: Data export capabilities
- **Alerting**: Notification system for important events
- **Performance Metrics**: Detailed performance analytics
- **Multi-cluster Support**: Manage multiple NSQ clusters

## Conclusion

The NSQ Admin UI provides a modern, feature-rich interface for NSQ cluster management. With its clean design, real-time capabilities, and comprehensive feature set, it offers a significant improvement over traditional NSQ admin interfaces while maintaining full compatibility with NSQ's HTTP API.
