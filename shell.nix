{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust development tools
    bacon          # Continuous testing/checking for Rust
    sqlx-cli       # SQLx command-line interface for database management

    # Node.js and frontend tools
    nodejs_22     # Node.js 22 LTS
    npm            # Node package manager

    # Database tools (useful with sqlx-cli)
    sqlite         # SQLite database
    sqlite-utils   # Enhanced SQLite CLI utilities

    # System dependencies and build tools
    openssl        # OpenSSL library
    pkg-config     # Required for finding system libraries
    protobuf       # Protocol Buffers library
    gnumake        # Make build tool
  ];

  shellHook = ''
    echo "🦀 NodeGaze development environment loaded!"
    echo ""
    echo "Available tools:"
    echo "  - rustc: Rust compiler"
    echo "  - cargo: Rust package manager"
    echo "  - bacon: continuous testing and checking for Rust"
    echo "  - sqlx: database CLI (try 'sqlx --help')"
    echo "  - node $(node --version): Node.js runtime"
    echo "  - npm $(npm --version): Node package manager"
    echo "  - sqlite: SQLite database"
    echo "  - make: Build automation tool"
    echo ""
    echo "Getting started:"
    echo "  1. Copy .env.example to .env and configure"
    echo "  2. Copy frontend/.env.example to frontend/.env.local"
    echo "  3. Run 'make dev' to start both backend and frontend"
    echo "  4. Or run backend and frontend separately:"
    echo "     - Backend: cd backend && cargo run"
    echo "     - Frontend: cd frontend && npm install && npm run dev"
    echo ""

    # Set helpful environment variables
    export RUST_LOG=info
    export DATABASE_URL="sqlite:backend/nodegaze.db"
  '';
}
