package router

import (
	"dokusho/pkg/config"
	"dokusho/pkg/http/utils"
	_ "embed"
	"log/slog"
	"net/http"
)

type FileRouter struct {
	config config.FileConfig
	l      *slog.Logger
}

//go:embed image.jpg
var mockImage []byte

func NewFileRouter(config config.FileConfig) *FileRouter {
	logger := slog.Default().WithGroup("backend_router")

	return &FileRouter{
		config: config,
		l:      logger,
	}
}

func (fr *FileRouter) SetupMux(mux *http.ServeMux) *http.ServeMux {
	fr.l.Info("Setting up file api router")

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

	useMockImage := utils.ExtractQueryValue(r, "mock", "true")
	if useMockImage == "true" {
		fr.serveMockImage(w)
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

	useMockImage := utils.ExtractQueryValue(r, "mock", "true")

	if hash == "image.jpg" || useMockImage == "true" {
		fr.serveMockImage(w)
		return
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

	useMockImage := utils.ExtractQueryValue(r, "mock", "true")
	if useMockImage == "true" {
		fr.serveMockImage(w)
		return
	}

	w.Write([]byte("Serie cover handler"))
}

func (fr *FileRouter) serveMockImage(w http.ResponseWriter) {
	fr.l.Info("Serving mock image from embeded file")

	w.Header().Set("Content-Type", "image/jpeg")
	w.WriteHeader(http.StatusOK)
	w.Write(mockImage)
}
