package mock

import (
	"context"
	"dokusho/pkg/sources/source_types"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

type mockSource struct {
	source_types.Source
}

func NewMockSource() *mockSource {
	return &mockSource{
		Source: source_types.Source{
			SourceInformation: source_types.SourceInformation{
				ID:        source_types.SourceID("mock_source"),
				Name:      "Mock Source",
				URL:       "http://localhost:8080",
				Icon:      "http://localhost:8080/files/image.jpg",
				Languages: []source_types.SourceLanguage{source_types.EN, source_types.FR, source_types.JP, source_types.KO, source_types.ZH, source_types.ZH_HK},
				UpdatedAt: time.Now(),
				Version:   "1.0.0",
				NSFW:      true,
				SearchFilters: source_types.SupportedFilters{
					Query:   true,
					Orders:  []source_types.FetchSearchSerieFilterOrder{source_types.ASC, source_types.DESC},
					Sorts:   []source_types.FetchSearchSerieFilterSort{source_types.ALPHABETIC, source_types.LATEST, source_types.POPULARITY, source_types.RELEVANCE},
					Artists: true,
					Authors: true,
					Types:   []source_types.SourceSerieType{source_types.TYPE_COMIC, source_types.TYPE_MANGA, source_types.TYPE_MANHWA, source_types.TYPE_DOUJINSHI, source_types.TYPE_LIGHTNOVEL, source_types.TYPE_MANHUA, source_types.TYPE_NOVEL, source_types.TYPE_OEL, source_types.TYPE_WEBTOON},
					Status:  []source_types.SourceSerieStatus{source_types.STATUS_CANCELED, source_types.STATUS_COMPLETED, source_types.STATUS_HIATUS, source_types.STATUS_ONGOING, source_types.STATUS_PUBLISHED, source_types.STATUS_PUBLISHING, source_types.STATUS_SCANLATED, source_types.STATUS_SCANLATING},
					Genres: source_types.SupportedFiltersGenres{
						Included:       true,
						Excluded:       true,
						PossibleValues: source_types.ALL_GENRES,
					},
				},
			},
			SourceAPIInformation: source_types.SourceAPIInformation{
				APIURL:                &url.URL{Scheme: "https", Host: "localhost:8080"},
				MinimumUpdateInterval: 5 * time.Minute,
				Timeout:               time.Second * 1,
				Headers: http.Header{
					"User-Agent": []string{"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:71.0) Gecko/20100101 Firefox/77.0"},
				},
				CanBlockScraping: true,
			},
		},
	}
}

func (w *mockSource) GetInformation() source_types.SourceInformation {
	return w.Source.SourceInformation
}

func (w *mockSource) GetAPIInformation() source_types.SourceAPIInformation {
	return w.Source.SourceAPIInformation
}

func (w *mockSource) FetchPopularSerie(context context.Context, page int) (source_types.SourcePaginatedSmallSerie, error) {
	mockSerie := source_types.SourceSmallSerie{
		ID: source_types.SourceSerieID("ID"),
		Title: source_types.MultiLanguageString{
			EN:    "Mock",
			JP:    "Mock",
			JP_RO: "Mock",
			FR:    "Mock",
			KO:    "Mock",
			ZH:    "Mock",
			ZH_HK: "Mock",
		},
		Cover: "http://localhost:8080/files/image.jpg",
	}

	return source_types.SourcePaginatedSmallSerie{
		HasNextPage: true,
		Series: []source_types.SourceSmallSerie{
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
		},
	}, nil
}

func (w *mockSource) FetchLatestUpdates(context context.Context, page int) (source_types.SourcePaginatedSmallSerie, error) {
	mockSerie := source_types.SourceSmallSerie{
		ID: source_types.SourceSerieID("ID"),
		Title: source_types.MultiLanguageString{
			EN:    "Mock",
			JP:    "Mock",
			JP_RO: "Mock",
			FR:    "Mock",
			KO:    "Mock",
			ZH:    "Mock",
			ZH_HK: "Mock",
		},
		Cover: "http://localhost:8080/files/image.jpg",
	}

	return source_types.SourcePaginatedSmallSerie{
		HasNextPage: true,
		Series: []source_types.SourceSmallSerie{
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
		},
	}, nil
}

func (w *mockSource) FetchSearchSerie(context context.Context, page int, filter source_types.FetchSearchSerieFilter) (source_types.SourcePaginatedSmallSerie, error) {
	mockSerie := source_types.SourceSmallSerie{
		ID: source_types.SourceSerieID("ID"),
		Title: source_types.MultiLanguageString{
			EN:    "Mock",
			JP:    "Mock",
			JP_RO: "Mock",
			FR:    "Mock",
			KO:    "Mock",
			ZH:    "Mock",
			ZH_HK: "Mock",
		},
		Cover: "http://localhost:8080/files/image.jpg",
	}

	return source_types.SourcePaginatedSmallSerie{
		HasNextPage: true,
		Series: []source_types.SourceSmallSerie{
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
			mockSerie,
		},
	}, nil
}

func (w *mockSource) FetchSerieDetail(context context.Context, serieID source_types.SourceSerieID) (source_types.SourceSerie, error) {
	title := source_types.MultiLanguageString{
		EN:    "Mock",
		JP:    "Mock",
		JP_RO: "Mock",
		FR:    "Mock",
		KO:    "Mock",
		ZH:    "Mock",
		ZH_HK: "Mock",
	}

	random := "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed"

	mockSerie := source_types.SourceSerie{
		ID:                source_types.SourceSerieID("ID"),
		Title:             title,
		Cover:             "http://localhost:8080/files/image.jpg",
		AlternativeTitles: []source_types.MultiLanguageString{title, title, title},
		Synopsis: source_types.MultiLanguageString{
			EN:    random,
			JP:    random,
			JP_RO: random,
			FR:    random,
			KO:    random,
			ZH:    random,
			ZH_HK: random,
		},
		Type:    source_types.TYPE_MANGA,
		Authors: []string{"Mock 1", "Mock 2"},
		Artists: []string{"Mock 1", "Mock 2"},
		Status:  []source_types.SourceSerieStatus{source_types.STATUS_COMPLETED, source_types.STATUS_PUBLISHING},
		Genres:  []source_types.SourceSerieGenre{source_types.ACTION, source_types.ADAPTATION, source_types.ADULT},
		Volumes: []source_types.SourceSerieVolume{
			{
				ID:           "mock-volume-1",
				Name:         "Mock Volume",
				VolumeNumber: 1,
				Chapters: []source_types.SourceSerieVolumeChapter{
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-1-chapter-2-en"),
						Name:          "Chapter 2",
						ChapterNumber: 2,
						Language:      source_types.EN,
						DateUpload:    time.Now(),
					},
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-1-chapter-1-en"),
						Name:          "Chapter 1",
						ChapterNumber: 1,
						Language:      source_types.EN,
						DateUpload:    time.Now(),
					},
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-1-chapter-1-fr"),
						Name:          "Chapitre 1",
						ChapterNumber: 1,
						Language:      source_types.FR,
						DateUpload:    time.Now(),
					},
				},
			},
			{
				ID:           "mock-volume-2",
				Name:         "Mock Volume",
				VolumeNumber: 2,
				Chapters: []source_types.SourceSerieVolumeChapter{
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-2-chapter-2-fr"),
						Name:          "Chapitre 2",
						ChapterNumber: 2,
						Language:      source_types.FR,
						DateUpload:    time.Now(),
					},
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-2-chapter-1-en"),
						Name:          "Chapter 1",
						ChapterNumber: 1,
						Language:      source_types.EN,
						DateUpload:    time.Now(),
					},
					{
						ID:            source_types.SourceSerieVolumeChapterID("mock-volume-2-chapter-1-fr"),
						Name:          "Chapitre 1",
						ChapterNumber: 1,
						Language:      source_types.FR,
						DateUpload:    time.Now(),
					},
				},
			},
		},
	}

	return mockSerie, nil
}

func (w *mockSource) FetchChapterData(ctx context.Context, serieID source_types.SourceSerieID, volumeID source_types.SourceSerieVolumeID, chapterID source_types.SourceSerieVolumeChapterID) (source_types.SourceSerieVolumeChapterData, error) {
	data := source_types.SourceSerieVolumeChapterData{
		Type: source_types.IMAGE,
		Images: []source_types.SourceSerieVolumeChapterImage{
			{Index: 1, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 2, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 3, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 4, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 5, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 6, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 7, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 8, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 9, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 10, URL: "http://localhost:8080/files/image.jpg"},
			{Index: 11, URL: "http://localhost:8080/files/image.jpg"},
		},
	}

	return data, nil
}

func (w *mockSource) SerieUrl(serieID source_types.SourceSerieID) (*url.URL, error) {
	url, err := url.Parse(w.SourceInformation.URL)
	if err != nil {
		return nil, errors.Join(source_types.ErrBuildingURL, err, fmt.Errorf("failed to build URL: %s", w.SourceInformation.URL))
	}

	return url.JoinPath("series", string(serieID)), nil
}
