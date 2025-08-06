#!/bin/bash
set -e

echo "🚀 Starting database preparation for SQLx compile-time checking..."

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Start PostgreSQL with Docker Compose
echo "📦 Starting PostgreSQL container..."
docker compose up -d postgres

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
    if docker compose exec -T postgres pg_isready -U dokusho > /dev/null 2>&1; then
        echo "✅ PostgreSQL is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ PostgreSQL failed to start in time"
        exit 1
    fi
    sleep 1
done

# Run migrations
echo "🔄 Running database migrations..."
cd crates/database
cargo sqlx migrate run

# Prepare offline mode data
echo "📝 Preparing SQLx offline mode data..."
cargo sqlx prepare

echo "✅ Database preparation complete!"
echo ""
echo "You can now build the project with:"
echo "  cargo build"
echo ""
echo "To stop the database:"
echo "  docker compose down"