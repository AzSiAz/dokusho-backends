package router

import (
	"dokusho/pkg/http/utils"
	"dokusho/pkg/sources/source_types"
	"encoding/json"
	"log/slog"
	"net/http"
	"strconv"
)

type SourceRouter struct {
	sources []source_types.SourceAPI
}

func NewSourceRouter(sources []source_types.SourceAPI) *SourceRouter {
	return &SourceRouter{
		sources: sources,
	}
}

func (s *SourceRouter) Router() *http.ServeMux {
	router := http.NewServeMux()

	router.HandleFunc("GET /sources", s.sourcesHandler)
	router.HandleFunc("GET /sources/{sourceID}", s.sourceHandler)
	router.HandleFunc("GET /sources/{sourceID}/popular", s.popularSeriesHandler)
	router.HandleFunc("GET /sources/{sourceID}/latest", s.latestSeriesHandler)
	router.HandleFunc("GET /sources/{sourceID}/search", s.searchSeriesHandler)
	router.HandleFunc("GET /sources/{sourceID}/series/{serieID}", s.serieHandler)
	router.HandleFunc("GET /sources/{sourceID}/series/{serieID}/{volumeID}/{chapterID}", s.chapterHandler)

	return router
}

func (s *SourceRouter) chapterHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	serieID := utils.ExtractPathParam(r, "serieID", "")
	if sourceID == "" {
		slog.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	volumeID := utils.ExtractPathParam(r, "volumeID", "")
	if volumeID == "" {
		slog.Error("No volume ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	chapterID := utils.ExtractPathParam(r, "chapterID", "")
	if chapterID == "" {
		slog.Error("No chapter ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchChapterData(r.Context(), source_types.SourceSerieID(serieID), source_types.SourceSerieVolumeID(volumeID), source_types.SourceSerieVolumeChapterID(chapterID))
			if err != nil {
				slog.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) serieHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	serieID := utils.ExtractPathParam(r, "serieID", "")
	if sourceID == "" {
		slog.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchSerieDetail(r.Context(), source_types.SourceSerieID(serieID))
			if err != nil {
				slog.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) searchSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		slog.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	query := utils.ExtractQueryValue(r, "query", "")

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchSearchSerie(r.Context(), page, source_types.FetchSearchSerieFilter{
				Query: query,
			})
			if err != nil {
				slog.Error("Error fetching search series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) popularSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		slog.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchPopularSerie(r.Context(), page)
			if err != nil {
				slog.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) latestSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		slog.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchLatestUpdates(r.Context(), page)
			if err != nil {
				slog.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) sourceHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		slog.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			json, err := json.Marshal(source.GetInformation())
			if err != nil {
				slog.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}
}

func (s *SourceRouter) sourcesHandler(w http.ResponseWriter, r *http.Request) {
	var info []source_types.SourceInformation

	for _, source := range s.sources {
		info = append(info, source.GetInformation())
	}

	json, err := json.Marshal(info)
	if err != nil {
		slog.Error("Error marshalling sources", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	w.Write(json)
}
