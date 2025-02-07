package database

import (
	"context"
	"dokusho/pkg/config"
	"fmt"
	"log/slog"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/riverqueue/river"
	"github.com/riverqueue/river/riverdriver/riverpgxv5"
	"github.com/riverqueue/river/rivermigrate"
)

func Connect(cfg config.DatabaseConfig) (*pgxpool.Pool, *river.Client[pgx.Tx], error) {
	DBPool, err := pgxpool.New(context.Background(), cfg.DatabaseAppURL)
	if err != nil {
		return nil, nil, fmt.Errorf("Error opening database connection: %w", err)
	}

	err = DBPool.Ping(context.Background())
	if err != nil {
		return nil, nil, fmt.Errorf("Error pinging database: %w", err)
	}

	if cfg.DatabaseApplyMigrations {
		slog.Info("Running migrations")

		migrationURL := cfg.DatabaseAppURL

		if strings.HasPrefix(cfg.DatabaseAppURL, "postgres") {
			migrationURL = strings.Replace(cfg.DatabaseAppURL, "postgres", "pgx5", 1)
		}

		err := Migrate(migrationURL)
		if err != nil {
			return nil, nil, err
		}

		slog.Info("Migrations ran successfully")
	}

	jobpool, err := pgxpool.New(context.Background(), cfg.DatabaseJobsURL)
	if err != nil {
		return nil, nil, fmt.Errorf("Error opening database connection: %w", err)
	}

	err = jobpool.Ping(context.Background())
	if err != nil {
		return nil, nil, fmt.Errorf("Error pinging database: %w", err)
	}

	driver := riverpgxv5.New(jobpool)
	riverDBClient, err := river.NewClient(driver, &river.Config{})

	if cfg.DatabaseApplyMigrations {
		migrator, err := rivermigrate.New(driver, nil)
		if err != nil {
			return nil, nil, fmt.Errorf("Error creating river migrator: %w", err)
		}

		r, err := migrator.Migrate(context.Background(), rivermigrate.DirectionUp, nil)
		if err != nil {
			return nil, nil, fmt.Errorf("Error running river migrations: %w", err)
		}

		for _, m := range r.Versions {
			slog.Info("Migrated River", "version", m.Version, "name", m.Name, "in", m.Duration)
		}
	}

	return DBPool, riverDBClient, nil
}
