# NodeGaze Makefile

.PHONY: help setup createdb migrate prepare run clean reset dev

# Default target
help:
 @echo "Available targets:"
 @echo "  setup     - Complete project setup (createdb + migrate + prepare)"
 @echo "  createdb  - Create the database"
 @echo "  migrate   - Run database migrations"
 @echo "  prepare   - Generate offline query data for SQLx"
 @echo "  run       - Run the application"
 @echo "  dev       - Setup and run the application"
 @echo "  reset     - Reset the database (drop and recreate)"
 @echo "  clean     - Clean build artifacts"

# Complete setup process
setup: createdb migrate prepare
 @echo "Setup complete! Ready to run the application."

# Create the database
createdb:
 @echo "Creating database..."
 sqlx database create

# Run database migrations
migrate:
 @echo "Running database migrations..."
 sqlx migrate run --source backend/migrations

# Generate offline query data for SQLx
prepare:
 @echo "Generating offline query data for SQLx..."
 cargo sqlx prepare --workspace

# Run the application
run:
 @echo "Starting NodeGaze..."
 cargo run

# Development workflow: setup then run
dev: setup run

# Reset database (drop and recreate)
reset:
 @echo "Resetting database..."
 sqlx database drop
 sqlx database create

# Clean build artifacts
clean:
 @echo "Cleaning build artifacts..."
 cargo clean
