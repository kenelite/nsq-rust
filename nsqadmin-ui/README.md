# NSQ Admin UI

Modern web interface for NSQ message queue management, built with React, TypeScript, and Tailwind CSS.

## Features

- **Real-time Dashboard**: Live statistics and monitoring
- **Topic Management**: Create, pause, unpause, and delete topics
- **Channel Monitoring**: View channel statistics and client connections
- **Node Status**: Monitor NSQd and NSQLookupd nodes
- **Dark Mode**: Toggle between light and dark themes
- **Responsive Design**: Works on desktop and mobile devices
- **Modern UI**: Clean, intuitive interface with smooth animations

## Technology Stack

- **React 18** with TypeScript
- **Vite** for fast development and building
- **Tailwind CSS** for styling
- **React Router** for navigation
- **Zustand** for state management
- **Axios** for API calls
- **Recharts** for data visualization
- **Lucide React** for icons
- **React Hot Toast** for notifications

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm or yarn
- NSQ services running (nsqd, nsqlookupd)

### Installation

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm run dev
```

3. Open your browser and navigate to `http://localhost:3000`

### Building for Production

```bash
npm run build
```

The built files will be in the `dist` directory.

## Configuration

The UI connects to NSQ services via HTTP APIs. Default configuration:

- **NSQd**: `http://localhost:4151`
- **NSQLookupd**: `http://localhost:4161`

You can change these settings in the Settings page of the UI.

## API Endpoints

The UI expects the following NSQ API endpoints to be available:

### NSQd Endpoints
- `GET /stats` - Cluster statistics
- `GET /topic/stats?topic={name}` - Topic statistics
- `GET /channel/stats?topic={name}&channel={name}` - Channel statistics
- `POST /topic/pause?topic={name}` - Pause topic
- `POST /topic/unpause?topic={name}` - Unpause topic
- `POST /topic/delete?topic={name}` - Delete topic
- `POST /channel/pause?topic={name}&channel={name}` - Pause channel
- `POST /channel/unpause?topic={name}&channel={name}` - Unpause channel
- `POST /channel/delete?topic={name}&channel={name}` - Delete channel

### NSQLookupd Endpoints
- `GET /stats` - Lookupd statistics
- `GET /topics` - List all topics
- `GET /channels?topic={name}` - List channels for topic
- `GET /nodes` - List registered nodes
- `GET /lookup?topic={name}` - Lookup producers for topic

## Development

### Project Structure

```
src/
├── components/          # React components
│   ├── Layout.tsx      # Main layout wrapper
│   ├── Dashboard.tsx    # Dashboard page
│   ├── Topics.tsx      # Topics management
│   ├── Nodes.tsx       # Node monitoring
│   └── Settings.tsx    # Settings page
├── hooks/              # Custom React hooks
│   └── useStats.ts     # Stats data fetching
├── stores/             # State management
│   └── useAppStore.ts  # Global app state
├── types/              # TypeScript type definitions
│   └── index.ts        # NSQ types
├── utils/              # Utility functions
│   ├── api.ts          # API client
│   └── cn.ts           # Class name utility
├── App.tsx             # Main app component
└── main.tsx            # App entry point
```

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript type checking

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is part of the NSQ Rust implementation and follows the same license terms.
