package router

import (
	"dokusho/pkg/config"
	"log/slog"
	"net/http"

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

func (r *BackendRouter) SetupMux(mux *http.ServeMux) *http.ServeMux {
	r.l.Info("Setting up backend api router")

	mux.HandleFunc("GET /api/v1/series", r.testHander)

	return mux
}

func (r *BackendRouter) testHander(w http.ResponseWriter, _ *http.Request) {
	r.l.Info("Test handler")

	w.Write([]byte("Test handler"))
}
