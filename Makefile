.PHONY: dev test migrate check clean docker-up docker-down

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
gen:
	sea-orm-cli generate entity -o crates/database/src/entities --with-serde both --with-copy-enums

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
	docker-compose -f compose.yml up -d

# Stop Docker services
docker-down:
	docker-compose -f compose.yml down

# View logs
logs:
	docker-compose -f compose.yml logs -f

# Database shell
db-shell:
	docker exec -it dokusho-backends-postgres-1 psql -U dokusho
