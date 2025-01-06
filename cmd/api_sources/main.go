package main

import (
	"dokusho/pkg/http/router"
	"dokusho/pkg/sources"
	"fmt"
	"log/slog"
	"net/http"
	"os"
)

var PORT = os.Getenv("PORT")

func init() {
	if PORT == "" {
		PORT = "8080"
	}
}

func main() {
	slog.Info("Starting server", "port", PORT)

	r := http.NewServeMux()
	sourceRouter := router.NewSourceRouter(sources.BuildSources())

	r.Handle("/", sourceRouter.Router())

	http.ListenAndServe(fmt.Sprintf(":%s", PORT), r)
}
