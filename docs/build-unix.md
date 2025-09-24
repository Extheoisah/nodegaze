# Linux/Unix Build Instructions

This guide covers building NodeGaze on Linux distributions and other Unix-like systems.

## Prerequisites

- A supported Linux distribution (Ubuntu, Debian, Fedora, Arch, etc.)
- Root or sudo access for package installation
- Internet connection for downloading dependencies

## Step 1: Install System Dependencies

### Ubuntu/Debian

```bash
# Update package list
sudo apt update && sudo apt upgrade -y

# Install essential build tools
sudo apt install -y build-essential curl git pkg-config libssl-dev

# Install SQLite
sudo apt install -y sqlite3 libsqlite3-dev

# Install Make
sudo apt install -y make
```

### Fedora/CentOS/RHEL

```bash
# Update package list
sudo dnf update -y  # or 'sudo yum update -y' on older systems

# Install essential build tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git pkg-config openssl-devel sqlite-devel make

# For CentOS/RHEL, you might need EPEL repository
sudo dnf install -y epel-release  # if not already installed
```

### Arch Linux

```bash
# Update system
sudo pacman -Syu

# Install dependencies
sudo pacman -S base-devel curl git pkgconf openssl sqlite make
```

### openSUSE

```bash
# Update system
sudo zypper refresh && sudo zypper update

# Install dependencies
sudo zypper install -t pattern devel_basis
sudo zypper install curl git pkg-config libopenssl-devel sqlite3-devel make
```

## Step 2: Install Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the Rust environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Step 3: Install Node.js

### Option 1: Using Package Manager (Ubuntu/Debian)

```bash
# Install Node.js via NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
```

### Option 2: Using Package Manager (Other Distributions)

```bash
# Fedora/CentOS/RHEL
sudo dnf install -y nodejs npm

# Arch Linux
sudo pacman -S nodejs npm

# openSUSE
sudo zypper install nodejs18 npm18
```

### Option 3: Using Node Version Manager (Recommended)

```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Reload your shell configuration
source ~/.bashrc  # or ~/.zshrc if using zsh

# Install and use Node.js 18
nvm install 18
nvm use 18
```

## Step 4: Install SQLx CLI

```bash
cargo install sqlx-cli
```

## Step 5: Clone the Repository

```bash
git clone https://github.com/Extheoisah/nodegaze.git
cd nodegaze
```

## Step 6: Environment Configuration

Set up backend environment:
```bash
cp .env.example .env
# Edit .env with your configuration
nano .env  # or vim, emacs, code, etc.
```

Set up frontend environment:
```bash
cd frontend
cp .env.example .env.local
# Edit .env.local with your configuration  
nano .env.local  # or your preferred editor
cd ..
```

## Step 7: Database Setup

Run the following commands in the project root directory:
```bash
# Run database migrations
sqlx migrate run --source backend/migrations

# Generate offline SQLx data (required for compilation)
cargo sqlx prepare --workspace
```

## Step 8: Build and Run the Backend

```bash
# Navigate to backend directory
cd backend

# Build the project
cargo build

# Run the backend server
cargo run
```

The backend server will start on the port specified in your `.env` file (default: `http://localhost:3030`)

## Step 9: Start the Frontend (Optional)

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

### Build Tool Issues
- **Ubuntu/Debian**: If you get compiler errors, install `build-essential`
- **Fedora/RHEL**: Install the "Development Tools" group
- **Arch**: Install `base-devel` package group

### Rust Issues
- If `cargo` is not found, restart your terminal or run `source ~/.cargo/env`
- Update Rust if needed: `rustup update`

### Node.js Issues
- Verify Node.js version: `node --version` (should be v18+)
- If using system packages, you might need to install `nodejs` and `npm` separately
- Consider using `nvm` for better version management

### SQLite Issues
- **Ubuntu/Debian**: Install `libsqlite3-dev`
- **Fedora/RHEL**: Install `sqlite-devel`
- **Arch**: Install `sqlite`

### Permission Issues
- Ensure you have write permissions in the project directory
- Avoid using `sudo` with `cargo` or `npm` commands when possible
- If needed, fix ownership: `sudo chown -R $USER:$USER ~/nodegaze`

### OpenSSL Issues
- **Ubuntu/Debian**: Install `libssl-dev`
- **Fedora/RHEL**: Install `openssl-devel`
- **Arch**: Install `openssl`

## Distribution-Specific Notes

### Alpine Linux
```bash
# Install additional packages for Alpine
sudo apk add musl-dev
```

### NixOS
```bash
# Use the provided nix-shell
nix-shell
```

### WSL (Windows Subsystem for Linux)
Follow the Ubuntu/Debian instructions, but ensure you're working within the WSL filesystem for better performance.

## Next Steps

Once both backend and frontend are running:
- Access the web interface at `http://localhost:3000` (or your configured frontend port)
- Backend API is available at `http://localhost:3030` (or your configured SERVER_PORT)
- Check the main README for configuration and usage instructions