package http_router

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"strconv"
	"strings"

	"dokusho/pkg/config"
	"dokusho/pkg/http_utils"
	"dokusho/pkg/sources/source_types"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

type SourceRouter struct {
	sources []source_types.SourceAPI
	l       *slog.Logger
	cfg     *config.SourceConfig
}

func NewSourceRouter(sources []source_types.SourceAPI, cfg *config.SourceConfig) *SourceRouter {
	logger := slog.Default().WithGroup("sources_router")

	return &SourceRouter{
		sources: sources,
		l:       logger,
		cfg:     cfg,
	}
}

func (s *SourceRouter) SetupMux() http.Handler {
	s.l.Info("Setting up source api router")

	mux := chi.NewMux()

	mux.Use(middleware.RequestID)
	mux.Use(middleware.RealIP)
	mux.Use(middleware.Logger)
	mux.Use(middleware.Recoverer)

	mux.Route("/api/v1/sources", func(r chi.Router) {
		r.Use(middleware.Heartbeat("/api/v1/sources/health"))
		r.Use(
			http_utils.WhitelistedReverseProxy(s.cfg.UseWhitelistedReverseProxy, s.cfg.WhitelistedReverseProxyAddr...),
			http_utils.APIKeyMiddleware(s.cfg.SourceUseAPIKey, s.cfg.SourceAPIKey),
		)

		r.Get("/", s.sourcesHandler)
		r.Get("/{sourceID}", s.sourceHandler)
		r.Get("/{sourceID}/popular", s.popularSeriesHandler)
		r.Get("/{sourceID}/latest", s.latestSeriesHandler)
		r.Get("/{sourceID}/search", s.searchSeriesHandler)
		r.Get("/{sourceID}/series/{serieID}", s.serieHandler)
		r.Get("/{sourceID}/series/{serieID}/source_url", s.serieUrlHandler)
		r.Get("/{sourceID}/series/{serieID}/{volumeID}/{chapterID}", s.chapterHandler)
	})

	return mux
}

type SerieURL struct {
	URL string `json:"url"`
}

func (s *SourceRouter) serieUrlHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	serieID := http_utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.l.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			url, err := source.SerieUrl(source_types.NewSourceSerieID(serieID))
			if err != nil {
				s.l.Error("Error generating serie url", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			data := SerieURL{URL: url.String()}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}

}

func (s *SourceRouter) chapterHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	serieID := http_utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.l.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	volumeID := http_utils.ExtractPathParam(r, "volumeID", "")
	if volumeID == "" {
		s.l.Error("No volume ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	chapterID := http_utils.ExtractPathParam(r, "chapterID", "")
	if chapterID == "" {
		s.l.Error("No chapter ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchChapterData(r.Context(), source_types.SourceSerieID(serieID), source_types.SourceSerieVolumeID(volumeID), source_types.SourceSerieVolumeChapterID(chapterID))
			if err != nil {
				s.l.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) serieHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	serieID := http_utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.l.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchSerieDetail(r.Context(), source_types.SourceSerieID(serieID))
			if err != nil {
				s.l.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) searchSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	p := http_utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.l.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	query := http_utils.ExtractQueryValue(r, "query", "")

	srt := http_utils.ExtractQueryValue(r, "sort", "")
	sort := source_types.NewFetchSearchSerieFilterSort(srt)

	order := http_utils.ExtractQueryValue(r, "order", "")
	ord := source_types.NewFetchSearchSerieFilterOrder(order)

	a := strings.Trim(http_utils.ExtractQueryValue(r, "artists", ""), "")
	aa := strings.Split(a, ",")
	artists := make([]string, len(aa))
	for _, artist := range aa {
		artists = append(artists, artist)
	}

	a = strings.Trim(http_utils.ExtractQueryValue(r, "authors", ""), "")
	aa = strings.Split(a, ",")
	authors := make([]string, len(aa))
	for _, author := range aa {
		authors = append(authors, author)
	}

	t := strings.Trim(http_utils.ExtractQueryValue(r, "types", ""), "")
	tt := strings.Split(t, ",")
	types := make([]source_types.SourceSerieType, len(tt))
	for _, typ := range tt {
		types = append(types, source_types.NewSourceSerieType(typ))
	}

	sts := strings.Trim(http_utils.ExtractQueryValue(r, "status", ""), "")
	stsl := strings.Split(sts, ",")
	statuses := make([]source_types.SourceSerieStatus, len(stsl))
	for _, stat := range stsl {
		statuses = append(statuses, source_types.NewSourceSerieStatus(stat))
	}

	ig := strings.Trim(http_utils.ExtractQueryValue(r, "include_genres", ""), "")
	igl := strings.Split(ig, ",")
	includeGenres := make([]source_types.SourceSerieGenre, len(igl))
	for _, genre := range igl {
		includeGenres = append(includeGenres, source_types.NewSourceSerieGenre(genre))
	}

	eg := strings.Trim(http_utils.ExtractQueryValue(r, "exclude_genres", ""), "")
	egl := strings.Split(eg, ",")
	excludeGenres := make([]source_types.SourceSerieGenre, len(egl))
	for _, genre := range egl {
		excludeGenres = append(excludeGenres, source_types.NewSourceSerieGenre(genre))
	}

	s.l.Info("filter", "query", query, "sort", sort, "order", ord, "artists", artists, "authors", authors, "types", types, "statuses", statuses, "include_genres", includeGenres, "exclude_genres", excludeGenres)

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			filter := source_types.FetchSearchSerieFilter{
				Query:   query,
				Sort:    sort,
				Order:   ord,
				Artists: artists,
				Authors: authors,
				Types:   types,
				Status:  statuses,
				Genres: source_types.FetchSearchSerieFilterGenres{
					Include: includeGenres,
					Exclude: excludeGenres,
				},
			}

			data, err := source.FetchSearchSerie(r.Context(), page, filter)
			if err != nil {
				s.l.Error("Error fetching search series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) popularSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	p := http_utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.l.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchPopularSerie(r.Context(), page)
			if err != nil {
				s.l.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) latestSeriesHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	p := http_utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.l.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchLatestUpdates(r.Context(), page)
			if err != nil {
				s.l.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err = encoder.Encode(data)
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) sourceHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := http_utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.l.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			w.Header().Set("Content-Type", "application/json; charset=utf-8")
			encoder := json.NewEncoder(w)
			err := encoder.Encode(source.GetInformation())
			if err != nil {
				s.l.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			return
		}
	}
}

func (s *SourceRouter) sourcesHandler(w http.ResponseWriter, r *http.Request) {
	var info []source_types.SourceInformation

	for _, source := range s.sources {
		info = append(info, source.GetInformation())
	}

	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	encoder := json.NewEncoder(w)
	err := encoder.Encode(info)
	if err != nil {
		s.l.Error("Error marshalling sources", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}
}
