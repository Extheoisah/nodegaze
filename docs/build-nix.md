# Nix Build Instructions

This guide covers building NodeGaze using Nix and NixOS.

## Prerequisites

- Nix package manager installed (works on Linux, macOS, and WSL)
- Internet connection for downloading dependencies
- Basic familiarity with Nix concepts

## Installation Methods

### Method 1: Using the Development Shell (Recommended)

The project includes a `shell.nix` file that provides all necessary dependencies.

#### Step 1: Enter the Nix Shell

Clone the repository first:

```bash
git clone https://github.com/Extheoisah/nodegaze.git
```

```bash
cd nodegaze
```

Enter the development shell:

```bash
nix-shell
```

This will automatically install and make available:
- Rust toolchain via rustup
- SQLx CLI for database management
- SQLite database
- Node.js and npm
- OpenSSL and build tools
- Protocol Buffers

#### Step 2: Install Rust (if not already installed)

Install Rust using rustup (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
source ~/.cargo/env
```

#### Step 3: Environment Configuration

Set up backend environment:

```bash
cp .env.example .env
```

Edit .env with your configuration:

```bash
$EDITOR .env
```

Set up frontend environment:

```bash
cd frontend
```

```bash
cp .env.example .env.local
```

Edit .env.local with your configuration:

```bash
$EDITOR .env.local
```

```bash
cd ..
```

#### Step 4: Database Setup

Run database migrations:

```bash
sqlx migrate run --source backend/migrations
```

Generate offline SQLx data:

```bash
cargo sqlx prepare --workspace
```

#### Step 5: Build and Run

Using the provided Makefile (recommended):

```bash
make dev
```

Or manually - Backend:

```bash
cd backend && cargo run &
```

Frontend (in new terminal):

```bash
cd frontend && npm install && npm run dev
```

### Method 2: Using Nix Flakes (Advanced)

If you prefer using Nix flakes, you can create a `flake.nix` file:

```nix
{
  description = "NodeGaze development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            nodejs_22
            npm
            sqlx-cli
            sqlite
            bacon
            openssl
            pkg-config
            protobuf
            gnumake
            git
          ];

          shellHook = ''
            echo "🦀 NodeGaze Nix development environment loaded!"
            echo "Available tools:"
            echo "  - rustc $(rustc --version)"
            echo "  - node $(node --version)"
            echo "  - npm $(npm --version)"
            echo "  - sqlx-cli for database management"
            echo "  - bacon for continuous testing"
            echo ""
            echo "Run 'make dev' to start both backend and frontend"
          '';
        };
      });
}
```

Then use:

Enter development shell:

```bash
nix develop
```

Or run commands directly:

```bash
nix develop -c make dev
```

## Troubleshooting

### Common Issues

#### Rust not found after entering nix-shell

If Rust is not available, install it manually:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
source ~/.cargo/env
```

#### Node.js version issues

The shell.nix provides Node.js 22. If you need a different version, modify the shell.nix:

```nix
buildInputs = with pkgs; [
  nodejs_20
];
```

#### SQLite/SQLx issues

Ensure SQLx CLI is properly installed.

Check if sqlx is available:

```bash
which sqlx
```

If not, install manually:

```bash
cargo install sqlx-cli
```

#### OpenSSL linking errors

This is usually resolved by the nix shell providing the correct pkg-config paths. If issues persist:

Check if pkg-config can find OpenSSL:

```bash
pkg-config --libs openssl
```

Set environment variables if needed:

```bash
export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
```

#### Database migration errors

Ensure the database file exists and permissions are correct:

```bash
ls -la backend/
```

If it exists:

```bash
chmod 644 backend/nodegaze.db
```

Reset migrations if needed:

```bash
make setup
```

### Performance Tips

1. **Use binary cache**: Nix will download pre-built binaries when possible
2. **Enable flakes**: For faster dependency resolution

## Advanced Configuration

### Custom Shell Environment

You can customize the shell.nix for your specific needs.

## Next Steps

Once your environment is set up:

1. **Development**: Run `make dev` to start both services
2. **Testing**: Use `bacon` for continuous testing in the backend
3. **Database**: Use `sqlx` for database management
4. **Frontend**: Access the app at `http://localhost:3000`
5. **Backend API**: Available at `http://localhost:3030`

## Resources

- [Nix Manual](https://nixos.org/manual/nix/stable/)
- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [Nix Pills](https://nixos.org/guides/nix-pills/) - Great learning resource
- [NodeGaze Main Documentation](../README.md)

For additional help, consult the main project documentation or open an issue on the repository.
