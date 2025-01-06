package router

import (
	"dokusho/pkg/http/utils"
	"dokusho/pkg/sources/source_types"
	"encoding/json"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
)

type SourceRouter struct {
	sources []source_types.SourceAPI
	logger  *slog.Logger
}

func NewSourceRouter(sources []source_types.SourceAPI) *SourceRouter {
	logger := slog.Default().WithGroup("sources_router")

	return &SourceRouter{
		sources: sources,
		logger:  logger,
	}
}

func (s *SourceRouter) Router() *http.ServeMux {
	router := http.NewServeMux()

	router.HandleFunc("GET /api/v1/sources", s.sourcesHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}", s.sourceHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/popular", s.popularSeriesHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/latest", s.latestSeriesHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/search", s.searchSeriesHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/series/{serieID}", s.serieHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/series/{serieID}/source_url", s.serieUrlHandler)
	router.HandleFunc("GET /api/v1/sources/{sourceID}/series/{serieID}/{volumeID}/{chapterID}", s.chapterHandler)

	return router
}

type SerieURL struct {
	URL string `json:"url"`
}

func (s *SourceRouter) serieUrlHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	serieID := utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.logger.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			url := source.SerieUrl(source_types.NewSourceSerieID(serieID))

			data := SerieURL{URL: url.String()}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			w.Write(json)
			return
		}
	}

}

func (s *SourceRouter) chapterHandler(w http.ResponseWriter, r *http.Request) {
	sourceID := utils.ExtractPathParam(r, "sourceID", "")
	if sourceID == "" {
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	serieID := utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.logger.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	volumeID := utils.ExtractPathParam(r, "volumeID", "")
	if volumeID == "" {
		s.logger.Error("No volume ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	chapterID := utils.ExtractPathParam(r, "chapterID", "")
	if chapterID == "" {
		s.logger.Error("No chapter ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchChapterData(r.Context(), source_types.SourceSerieID(serieID), source_types.SourceSerieVolumeID(volumeID), source_types.SourceSerieVolumeChapterID(chapterID))
			if err != nil {
				s.logger.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	serieID := utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		s.logger.Error("No serie ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchSerieDetail(r.Context(), source_types.SourceSerieID(serieID))
			if err != nil {
				s.logger.Error("Error fetching serie information", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.logger.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	query := utils.ExtractQueryValue(r, "query", "")

	srt := utils.ExtractQueryValue(r, "sort", "")
	sort := source_types.NewFetchSearchSerieFilterSort(srt)

	order := utils.ExtractQueryValue(r, "order", "")
	ord := source_types.NewFetchSearchSerieFilterOrder(order)

	a := strings.Trim(utils.ExtractQueryValue(r, "artists", ""), "")
	aa := strings.Split(a, ",")
	artists := make([]string, len(aa))
	for _, artist := range aa {
		artists = append(artists, artist)
	}

	a = strings.Trim(utils.ExtractQueryValue(r, "authors", ""), "")
	aa = strings.Split(a, ",")
	authors := make([]string, len(aa))
	for _, author := range aa {
		authors = append(authors, author)
	}

	t := strings.Trim(utils.ExtractQueryValue(r, "types", ""), "")
	tt := strings.Split(t, ",")
	types := make([]source_types.SourceSerieType, len(tt))
	for _, typ := range tt {
		types = append(types, source_types.NewSourceSerieType(typ))
	}

	sts := strings.Trim(utils.ExtractQueryValue(r, "status", ""), "")
	stsl := strings.Split(sts, ",")
	statuses := make([]source_types.SourceSerieStatus, len(stsl))
	for _, stat := range stsl {
		statuses = append(statuses, source_types.NewSourceSerieStatus(stat))
	}

	ig := strings.Trim(utils.ExtractQueryValue(r, "include_genres", ""), "")
	igl := strings.Split(ig, ",")
	includeGenres := make([]source_types.SourceSerieGenre, len(igl))
	for _, genre := range igl {
		includeGenres = append(includeGenres, source_types.NewSourceSerieGenre(genre))
	}

	eg := strings.Trim(utils.ExtractQueryValue(r, "exclude_genres", ""), "")
	egl := strings.Split(eg, ",")
	excludeGenres := make([]source_types.SourceSerieGenre, len(egl))
	for _, genre := range egl {
		excludeGenres = append(excludeGenres, source_types.NewSourceSerieGenre(genre))
	}

	s.logger.Info("filter", "query", query, "sort", sort, "order", ord, "artists", artists, "authors", authors, "types", types, "statuses", statuses, "include_genres", includeGenres, "exclude_genres", excludeGenres)

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
				s.logger.Error("Error fetching search series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.logger.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchPopularSerie(r.Context(), page)
			if err != nil {
				s.logger.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)
	}

	p := utils.ExtractQueryValue(r, "page", "1")
	page, err := strconv.Atoi(p)
	if err != nil {
		s.logger.Error("Error parsing page", "error", err)
		w.WriteHeader(http.StatusBadRequest)
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			data, err := source.FetchLatestUpdates(r.Context(), page)
			if err != nil {
				s.logger.Error("Error fetching popular series", "error", err)
				w.WriteHeader(http.StatusInternalServerError)
				return
			}

			json, err := json.Marshal(data)
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("No source ID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	for _, source := range s.sources {
		info := source.GetInformation()

		if string(info.ID) == sourceID {
			json, err := json.Marshal(source.GetInformation())
			if err != nil {
				s.logger.Error("Error marshalling sources", "error", err)
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
		s.logger.Error("Error marshalling sources", "error", err)
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	w.Write(json)
}
