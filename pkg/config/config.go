package config

import (
	"dokusho/pkg/utils"
	"errors"
	"fmt"
	"log/slog"
	"os"
)

var PORT = utils.Getenv("PORT", "8080")
var LISTEN_ADDR = utils.Getenv("LISTEN_ADDR", "localhost")
var LOG_LEVEL = utils.Getenv("LOG_LEVEL", "INFO")

var DATABASE_APP_URL = utils.Getenv("DATABASE_URL", "postgres://postgres@localhost:5433/dokusho")
var DATABASE_JOBS_URL = utils.Getenv("DATABASE_JOBS_URL", "postgres://postgres@localhost:5433/dokusho?search_path=jobs")
var DATABASE_APPLY_MIGRATIONS = utils.Getenv("DATABASE_APPLY_MIGRATIONS", "true") == "true"

var SOURCE_API_URL = utils.Getenv("SOURCE_API_URL", fmt.Sprintf("http://%s:%s", LISTEN_ADDR, PORT))
var SOURCE_USE_FLARESOLVER = utils.Getenv("SOURCE_USE_FLARESOLVER", "false") == "true"
var SOURCE_FLARESOLVER_URL = utils.Getenv("SOURCE_FLARESOLVER_URL", "")
var SOURCE_USE_MOCK = utils.Getenv("SOURCE_USE_MOCK", "true") == "true"

var FILE_SERVE_URL = utils.Getenv("FILE_SERVE_URL", fmt.Sprintf("http://%s:%s", LISTEN_ADDR, PORT))
var FILE_SERVE_MOCK = utils.Getenv("FILE_SERVE_MOCK", "false") == "true"
var FILE_ROOT_DIR = utils.Getenv("FILE_ROOT_DIR", "/Users/stef/Pictures/Dokusho")

func init() {
	slog.SetLogLoggerLevel(utils.NewLogLevel(LOG_LEVEL).SlogLevel())
}

type basicConfig struct {
	Port       string
	ListenAddr string
	LogLevel   string
}

type FileConfig struct {
	FileServeURL  string
	FileServeMock bool
	FileRootDir   string
}

type SourceConfig struct {
	SourceAPIURL         string
	SourceUseFlaresolver bool
	SourceFlaresolverURL string
	SourceUseMock        bool
}

type DatabaseConfig struct {
	DatabaseAppURL          string
	DatabaseJobsURL         string
	DatabaseApplyMigrations bool
}

type AIOConfig struct {
	basicConfig
	FileConfig
	SourceConfig
	DatabaseConfig
}

func NewAIOConfig() (AIOConfig, error) {
	bce := validateBasicConfig()
	fsce := validateFileServerConfig()
	ssce := validateSourceServerConfig()
	dbce := validateDatabaseConfig()

	err := errors.Join(bce, fsce, ssce, dbce)

	return AIOConfig{
		basicConfig: basicConfig{
			Port:       PORT,
			ListenAddr: LISTEN_ADDR,
			LogLevel:   LOG_LEVEL,
		},
		FileConfig: FileConfig{
			FileServeURL:  FILE_SERVE_URL,
			FileServeMock: FILE_SERVE_MOCK,
			FileRootDir:   FILE_ROOT_DIR,
		},
		DatabaseConfig: DatabaseConfig{
			DatabaseAppURL:          DATABASE_APP_URL,
			DatabaseJobsURL:         DATABASE_JOBS_URL,
			DatabaseApplyMigrations: DATABASE_APPLY_MIGRATIONS,
		},
		SourceConfig: SourceConfig{
			SourceAPIURL:         SOURCE_API_URL,
			SourceUseFlaresolver: SOURCE_USE_FLARESOLVER,
			SourceFlaresolverURL: SOURCE_FLARESOLVER_URL,
			SourceUseMock:        SOURCE_USE_MOCK,
		},
	}, err
}

func validateFileServerConfig() error {
	if FILE_ROOT_DIR == "" {
		return fmt.Errorf("FILE_ROOT_DIR is required")
	} else {
		err := os.MkdirAll(FILE_ROOT_DIR, os.ModePerm)
		if err != nil {
			return fmt.Errorf("Failed to create root dir: %w", err)
		}

		slog.Info("File root dir created, or already existing", "root_dir", FILE_ROOT_DIR)
	}

	return nil
}

func validateSourceServerConfig() error {
	if SOURCE_USE_FLARESOLVER && SOURCE_FLARESOLVER_URL == "" {
		return fmt.Errorf("SOURCE_FLARESOLVER_URL is required when SOURCE_USE_FLARESOLVER is true")
	}

	return nil
}

func validateDatabaseConfig() error {
	return nil
}

func validateBasicConfig() error {
	return nil
}
