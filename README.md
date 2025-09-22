# NodeGaze 🌟
*Unveiling Your Node's Universe*

NodeGaze is an open-source observability tool for Lightning node operators. Get unparalleled, real-time insight into your node's performance, channel health, and logs. It's implementation-agnostic (LND, c-lightning, Eclair, LDK) and offers a powerful Event Propagation API for real-time alerts and external integrations.
NodeGaze helps you truly see, understand, and master your Lightning node, setting the foundation for future provisioning services.

## 🚀 Features

### Core Monitoring
- **Real-time Event Tracking**: Monitor invoice creation/settlement, channel operations, and network events
- **Multi-Node Support**: Manage and monitor multiple Lightning nodes from a single dashboard
- **Event History**: Comprehensive logging and filtering of all node activities
- **Performance Metrics**: Track node performance, channel health, and transaction flows

### Notification System
- **Webhook Integration**: Send real-time events to external services via HTTP webhooks
- **Discord Notifications**: Direct integration with Discord channels for team alerts
- **Event Filtering**: Configure notifications based on event types and severity levels
- **Retry Logic**: Automatic retry for failed notification deliveries

### User Experience
- **Modern Web Interface**: Clean, responsive dashboard built with Next.js and React
- **Authentication & Security**: Secure user authentication with JWT tokens
- **Multi-tenant Architecture**: Support for multiple users and organizations
- **Real-time Updates**: Live event streaming and dashboard updates

### Developer-Friendly
- **RESTful API**: Comprehensive API for integrations and custom applications
- **Implementation Agnostic**: Designed to work with LND, CLN, Eclair, and LDK
- **Open Source**: MIT licensed with community-driven development
- **Docker Support**: Easy deployment with containerization

## 🏗️ Architecture

NodeGaze consists of two main components:

- **Backend (Rust)**: High-performance API server with SQLite database
  - Authentication and user management
  - Event processing and storage
  - Notification delivery system
  - RESTful API endpoints

- **Frontend (Next.js)**: Modern web interface
  - Server-side rendering for optimal performance
  - Real-time event streaming
  - Responsive design for desktop and mobile
  - Authentication integration

## 🚀 Quick Start

### Prerequisites

- **Rust toolchain** (latest stable)
- **Node.js** (v18+ recommended) & **npm/yarn**
- **SQLite**
- **Make** (for build automation)
- **sqlx-cli**: `cargo install sqlx-cli`
- **Docker** (optional for Polar testing)
- **Polar** (for local development & testing)

### Build Instructions

Choose your platform for detailed setup instructions:

- **[Windows (WSL)](docs/build-windows.md)** - Setup using Windows Subsystem for Linux
- **[macOS](docs/build-macos.md)** - Setup on macOS systems
- **[Linux](docs/build-unix.md)** - Setup on Linux distributions

### Quick Development Setup (Make Required)

If you already have all prerequisites installed and Make available:

1. **Clone the repository**
   ```bash
   git clone https://github.com/Extheoisah/nodegaze.git
   cd nodegaze
   ```

2. **Environment Setup**
   ```bash
   # Backend environment
   cp .env.example .env
   # Edit .env with your configuration
   
   # Frontend environment
   cd frontend
   cp .env.example .env.local
   # Edit .env.local with your configuration
   cd ..
   ```

3. **Complete setup and run**
   ```bash
   make dev
   ```
   
   **Or step by step:**
   ```bash
   # 1. Set up database
   make setup
   
   # 2. Run backend (terminal 1)
   make run
   
   # 3. Run frontend (terminal 2)
   cd frontend && npm install && npm run dev
   ```

4. **Access the application**
   - Frontend: <http://localhost:3000> (or port specified in frontend/.env.local)
   - Backend API: <http://localhost:3030> (or SERVER_PORT in .env)

### Manual Database Management

The project uses SQLite with SQLx for database operations. Manual commands:

- **Run migrations**: `sqlx migrate run --source backend/migrations`
- **Create new migration**: `sqlx migrate add <migration_name> --source backend/migrations`
- **Reset database**: `sqlx database drop && sqlx database create`
- **Generate offline data**: `cargo sqlx prepare --workspace`

## ⚙️ Configuration

### Backend Environment Variables

Copy `.env.example` to `.env` and configure:

#### Database Configuration
- `DATABASE_URL`: SQLite database path (default: sqlite:nodegaze.db)
- `DB_MAX_CONNECTIONS`: Maximum database connections (default: 5)
- `DB_ACQUIRE_TIMEOUT_SECONDS`: Connection timeout (default: 3)

#### Security & Authentication
- `ENCRYPTION_KEY`: Key for sensitive data encryption (32 bytes base64 encoded)
- `JWT_SECRET`: Secret key for JWT token generation
- `JWT_EXPIRES_IN_SECONDS`: JWT token expiration time (default: 86400)

#### Server Configuration
- `SERVER_PORT`: Backend server port (default: 3030)
- `BASE_URL`: Frontend base URL for backend communication (default: http://localhost:3000)

#### Email Configuration (SMTP)
- `SMTP_HOST`: SMTP server hostname
- `SMTP_PORT`: SMTP server port (default: 587)
- `SMTP_USERNAME`: SMTP authentication username
- `SMTP_PASSWORD`: SMTP authentication password (use app passwords for Gmail)
- `FROM_EMAIL`: Email address for outgoing emails
- `FROM_NAME`: Display name for outgoing emails

#### Logging
- `RUST_LOG`: Logging level (default: info, options: error, warn, info, debug, trace)

### Frontend Environment Variables

Copy `frontend/.env.example` to `frontend/.env.local` and configure as needed for your Next.js application.

## 🛠️ Development Tools

The Nix shell provides:
> This is optional if you are not on linux or not familiar with Nix

The Nix shell provides a complete development environment:

- **bacon**: Continuous testing and checking
- **sqlx-cli**: Database migrations and management  
- **rust-analyzer**: LSP server for IDE support

```bash
nix-shell  # Enter development environment
```

### Useful Commands

```bash
# Backend development
make setup      # Initialize database
make run        # Run backend server
make test       # Run tests
make format     # Format code

# Frontend development
cd frontend
npm install     # Install dependencies
npm run dev     # Start development server
npm run build   # Build for production
npm run lint    # Run linting
```

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository** and clone your fork
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests if applicable
4. **Run tests**: `make test` (backend) and `npm test` (frontend)
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to your branch**: `git push origin feature/amazing-feature`
7. **Open a Pull Request** with a clear description of your changes

### Development Guidelines

- Follow Rust naming conventions and use `cargo fmt`
- Use TypeScript for all new frontend code
- Add tests for new functionality
- Update documentation for user-facing changes
- Ensure CI passes before requesting review

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋‍♂️ Support

- **Issues**: Report bugs or request features via [GitHub Issues](https://github.com/Extheoisah/nodegaze/issues)
- **Discussions**: Join the conversation in [GitHub Discussions](https://github.com/Extheoisah/nodegaze/discussions)

---

**NodeGaze** - *Unveiling Your Node's Universe* 🌟