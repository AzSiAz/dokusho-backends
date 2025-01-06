package main

import (
	"dokusho/pkg/http/router"
	"dokusho/pkg/sources"
	"log/slog"
	"net/http"
)

func main() {
	slog.Info("Starting API Sources")

	r := http.NewServeMux()
	sourceRouter := router.NewSourceRouter(sources.BuildSources())

	r.Handle("/", sourceRouter.Router())

	http.ListenAndServe(":8080", r)
}
