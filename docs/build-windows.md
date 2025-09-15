# Windows Build Instructions

This guide covers building NodeGaze on Windows using WSL (Windows Subsystem for Linux).

## Prerequisites

- Administrator access to install WSL

## Step 1: Install WSL2 with Ubuntu

Open PowerShell as Administrator and run:

```powershell
# Install WSL2 with Ubuntu
wsl --install -d Ubuntu
```

If WSL is already installed, ensure you have Ubuntu:
```powershell
# List available distributions
wsl --list --online

# Install Ubuntu if not present
wsl --install -d Ubuntu
```

Restart your computer if prompted, then open Ubuntu from the Start menu and complete the initial setup (create username and password).

## Step 2: Install Dependencies in Ubuntu

Open your Ubuntu terminal and run the following commands:

```bash
# Update package list
sudo apt update && sudo apt upgrade -y

# Install essential build tools
sudo apt install -y build-essential curl git pkg-config libssl-dev

# Install SQLite
sudo apt install -y sqlite3 libsqlite3-dev

# Install Make
sudo apt install -y make

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc

# Install Node.js (using NodeSource repository)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Install SQLx CLI
cargo install sqlx-cli
```

## Step 3: Clone the Repository

**Important**: Clone the repository in the Ubuntu filesystem (not Windows filesystem) for better performance:

```bash
cd ~
git clone https://github.com/Extheoisah/nodegaze.git
cd nodegaze
```

## Step 4: Environment Configuration

Set up backend environment:
```bash
cp .env.example .env
# Edit .env with your configuration
nano .env  # or use your preferred editor
```

Set up frontend environment:
```bash
cd frontend
cp .env.example .env.local
# Edit .env.local with your configuration  
nano .env.local  # or use your preferred editor
cd ..
```

## Step 5: Database Setup

Run the following commands in the project root directory:
```bash
# Run database migrations
sqlx migrate run --source backend/migrations

# Generate offline SQLx data (required for compilation)
cargo sqlx prepare --workspace
```

## Step 6: Build and Run the Backend

```bash
# Navigate to backend directory
cd backend

# Build the project
cargo build

# Run the backend server
cargo run
```

The backend server will start on the port specified in your `.env` file (default: `http://localhost:3030`)

## Step 7: Start the Frontend (Optional)

Open a new Ubuntu terminal tab/window and run:
```bash
# Navigate to frontend directory (from project root)
cd ~/nodegaze/frontend

# Install dependencies
npm install

# Start the development server
npm run dev
```

The frontend will be available at `http://localhost:3000` (or the port specified in your frontend/.env.local)

## Troubleshooting

### Performance Issues
- Always work within the Ubuntu filesystem (`/home/username/`) rather than Windows filesystem (`/mnt/c/`)
- This significantly improves file I/O performance

### WSL Integration
- You can access your Ubuntu files from Windows Explorer at `\\wsl$\Ubuntu\home\yourusername\`
- VS Code works well with WSL through the Remote-WSL extension

### Common Issues
- If Rust commands aren't found, run `source ~/.bashrc` or restart your terminal
- If Node.js installation fails, try updating package lists with `sudo apt update`
- For permission issues, ensure you're working in your home directory (`~/`)

## Next Steps

Once both backend and frontend are running:
- Access the web interface at `http://localhost:3000` (or your configured frontend port)
- Backend API is available at `http://localhost:3030` (or your configured SERVER_PORT)
- Check the main README for configuration and usage instructions