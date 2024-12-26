package database

import (
	"embed"
	"errors"
	"fmt"

	"github.com/golang-migrate/migrate/v4"
	_ "github.com/golang-migrate/migrate/v4/database/pgx/v5"
	"github.com/golang-migrate/migrate/v4/source/iofs"
)

//go:embed migrations/*.sql
var migrations embed.FS

func Migrate(databaseURL string) error {
	d, err := iofs.New(migrations, "migrations")
	if err != nil {
		return fmt.Errorf("Failed to build FS: %w", err)
	}

	m, err := migrate.NewWithSourceInstance("iofs", d, databaseURL)
	if err != nil {
		return fmt.Errorf("Failed to create migration instance: %w", err)
	}

	err = m.Up()
	if err != nil {
		if errors.Is(migrate.ErrNoChange, err) || errors.Is(migrate.ErrNilVersion, err) {
			return nil
		}

		return fmt.Errorf("Failed to run migrations: %w", err)
	}

	return nil
}
