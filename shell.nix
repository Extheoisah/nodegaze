{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust development tools
    bacon          # Continuous testing/checking for Rust
    sqlx-cli       # SQLx command-line interface for database management
    # Database tools (useful with sqlx-cli)
    sqlite         # SQLite database
    # OpenSSL and build tools
    openssl        # OpenSSL library
    pkg-config     # Required for finding system libraries

  ];

  shellHook = ''
    echo "🦀 Rust development environment loaded!"
    echo "Available tools:"
    echo "  - bacon: continuous testing and checking"
    echo "  - sqlx: database CLI (try 'sqlx --help')"
    echo "  - sqlite, database clients"
    echo "  - openssl, pkg-config for system library linking"
  '';
}
