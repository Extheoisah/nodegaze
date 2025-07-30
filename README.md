# NodeGaze
NodeGaze: Unveiling Your Node's Universe.

NodeGaze is an open-source observability tool for Lightning node operators. Get unparalleled, real-time insight into your node's performance, channel health, and logs. It's implementation-agnostic (LND, c-lightning, Eclair, LDK) and offers a powerful Event Propagation API for real-time alerts and external integrations.
NodeGaze helps you truly see, understand, and master your Lightning node, setting the foundation for future provisioning services.

## Features

- **Multi-Implementation Support**: Works with LND, c-lightning, Eclair, and LDK
- **Real-time Monitoring**: Live insights into node performance and channel health
- **Event Propagation API**: Real-time alerts and external integrations
- **Observability Dashboard**: Comprehensive view of your Lightning node's universe
- **Open Source**: Fully open-source with community-driven development

## Development Setup

### Prerequisites

- Rust toolchain
- SQLite
- Make
- Polar
- Docker (Polar depends on docker)

### Getting Started

#### You will need sqlx-cli for db operations, for db operations, you can install it using `cargo install sqlx-cli`

1. **Clone the repository**
   ```bash
   git clone https://github.com/Extheoisah/nodegaze.git
   cd nodegaze
   ```

2. **Set up environment variables**
   ```bash
   # Create environment file by copying the example, make sure to update the values.
   cp .env.example .env
   ```

3. **Quick setup and run**
   ```bash
   # Complete setup (database creation, migrations, and SQLx preparation) then run
   make dev
   ```

   **Or step by step:**
   ```bash
   # Set up database only
   make setup

   # Run the project
   make run
   ```

### Manual Database Management

The project uses SQLite with SQLx for database operations. Manual commands:

- **Run migrations**: `sqlx migrate run --source backend/migrations`
- **Create new migration**: `sqlx migrate add <migration_name> --source backend/migrations`
- **Reset database**: `sqlx database drop && sqlx database create`
- **Generate offline data**: `cargo sqlx prepare --workspace`

### Environment Variables

Copy `.env.example` to `.env` and configure:

- `DATABASE_URL`: SQLite database path
- `DB_MAX_CONNECTIONS`: Maximum database connections (default: 5)
- `DB_ACQUIRE_TIMEOUT_SECONDS`: Connection timeout (default: 3)
- `JWT_SECRET`: Secret key for JWT token generation
- `JWT_EXPIRES_IN_SECONDS`: JWT token expiration time (default: 86400)
- `SERVER_PORT`: Server port (default: 3000)
- `ENCRYPTION_KEY`: Key for data encryption

### Development Tools

The Nix shell provides:
> This is optional if you are not on linux or not familiar with Nix

- **bacon**: Continuous testing and checking
- **sqlx-cli**: Database migrations and management
- **rust-analyzer**: LSP server for IDE support

## Contributing

1. Clone the repository or fork it
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
