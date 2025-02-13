package http_router

import (
	"log/slog"
	"net/http"

	"dokusho/pkg/config"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/jackc/pgx/v5/pgxpool"
)

type BackendRouter struct {
	config config.SourceConfig
	l      *slog.Logger
	pgpool *pgxpool.Pool
}

func NewBackendRouter(config config.SourceConfig, pgpool *pgxpool.Pool) *BackendRouter {
	logger := slog.Default().WithGroup("backend_router")

	return &BackendRouter{
		config: config,
		l:      logger,
	}
}

func (r *BackendRouter) SetupMux() *chi.Mux {
	r.l.Info("Setting up backend api router")

	mux := chi.NewMux()

	mux.Use(middleware.RequestID)
	mux.Use(middleware.RealIP)
	mux.Use(middleware.Logger)
	mux.Use(middleware.Recoverer)

	mux.Get("/api/v1/series", r.testHander)

	return mux
}

func (r *BackendRouter) testHander(w http.ResponseWriter, _ *http.Request) {
	r.l.Info("Test handler")

	w.Write([]byte("Test handler"))
}
