package weebcentral_test

import (
	"dokusho/pkg/sources/scrapers/weebcentral"
	"dokusho/pkg/sources/source_types"
	"log/slog"
	"strings"
	"testing"
)

var source = weebcentral.NewWeebCentral()
var disableLog = false

func init() {
	// Disable the logger
	if disableLog {
		slog.SetLogLoggerLevel(slog.LevelError)
	} else {
		slog.SetLogLoggerLevel(slog.LevelDebug)
	}
}

func TestWeebCentralParseFetchSearchSerie(t *testing.T) {
	t.Parallel()

	series, err := source.ParseFetchSearchSerie(strings.NewReader(searchHTML))
	if err != nil {
		t.Error(err)
	}

	if len(series.Series) == 0 {
		t.Error("Series should not be empty")
	}

	if series.HasNextPage == false {
		t.Error("HasNextPage should not be false")
	}

	for _, serie := range series.Series {
		t.Log(serie)
	}
}

func TestWeebCentralParseFetchSerieDetail(t *testing.T) {
	t.Parallel()

	serie, err := source.ParseFetchSerieDetail("01J76XYGC5B3EH5D5XDR7M490Q", strings.NewReader(chaptersListHTML), strings.NewReader(serieHTML))
	if err != nil {
		t.Error(err)
	}

	if serie.Title.EN == "" {
		t.Error("Serie title should not be empty")
	}

	if serie.Synopsis.EN == "" {
		t.Error("Serie description should not be empty")
	}

	t.Log(serie.Title)
	t.Log(serie.Synopsis)
	t.Log(serie.Cover)
	t.Log(serie.Status)
	t.Log(serie.Type)
}

func TestWeebCentralParseFetchChapterData(t *testing.T) {
	t.Parallel()

	reader := strings.NewReader(chapterImagesHTML)
	chapter, err := source.ParseFetchChapterData(reader)
	if err != nil {
		t.Error(err)
	}

	if len(chapter.Texts) != 0 {
		t.Error("Chapter for this source should not have texts")
	}

	if len(chapter.Images) == 0 {
		t.Error("Chapter for this source should have images")
	}

	if chapter.Type != source_types.IMAGE {
		t.Error("Chapter for this source should only have type Image")
	}

	for _, image := range chapter.Images {
		t.Log(image)
	}
}

func TestWeebCentralSerieUrl(t *testing.T) {
	var serieIDs = []source_types.SourceSerieID{
		"01J76XYGC5B3EH5D5XDR7M490Q/Sono-Munou-Jitsuha-Sekai-Saikyou-No-Mahoutsukai",
		"01J76XYGC5B3EH5D5XDR7M490Q/",
		"01J76XYGC5B3EH5D5XDR7M490Q",
	}

	for _, serieID := range serieIDs {
		t.Run(string(serieID), func(t *testing.T) {
			t.Parallel()

			// TODO: Fix this test by checking the URL
			source.SerieUrl(serieID)
		})
	}
}
