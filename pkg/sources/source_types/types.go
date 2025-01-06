package source_types

import (
	"context"
	"net/http"
	"net/url"
	"time"
)

type SourceSerieVolumeChapterImage struct {
	Index int    `json:"index"`
	URL   string `json:"url"`
}

type SourceSerieVolumeChapterText struct {
	Index int    `json:"index"`
	Text  string `json:"text"`
}

type SourceSerieVolumeChapterDataType string

const (
	TEXT  SourceSerieVolumeChapterDataType = "text"
	IMAGE SourceSerieVolumeChapterDataType = "image"
)

type SourceSerieVolumeChapterData struct {
	Type   SourceSerieVolumeChapterDataType `json:"type"`
	Images []SourceSerieVolumeChapterImage  `json:"images,omitempty"`
	Texts  []SourceSerieVolumeChapterText   `json:"texts,omitempty"`
}

type SourceSerieVolumeChapter struct {
	ID            SourceSerieVolumeChapterID `json:"id"`
	Name          string                     `json:"name"`
	ChapterNumber float64                    `json:"chapterNumber"`
	DateUpload    time.Time                  `json:"dateUpload"`
	ExternalURL   *url.URL                   `json:"externalURL"`
}

type SourceSerieVolume struct {
	ID           SourceSerieVolumeID        `json:"id"`
	Name         string                     `json:"name"`
	VolumeNumber float64                    `json:"volumeNumber"`
	Chapters     []SourceSerieVolumeChapter `json:"chapters"`
}

type SourceSerie struct {
	ID                SourceSerieID         `json:"id"`
	Title             MultiLanguageString   `json:"title"`
	AlternativeTitles []MultiLanguageString `json:"alternativeTitles"`
	Cover             string                `json:"cover"`
	Synopsis          MultiLanguageString   `json:"synopsis"`
	Type              SourceSerieType       `json:"type"`
	Genres            []SourceSerieGenre    `json:"genres"`
	Status            []SourceSerieStatus   `json:"status"`
	Authors           []string              `json:"authors"`
	Artists           []string              `json:"artists"`
	Volumes           []SourceSerieVolume   `json:"volumes"`
}

type SourceSmallSerie struct {
	ID    SourceSerieID       `json:"id"`
	Title MultiLanguageString `json:"title"`
	Cover string              `json:"cover"`
}

type SourcePaginatedSmallSerie struct {
	HasNextPage bool               `json:"hasNextPage"`
	Series      []SourceSmallSerie `json:"series"`
}

type FetchSearchSerieFilterGenres struct {
	Include []SourceSerieGenre `json:"include"`
	Exclude []SourceSerieGenre `json:"exclude"`
}

type FetchSearchSerieFilter struct {
	Query   string                       `json:"query"`
	Order   FetchSearchSerieFilterOrder  `json:"order"`
	Sort    FetchSearchSerieFilterSort   `json:"sort"`
	Artists []string                     `json:"artists"`
	Authors []string                     `json:"authors"`
	Genres  FetchSearchSerieFilterGenres `json:"genres"`
	Types   []SourceSerieType            `json:"types"`
	Status  []SourceSerieStatus          `json:"status"`
}

type SupportedFiltersGenres struct {
	Included       bool               `json:"included"`
	Excluded       bool               `json:"excluded"`
	PossibleValues []SourceSerieGenre `json:"values"`
}

type SupportedFilters struct {
	Query   bool                          `json:"query"`
	Orders  []FetchSearchSerieFilterOrder `json:"orders"`
	Sorts   []FetchSearchSerieFilterSort  `json:"sorts"`
	Artists bool                          `json:"artists"`
	Authors bool                          `json:"authors"`
	Types   []SourceSerieType             `json:"types"`
	Genres  SupportedFiltersGenres        `json:"genres"`
	Status  []SourceSerieStatus           `json:"status"`
}

type SourceInformation struct {
	ID            SourceID         `json:"id"`
	Name          string           `json:"name"`
	URL           string           `json:"url"`
	Icon          string           `json:"icon"`
	Languages     []SourceLanguage `json:"languages"`
	UpdatedAt     time.Time        `json:"updatedAt"`
	Version       string           `json:"version"`
	NSFW          bool             `json:"nsfw"`
	SearchFilters SupportedFilters `json:"supportedFilters"`
}

type SourceAPIInformation struct {
	APIURL                *url.URL      `json:"apiURL"`
	Headers               http.Header   `json:"headers"`
	MinimumUpdateInterval time.Duration `json:"minimumUpdateInterval"`
	Timeout               time.Duration `json:"timeout"`
	CanBlockScraping      bool          `json:"canBlockScraping"`
}

type Source struct {
	SourceInformation
	SourceAPIInformation
}

type SourceAPI interface {
	GetInformation() SourceInformation
	GetAPIInformation() SourceAPIInformation

	FetchPopularSerie(context context.Context, page int) (SourcePaginatedSmallSerie, error)
	FetchLatestUpdates(context context.Context, page int) (SourcePaginatedSmallSerie, error)
	FetchSearchSerie(context context.Context, page int, filter FetchSearchSerieFilter) (SourcePaginatedSmallSerie, error)
	FetchSerieDetail(context context.Context, serieID SourceSerieID) (SourceSerie, error)
	FetchChapterData(context context.Context, serieID SourceSerieID, volumeID SourceSerieVolumeID, chapterID SourceSerieVolumeChapterID) (SourceSerieVolumeChapterData, error)
	SerieUrl(serieID SourceSerieID) (*url.URL, error)
}
