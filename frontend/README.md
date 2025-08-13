# NodeGaze Frontend 🎯

The modern web interface for NodeGaze, built with Next.js 14, React, and TypeScript. Provides a comprehensive dashboard for monitoring Lightning Network nodes with real-time event tracking and notification management.

## 🚀 Features

### Dashboard & Monitoring

- **Real-time Event Streaming**: Live updates of Lightning Network events
- **Event Filtering**: Filter by event type (Invoice, Channel operations) and severity
- **Event History**: Paginated event lists with detailed information
- **Node Information**: Display node aliases, IDs, and connection status

### Notification Management

- **Webhook Configuration**: Set up HTTP endpoints for event notifications
- **Discord Integration**: Send formatted alerts to Discord channels
- **Event Type Selection**: Choose which events to forward to each notification endpoint
- **Notification Status**: Track delivery status and retry failed notifications

### User Experience

- **Responsive Design**: Works seamlessly on desktop, tablet, and mobile
- **Modern UI**: Clean interface built with shadcn/ui components
- **Authentication**: Secure login/signup with NextAuth.js integration
- **Dark/Light Theme**: Support for user preference themes (coming soon)

## 🛠️ Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **UI Components**: shadcn/ui
- **Authentication**: NextAuth.js
- **State Management**: React Hooks + Context
- **HTTP Client**: Native fetch API
- **Icons**: Lucide React

## 🚀 Getting Started

### Prerequisites

- Node.js 18+
- npm, yarn, or pnpm
- NodeGaze backend running on `http://localhost:3030`

### Installation

1. **Install dependencies**

   ```bash
   npm install
   # or
   yarn install
   # or
   pnpm install
   ```

2. **Environment Setup**

   ```bash
   cp .env.example .env.local
   ```

   Configure your environment variables:

   ```bash
   # API Configuration
   BACKEND_URL=http://localhost:3030
   NEXTAUTH_URL=http://localhost:3000
   NEXTAUTH_SECRET=your-nextauth-secret-key
   
   # JWT Configuration
   NEXTAUTH_JWT_SECRET=your-jwt-secret-key
   ```

3. **Start Development Server**

   ```bash
   npm run dev
   # or
   yarn dev
   # or
   pnpm dev
   ```

4. **Open in Browser**
   Navigate to [http://localhost:3000](http://localhost:3000)

## 🎨 UI Components

The project uses [shadcn/ui](https://ui.shadcn.com/) for consistent, accessible components:

- **Layout**: Sidebar navigation, responsive header
- **Forms**: Input fields, selects, buttons with validation
- **Data Display**: Tables, cards, badges, tooltips
- **Navigation**: Breadcrumbs, pagination, links
- **Feedback**: Loading states, error messages, success notifications

## 🔧 Development

### Available Scripts

```bash
# Development
npm run dev          # Start development server
npm run build        # Build for production
npm run start        # Start production server

# Code Quality
npm run lint         # Run ESLint
npm run lint:fix     # Fix ESLint issues automatically
npm run type-check   # Run TypeScript type checking

# Testing (if configured)
npm run test         # Run tests
npm run test:watch   # Run tests in watch mode
```

### Code Style

The project follows these conventions:

- **TypeScript**: Strict mode enabled with comprehensive types
- **ESLint**: Extended Next.js configuration with custom rules
- **Prettier**: Automatic code formatting
- **File Naming**: kebab-case for files, PascalCase for components
- **Import Order**: External libraries → internal modules → relative imports

### API Integration

The frontend communicates with the NodeGaze backend through:

- **Authentication**: NextAuth.js with custom JWT provider
- **API Routes**: Next.js API routes for backend proxy
- **Error Handling**: Comprehensive error states and user feedback
- **Type Safety**: Full TypeScript coverage for API responses

## 🚢 Deployment

### Production Build

```bash
npm run build
npm run start
```

### Environment Variables

Ensure these are set in your production environment:

```bash
BACKEND_URL=https://your-backend-domain.com
NEXTAUTH_URL=https://your-frontend-domain.com
NEXTAUTH_SECRET=production-secret-key
NEXTAUTH_JWT_SECRET=production-jwt-secret
```

### Deployment Platforms

- **Vercel**: Optimized for Next.js applications
- **Netlify**: Support for static exports and serverless functions
- **Docker**: Container-ready with proper multi-stage builds
- **Self-hosted**: Works with any Node.js hosting provider

## 🤝 Contributing

1. Follow the existing code style and conventions
2. Add TypeScript types for new features
3. Test your changes across different screen sizes
4. Update component documentation if needed
5. Ensure accessibility standards are met

## 📚 Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [React Documentation](https://react.dev)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [shadcn/ui Components](https://ui.shadcn.com)
- [NextAuth.js Guide](https://next-auth.js.org)

---

**NodeGaze Frontend** - *Modern Lightning Network Monitoring Interface* ⚡
