package main

import (
	"dokusho/pkg/database"
	"dokusho/pkg/http/router"
	"dokusho/pkg/sources"
	"dokusho/pkg/utils"
	"fmt"
	"log/slog"
	"net/http"
	"os"
)

var PORT = utils.Getenv("PORT", "8080")
var LOG_LEVEL = utils.Getenv("LOG_LEVEL", "INFO")

var DATABASE_APP_URL = utils.Getenv("DATABASE_URL", "postgres://postgres@localhost:5433/dokusho")
var DATABASE_JOBS_URL = utils.Getenv("DATABASE_JOBS_URL", "postgres://postgres@localhost:5433/dokusho?search_path=jobs")
var DATABASE_APPLY_MIGRATIONS = utils.Getenv("DATABASE_APPLY_MIGRATIONS", "true") == "true"

var SOURCE_API_URL = utils.Getenv("SOURCE_API_URL", fmt.Sprintf("http://localhost:%s", PORT))
var SOURCE_USE_FLARESOLVER = utils.Getenv("SOURCE_USE_FLARESOLVER", "false") == "true"
var SOURCE_FLARESOLVER_URL = utils.Getenv("SOURCE_FLARESOLVER_URL", "")
var SOURCE_USE_MOCK = utils.Getenv("SOURCE_USE_MOCK", "false") == "true"

var FILE_SERVE_MOCK = utils.Getenv("FILE_SERVE_MOCK", "false") == "true"
var FILE_ROOT_DIR = utils.Getenv("FILE_ROOT_DIR", "/Users/stef/Pictures/Dokusho")

func init() {
	slog.SetLogLoggerLevel(utils.NewLogLevel(LOG_LEVEL).SlogLevel())

	if FILE_ROOT_DIR == "" {
		panic("FILE_ROOT_DIR is required")
	} else {
		err := os.MkdirAll(FILE_ROOT_DIR, os.ModePerm)
		if err != nil {
			panic(fmt.Errorf("Failed to create root dir: %w", err))
		}

		slog.Info("File root dir created, or already existing", "root_dir", FILE_ROOT_DIR)
	}

	if SOURCE_USE_FLARESOLVER && SOURCE_FLARESOLVER_URL == "" {
		panic("SOURCE_FLARESOLVER_URL is required when SOURCE_USE_FLARESOLVER is true")
	}
}

func main() {
	pgpool, _, err := database.Connect(DATABASE_APP_URL, DATABASE_JOBS_URL, DATABASE_APPLY_MIGRATIONS)
	if err != nil {
		slog.Error("Error getting a pg pool", "error", err)
		os.Exit(1)
	}
	defer pgpool.Close()

	mux := http.NewServeMux()

	sourceRouter := router.NewSourceRouter(sources.BuildSources(SOURCE_USE_MOCK))
	mux = sourceRouter.SetupMux(mux)

	backendRouter := router.NewBackendRouter(router.BackendRouterConfig{SourceAPIURL: SOURCE_API_URL}, pgpool)
	mux = backendRouter.SetupMux(mux)

	fileRouter := router.NewFileRouter(router.FileRouterConfig{RootDir: FILE_ROOT_DIR, ServeMock: FILE_SERVE_MOCK})
	mux = fileRouter.SetupMux(mux)

	slog.Info("Starting server", "url", fmt.Sprintf("http://localhost:%s", PORT))
	http.ListenAndServe(fmt.Sprintf(":%s", PORT), mux)
}
