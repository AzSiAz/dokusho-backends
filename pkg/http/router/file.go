package router

import (
	"dokusho/pkg/http/utils"
	_ "embed"
	"log/slog"
	"net/http"
)

type FileRouterConfig struct {
	RootDir   string
	ServeMock bool
}

type FileRouter struct {
	config FileRouterConfig
	l      *slog.Logger
}

//go:embed image.jpg
var mockImage []byte

func NewFileRouter(config FileRouterConfig) *FileRouter {
	logger := slog.Default().WithGroup("backend_router")

	return &FileRouter{
		config: config,
		l:      logger,
	}
}

func (fr *FileRouter) SetupMux(mux *http.ServeMux) *http.ServeMux {
	fr.l.Info("Setting up backend api router")

	mux.HandleFunc("GET /files/{serieID}/{volumeID}/{chapterID}/{page}", fr.fileSerieHandler)
	mux.HandleFunc("GET /files/{serieID}/cover", fr.fileSerieCoverHandler)
	mux.HandleFunc("GET /files/{hash}", fr.hashFileHandler)

	return mux
}

func (fr *FileRouter) fileSerieHandler(w http.ResponseWriter, r *http.Request) {
	serieID := utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		fr.l.Error("No serieID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	volumeID := utils.ExtractPathParam(r, "volumeID", "")
	if volumeID == "" {
		fr.l.Error("No volumeID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	chapterID := utils.ExtractPathParam(r, "chapterID", "")
	if chapterID == "" {
		fr.l.Error("No chapterID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	page := utils.ExtractPathParam(r, "page", "")
	if page == "" {
		fr.l.Error("No page provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	w.Write([]byte("Serie page handler"))
}

func (fr *FileRouter) hashFileHandler(w http.ResponseWriter, r *http.Request) {
	hash := utils.ExtractPathParam(r, "hash", "")
	if hash == "" {
		fr.l.Error("No hash provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	if hash == "image.jpg" {
		fr.l.Info("Serving mock image from embeded file")

		w.Header().Set("Content-Type", "image/jpeg")
		w.WriteHeader(http.StatusOK)
		w.Write(mockImage)
	}

	http.NotFound(w, r)
}

func (fr *FileRouter) fileSerieCoverHandler(w http.ResponseWriter, r *http.Request) {
	serieID := utils.ExtractPathParam(r, "serieID", "")
	if serieID == "" {
		fr.l.Error("No serieID provided")
		w.WriteHeader(http.StatusBadRequest)

		return
	}

	w.Write([]byte("Serie cover handler"))
}
