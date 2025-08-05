.PHONY: setup dev test migrate check clean docker-up docker-down

# Setup development environment
setup:
	docker-compose -f docker-compose.dev.yml up -d
	cargo install sqlx-cli cargo-watch --locked
	sqlx database create
	sqlx migrate run

# Run API server with hot reload
dev:
	cargo watch -x "run --bin api"

# Run worker with hot reload
dev-worker:
	cargo watch -x "run --bin worker"

# Run all tests
test:
	cargo test --workspace

# Run tests for specific package
test-core:
	cargo test --package dokusho-core

# Run database migrations
migrate:
	sqlx migrate run

# Check code quality
check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Start Docker services
docker-up:
	docker-compose -f docker-compose.dev.yml up -d

# Stop Docker services
docker-down:
	docker-compose -f docker-compose.dev.yml down

# View logs
logs:
	docker-compose -f docker-compose.dev.yml logs -f

# Database shell
db-shell:
	docker exec -it dokusho-backends-postgres-1 psql -U dokusho