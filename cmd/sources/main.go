package main

import (
	"fmt"
	"log/slog"
	"net/http"
	"os"

	"dokusho/pkg/client"
	"dokusho/pkg/config"
	"dokusho/pkg/http_router"
	"dokusho/pkg/sources"
)

func main() {
	cfg, err := config.NewSourceConfig()
	if err != nil {
		slog.Error("Failed to load config", "error", err)
		os.Exit(1)
	}

	err = isFlareSolverrHealthy(cfg.SourceFlaresolverURL)
	if err != nil {
		slog.Error("Failed to setup flaresolver", "error", err)
		os.Exit(1)
	}
	slog.Info("Flaresolver is healthy", "url", cfg.SourceFlaresolverURL)

	mux := http.NewServeMux()

	sourceRouter := http_router.NewSourceRouter(sources.BuildSources(cfg.SourceBaseConfig), cfg.SourceBaseConfig)
	mux = sourceRouter.SetupMux(mux)

	slog.Info("Starting server", "url", fmt.Sprintf("http://%s:%s", cfg.ListenAddr, cfg.Port))
	http.ListenAndServe(fmt.Sprintf(":%s", cfg.Port), mux)
}

func isFlareSolverrHealthy(url string) error {
	flaresolverClient := client.NewFlareSolverClient(url)
	err := flaresolverClient.Ping()
	if err != nil {
		return fmt.Errorf("Failed to connect to flaresolver, or flaresolver is not healthy: %w", err)
	}

	return nil
}
