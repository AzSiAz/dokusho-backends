package database

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/riverqueue/river"
	"github.com/riverqueue/river/riverdriver/riverpgxv5"
	"github.com/riverqueue/river/rivermigrate"
)

func Connect(databaseAppURL, databaseJobURL string, migrate bool) (*pgxpool.Pool, *river.Client[pgx.Tx], error) {
	apppool, err := pgxpool.New(context.Background(), databaseAppURL)
	if err != nil {
		return nil, nil, fmt.Errorf("Error opening database connection: %w", err)
	}

	err = apppool.Ping(context.Background())
	if err != nil {
		return nil, nil, fmt.Errorf("Error pinging database: %w", err)
	}

	if migrate {
		slog.Info("Running migrations")
		if strings.HasPrefix(databaseAppURL, "postgres") {
			databaseAppURL = strings.Replace(databaseAppURL, "postgres", "pgx5", 1)
		}

		err := Migrate(databaseAppURL)
		if err != nil {
			return nil, nil, err
		}

		slog.Info("Migrations ran successfully")
	}

	jobpool, err := pgxpool.New(context.Background(), databaseJobURL)
	if err != nil {
		return nil, nil, fmt.Errorf("Error opening database connection: %w", err)
	}

	err = jobpool.Ping(context.Background())
	if err != nil {
		return nil, nil, fmt.Errorf("Error pinging database: %w", err)
	}

	driver := riverpgxv5.New(jobpool)
	riverClient, err := river.NewClient(driver, &river.Config{})

	if migrate {
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

	return apppool, riverClient, nil
}
