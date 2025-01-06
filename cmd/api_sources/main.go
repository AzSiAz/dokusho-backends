package main

import (
	"dokusho/pkg/http/router"
	"dokusho/pkg/sources"
	"log/slog"
	"net/http"
)

func main() {
	slog.Info("Starting API Sources")

	srcs := sources.BuildSources()

	r := http.NewServeMux()
	sourceRouter := router.NewSourceRouter(srcs)

	r.Handle("/", sourceRouter.Router())

	http.ListenAndServe(":8080", r)
}
