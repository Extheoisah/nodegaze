# macOS Build Instructions

This guide covers building NodeGaze on macOS systems.

## Prerequisites

- Command Line Tools for Xcode
- Administrator access to install dependencies

## Step 1: Install Command Line Tools

If you don't have Xcode installed, install the command line tools:

```bash
xcode-select --install
```

## Step 2: Install Homebrew

If you don't have Homebrew installed:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the instructions to add Homebrew to your PATH.

## Step 3: Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install other dependencies via Homebrew
brew install node sqlite make

# Install SQLx CLI
cargo install sqlx-cli
```

## Step 4: Clone the Repository

```bash
git clone https://github.com/Extheoisah/nodegaze.git
cd nodegaze
```

## Step 5: Environment Configuration

Set up backend environment:
```bash
cp .env.example .env
# Edit .env with your configuration
nano .env  # or use your preferred editor (vim, code, etc.)
```

Set up frontend environment:
```bash
cd frontend
cp .env.example .env.local
# Edit .env.local with your configuration  
nano .env.local  # or use your preferred editor
cd ..
```

## Step 6: Database Setup

Run the following commands in the project root directory:
```bash
# Run database migrations
sqlx migrate run --source backend/migrations

# Generate offline SQLx data (required for compilation)
cargo sqlx prepare --workspace
```

## Step 7: Build and Run the Backend

```bash
# Navigate to backend directory
cd backend

# Build the project
cargo build

# Run the backend server
cargo run
```

The backend server will start on the port specified in your `.env` file (default: `http://localhost:3030`)

## Step 8: Start the Frontend (Optional)

Open a new terminal tab/window and run:
```bash
# Navigate to frontend directory (from project root)
cd frontend

# Install dependencies
npm install

# Start the development server
npm run dev
```

The frontend will be available at `http://localhost:3000` (or the port specified in your frontend/.env.local)

## Alternative: Using Make (Recommended)

If you prefer using the automated setup:

```bash
# After cloning and setting up environment files
make dev
```

This will handle database setup, build, and run both backend and frontend.

## Troubleshooting

### Rust Installation Issues
- If `cargo` command is not found, restart your terminal or run `source ~/.cargo/env`
- Ensure Rust is properly installed: `rustc --version`

### Homebrew Issues
- If commands are not found, ensure Homebrew is in your PATH
- Run `brew doctor` to check for common issues

### Node.js Issues
- Verify Node.js installation: `node --version` (should be v18+)
- If you have multiple Node versions, consider using `nvm`

### SQLite Issues
- Verify SQLite installation: `sqlite3 --version`
- Ensure you have write permissions in the project directory

### Permission Issues
- If you encounter permission errors, avoid using `sudo` with npm
- Consider using a Node version manager like `nvm`

## Development Environment

### Recommended Tools
- **VS Code** with Rust and TypeScript extensions
- **Terminal** with multiple tabs for backend/frontend
- **Homebrew** for package management

### Performance Tips
- Use `cargo build --release` for optimized builds
- Consider using `cargo-watch` for automatic rebuilds during development

## Next Steps

Once both backend and frontend are running:
- Access the web interface at `http://localhost:3000` (or your configured frontend port)
- Backend API is available at `http://localhost:3030` (or your configured SERVER_PORT)
- Check the main README for configuration and usage instructions