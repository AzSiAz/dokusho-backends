package main

import (
	"dokusho/pkg/sources"
	"dokusho/pkg/sources/source_types"
	"encoding/json"
	"log/slog"
	"net/http"
)

func main() {
	slog.Info("Starting API Sources")

	srcs := sources.GetSources(nil)

	router := http.NewServeMux()
	router.HandleFunc("/sources", sourcesHandler(srcs))

	http.ListenAndServe(":8080", router)
}

func sourcesHandler(srcs []source_types.SourceAPI) http.HandlerFunc {

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var info []source_types.SourceInformation

		for _, source := range srcs {
			info = append(info, source.GetInformation())
		}

		json, err := json.Marshal(info)
		if err != nil {
			slog.Error("Error marshalling sources", "error", err)
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		w.Write(json)
	})
}
