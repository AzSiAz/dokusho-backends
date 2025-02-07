package main

import (
	"dokusho/pkg/config"
	"dokusho/pkg/database"
	"dokusho/pkg/http/router"
	"dokusho/pkg/sources"
	"fmt"
	"log/slog"
	"net/http"
	"os"
)

func main() {
	cfg, err := config.NewAIOConfig()

	pgpool, _, err := database.Connect(cfg.DatabaseConfig)
	if err != nil {
		slog.Error("Error getting a postgres pool for application or jobs", "error", err)
		os.Exit(1)
	}
	defer pgpool.Close()

	mux := http.NewServeMux()

	sourceRouter := router.NewSourceRouter(sources.BuildSources(cfg.SourceConfig))
	mux = sourceRouter.SetupMux(mux)

	backendRouter := router.NewBackendRouter(cfg.SourceConfig, pgpool)
	mux = backendRouter.SetupMux(mux)

	fileRouter := router.NewFileRouter(cfg.FileConfig)
	mux = fileRouter.SetupMux(mux)

	slog.Info("Starting server", "url", fmt.Sprintf("http://%s:%s", cfg.ListenAddr, cfg.Port))
	http.ListenAndServe(fmt.Sprintf(":%s", cfg.Port), mux)
}
