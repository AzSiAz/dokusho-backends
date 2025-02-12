package client

import (
	"context"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"

	"dokusho/pkg/http_router"
	"dokusho/pkg/sources/source_types"
)

type HTTPSourceAPIClient struct {
	BaseURL    *url.URL
	httpClient *http.Client
	logger     *slog.Logger
}

func NewHTTPSourceAPIClient(baseURL string, timeout time.Duration) (*HTTPSourceAPIClient, error) {
	if timeout == 0 {
		timeout = 10 * time.Second
	}

	httpClient := &http.Client{Timeout: timeout}
	logger := slog.Default().WithGroup("sources_router_client")

	url, err := url.Parse(baseURL)
	if err != nil {
		return nil, err
	}

	return &HTTPSourceAPIClient{
		BaseURL:    url,
		httpClient: httpClient,
		logger:     logger,
	}, nil
}

func (s *HTTPSourceAPIClient) GetSources(ctx context.Context) ([]source_types.SourceInformation, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources")

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return nil, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var data []source_types.SourceInformation
	err = json.Unmarshal(body, &data)
	if err != nil {
		return nil, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) GetSource(ctx context.Context, sourceID source_types.SourceID) (source_types.SourceInformation, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID))

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourceInformation{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourceInformation{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourceInformation{}, err
	}

	var data source_types.SourceInformation
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourceInformation{}, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) FetchPopularSeries(ctx context.Context, sourceID source_types.SourceID, page int) (source_types.SourcePaginatedSmallSerie, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "popular")

	q := url.Query()
	q.Set("page", strconv.Itoa(page))
	url.RawQuery = q.Encode()

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	var data source_types.SourcePaginatedSmallSerie
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) FetchLatestSeries(ctx context.Context, sourceID source_types.SourceID, page int) (source_types.SourcePaginatedSmallSerie, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "latest")

	q := url.Query()
	q.Set("page", strconv.Itoa(page))
	url.RawQuery = q.Encode()

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	var data source_types.SourcePaginatedSmallSerie
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) FetchSearchSeries(ctx context.Context, sourceID source_types.SourceID, page int, filter source_types.FetchSearchSerieFilter) (source_types.SourcePaginatedSmallSerie, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "search")

	q := url.Query()

	q.Set("page", strconv.Itoa(page))
	q.Set("query", filter.Query)
	q.Set("order", filter.Order.String())
	q.Set("sort", filter.Sort.String())
	q.Set("artists", strings.Join(filter.Artists, ","))
	q.Set("authors", strings.Join(filter.Authors, ","))

	var genres []string
	for _, genre := range filter.Genres.Exclude {
		genres = append(genres, genre.String())
	}
	q.Set("exclude_genres", strings.Join(genres, ","))

	genres = nil
	for _, genre := range filter.Genres.Include {
		genres = append(genres, genre.String())
	}
	q.Set("include_genres", strings.Join(genres, ","))

	var types []string
	for _, t := range filter.Types {
		types = append(types, t.String())
	}
	q.Set("types", strings.Join(types, ","))

	var status []string
	for _, s := range filter.Status {
		status = append(status, s.String())
	}
	q.Set("status", strings.Join(status, ","))

	url.RawQuery = q.Encode()

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	var data source_types.SourcePaginatedSmallSerie
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) FetchSerieInformation(ctx context.Context, sourceID source_types.SourceID, serieID source_types.SourceSerieID) (source_types.SourceSerie, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "series", string(serieID))

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourceSerie{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourceSerie{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourceSerie{}, err
	}

	var data source_types.SourceSerie
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourceSerie{}, err
	}

	return data, nil
}

func (s *HTTPSourceAPIClient) FetchSerieSourceUrl(ctx context.Context, sourceID source_types.SourceID, serieID source_types.SourceSerieID) (string, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "series", string(serieID), "source_url")

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return "", err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	var data http_router.SerieURL
	err = json.Unmarshal(body, &data)
	if err != nil {
		return "", err
	}

	return data.URL, nil
}

func (s *HTTPSourceAPIClient) FetchSerieChapters(ctx context.Context, sourceID source_types.SourceID, serieID source_types.SourceSerieID, volumeID source_types.SourceSerieVolumeID, chapterID source_types.SourceSerieVolumeChapterID) (source_types.SourceSerieVolumeChapterData, error) {
	url := s.BaseURL.JoinPath("/api/v1/sources", string(sourceID), "series", string(serieID), string(volumeID), string(chapterID))

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, err
	}

	req.WithContext(ctx)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, err
	}
	defer resp.Body.Close()

	// Read the body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, err
	}

	var data source_types.SourceSerieVolumeChapterData
	err = json.Unmarshal(body, &data)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, err
	}

	return data, nil
}
