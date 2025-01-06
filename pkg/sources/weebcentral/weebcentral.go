package weebcentral

import (
	"context"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"
	"time"

	"dokusho/pkg/sources/source_types"
	sources "dokusho/pkg/sources/source_types"

	"github.com/PuerkitoBio/goquery"
)

var (
	chapterNumberRegex = regexp.MustCompile(`\d+(\.\d+)?`)
)

type weebCentral struct {
	sources.Source

	httpClient *http.Client
	logger     *slog.Logger
}

func NewWeebCentral() *weebCentral {
	timeout := 5 * time.Second

	client := http.Client{Timeout: timeout}
	logger := slog.Default().WithGroup("weebcentral")

	return &weebCentral{
		httpClient: &client,
		logger:     logger,
		Source: sources.Source{
			SourceInformation: sources.SourceInformation{
				ID:        "weebcentral",
				Name:      "WeebCentral",
				URL:       "https://weebcentral.com",
				Icon:      "https://weebcentral.com/favicon.ico",
				Version:   "1.0.0",
				Languages: []sources.SourceLanguage{sources.EN},
				UpdatedAt: time.Date(2024, time.December, 26, 14, 22, 0, 0, time.UTC),
				NSFW:      false,
				SearchFilters: sources.SupportedFilters{
					Query:   true,
					Artists: true,
					Authors: true,
					Orders:  GetSearchableOrders(),
					Sorts:   GetSearchableSorts(),
					Types:   GetSearchableTypes(),
					Status:  GetSearchableStatus(),
					Genres: sources.SupportedFiltersGenres{
						Included:       true,
						Excluded:       true,
						PossibleValues: GetSearchableGenres(),
					},
				},
			},
			SourceAPIInformation: sources.SourceAPIInformation{
				APIURL:                &url.URL{Scheme: "https", Host: "weebcentral.com"},
				MinimumUpdateInterval: 5 * time.Minute,
				Timeout:               timeout,
				Headers: http.Header{
					"User-Agent": []string{"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:71.0) Gecko/20100101 Firefox/77.0"},
				},
				CanBlockScraping: true,
			},
		},
	}
}

func (w *weebCentral) GetInformation() sources.SourceInformation {
	return w.Source.SourceInformation
}

func (w *weebCentral) GetAPIInformation() sources.SourceAPIInformation {
	return w.Source.SourceAPIInformation
}

func (w *weebCentral) FetchPopularSerie(context context.Context, page int) (sources.SourcePaginatedSmallSerie, error) {
	filter := sources.FetchSearchSerieFilter{
		Order: sources.DESC,
		Sort:  sources.POPULARITY,
	}

	return w.FetchSearchSerie(context, page, filter)
}

func (w *weebCentral) FetchLatestUpdates(context context.Context, page int) (sources.SourcePaginatedSmallSerie, error) {
	filter := sources.FetchSearchSerieFilter{
		Order: sources.DESC,
		Sort:  sources.LATEST,
	}

	return w.FetchSearchSerie(context, page, filter)
}

func (w *weebCentral) FetchSearchSerie(context context.Context, page int, filter sources.FetchSearchSerieFilter) (sources.SourcePaginatedSmallSerie, error) {
	url := w.SourceAPIInformation.APIURL.JoinPath(fmt.Sprintf("/search/data"))

	limit := 24
	offset := (page - 1) * limit

	q := url.Query()
	q.Set("limit", strconv.Itoa(limit))
	if offset > 0 {
		q.Set("offset", strconv.Itoa(offset))
	}

	q.Set("official", "Any")
	q.Set("display_mode", "Full Display")

	if filter.Query != "" {
		q.Set("text", filter.Query)
	}

	if filter.Sort != "" {
		sort, err := ConvertSourceSerieSort(filter.Sort)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchSort, err, fmt.Errorf("invalid sort: %s", filter.Sort))
		}

		q.Set("sort", string(sort))
	}

	if filter.Order != "" {
		order, err := ConvertSourceSerieOrder(filter.Order)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchOrder, err, fmt.Errorf("invalid order: %s", filter.Order))
		}

		q.Set("order", string(order))
	}

	if filter.Types != nil {
		types, err := ConvertSourceSerieTypes(filter.Types)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchTypes, err, fmt.Errorf("invalid types: %s", filter.Types))
		}

		for _, t := range types {
			q.Add("included_type", string(t))
		}
	}

	if filter.Status != nil {
		status, err := ConvertSourceSerieStatuses(filter.Status)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchStatus, err, fmt.Errorf("invalid status: %s", filter.Status))
		}

		for _, s := range status {
			q.Add("included_status", string(s))
		}
	}

	if filter.Genres.Include != nil {
		ig, err := ConvertSourceSerieGenres(filter.Genres.Include)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchGenres, err, fmt.Errorf("invalid included genres: %s", filter.Genres.Include))
		}

		for _, g := range ig {
			q.Add("included_tag", string(g))
		}
	}

	if filter.Genres.Exclude != nil {
		eg, err := ConvertSourceSerieGenres(filter.Genres.Exclude)
		if err != nil {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSearchGenres, err, fmt.Errorf("invalid excluded genres: %s", filter.Genres.Exclude))
		}

		for _, g := range eg {
			q.Add("excluded_tag", string(g))
		}
	}

	if filter.Artists != nil || len(filter.Artists) > 0 {
		for _, a := range filter.Artists {
			q.Add("author", a)
		}
	}

	if filter.Authors != nil || len(filter.Authors) > 0 {
		for _, a := range filter.Authors {
			q.Add("author", a)
		}
	}

	url.RawQuery = q.Encode()

	w.logger.Info("Fetching search url", "url", url.String())

	req, err := http.NewRequest("GET", url.String(), nil)
	if err != nil {
		return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", url))
	}

	req.WithContext(context)
	req.Header = w.SourceAPIInformation.Headers

	resp, err := w.httpClient.Do(req)
	if err != nil || resp.StatusCode != http.StatusOK {
		return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch search data: %+v", resp))
	}
	defer resp.Body.Close()

	w.logger.Debug("Fetched chapter data", "status", resp.Status, "header", resp.Header)

	return w.ParseFetchSearchSerie(resp.Body)
}

func (w *weebCentral) ParseFetchSearchSerie(html io.Reader) (sources.SourcePaginatedSmallSerie, error) {
	doc, err := goquery.NewDocumentFromReader(html)
	if err != nil {
		html, errRead := io.ReadAll(html)
		w.logger.Debug("Weird search HTML received", "html", html, "error_read", errRead)

		return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrParsingHTML, err, fmt.Errorf("failed to parse search html: %s", html))
	}

	articles := doc.Find("body > article")
	w.logger.Debug("Found series in search html", "count", articles.Length())

	if articles.Length() == 0 {
		return sources.SourcePaginatedSmallSerie{HasNextPage: false}, nil
	}

	var series []sources.SourceSmallSerie

	articles.Each(func(i int, s *goquery.Selection) {
		coverRaw := s.Find("section").First().Find("a > article > picture > source").First().AttrOr("srcset", "")
		cover, err := url.Parse(coverRaw)
		if err != nil {
			w.logger.Warn("Failed to parse cover URL", "raw_url", coverRaw, "error", err)
		}

		title := s.Find("section").Last().Find("div").First().Find("a").Text()
		if title == "" {
			w.logger.Warn("Empty title")
			title = "Unknown Title"
		}

		firstLinkRaw := s.Find("section").First().Find("a").AttrOr("href", "")
		firstLink, err := url.Parse(firstLinkRaw)
		if err != nil {
			w.logger.Warn("Failed to parse first link URL", "raw_url", firstLinkRaw, "error", err)
		}

		idParts := strings.Split(firstLink.Path, "/")
		id := sources.SourceSerieID(idParts[len(idParts)-2])

		series = append(series, sources.SourceSmallSerie{
			ID:    id,
			Title: sources.MultiLanguageString{EN: title},
			Cover: cover.String(),
		})
	})

	for _, serie := range series {
		if serie.ID == "" {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidSerieID, fmt.Errorf("Serie id can't be empty: %s", serie))
		}

		if serie.Cover == "" {
			return sources.SourcePaginatedSmallSerie{}, errors.Join(sources.ErrInvalidCover, fmt.Errorf("Serie cover can't be empty: %s", serie))
		}
	}

	hasNextPage := doc.Find("button > span").First().Text() == "View More Results..."

	return sources.SourcePaginatedSmallSerie{
		HasNextPage: hasNextPage,
		Series:      series,
	}, nil
}

func (w *weebCentral) FetchSerieDetail(context context.Context, serieID sources.SourceSerieID) (sources.SourceSerie, error) {
	serieURL, err := w.SerieUrl(serieID)
	if err != nil {
		return sources.SourceSerie{}, errors.Join(sources.ErrBuildingURL, err, fmt.Errorf("failed to build serie URL: %s", serieID))
	}

	chaptersURL := serieURL.JoinPath("full-chapter-list")

	w.logger.Info("Fetching serie detail", "serie_url", serieURL.String(), "chapters_url", chaptersURL.String())

	serieReq, err := http.NewRequest("GET", serieURL.String(), nil)
	if err != nil {
		return sources.SourceSerie{}, errors.Join(sources.ErrBuildingRequest, err, fmt.Errorf("failed to build serie request: %s", serieURL))
	}

	chaptersReq, err := http.NewRequest("GET", chaptersURL.String(), nil)
	if err != nil {
		return sources.SourceSerie{}, errors.Join(sources.ErrBuildingRequest, err, fmt.Errorf("failed to build chapters request: %s", chaptersURL))
	}

	serieReq.WithContext(context)
	chaptersReq.WithContext(context)
	serieReq.Header = w.SourceAPIInformation.Headers
	chaptersReq.Header = w.SourceAPIInformation.Headers

	serieResp, err := w.httpClient.Do(serieReq)
	if err != nil || serieResp.StatusCode != http.StatusOK {
		return sources.SourceSerie{}, errors.Join(sources.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch serie detail data: %+v", serieResp))
	}
	defer serieResp.Body.Close()

	w.logger.Debug("Fetched serie detail", "status", serieResp.Status, "header", serieResp.Header)

	chaptersResp, err := w.httpClient.Do(chaptersReq)
	if err != nil || chaptersResp.StatusCode != http.StatusOK {
		return sources.SourceSerie{}, errors.Join(sources.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch chapters list data: %+v", chaptersResp))
	}
	defer chaptersResp.Body.Close()

	w.logger.Debug("Fetched chapters list", "status", chaptersResp.Status, "header", chaptersResp.Header)

	return w.ParseFetchSerieDetail(serieID, chaptersResp.Body, serieResp.Body)
}

func (w *weebCentral) ParseFetchSerieDetail(serieID sources.SourceSerieID, chaptersHTML io.Reader, serieHTML io.Reader) (sources.SourceSerie, error) {
	chaptersDoc, err := goquery.NewDocumentFromReader(chaptersHTML)
	if err != nil {
		html, errRead := io.ReadAll(chaptersHTML)
		w.logger.Debug("Weird chapters HTML received", "html", html, "error_read", errRead)

		return sources.SourceSerie{}, errors.Join(sources.ErrParsingHTML, err, fmt.Errorf("failed to parse chapters list html: %s", html))
	}

	cm := chaptersDoc.Find("body").ChildrenFiltered("a.flex")
	w.logger.Debug("Found chapters", "count", cm.Length())

	var chapters []sources.SourceSerieVolumeChapter
	for i := range cm.Nodes {
		elem := cm.Eq(i)

		name := elem.Find("span.flex > span").First().Text()

		chapterIDParts := strings.Split(elem.AttrOr("href", ""), "/")
		chapterID := chapterIDParts[len(chapterIDParts)-1]

		rawDate := elem.Find("time").AttrOr("datetime", "")
		dateUpload, err := time.Parse(time.RFC3339, rawDate)
		if err != nil {
			w.logger.Warn("Failed to parse date", "raw_date", rawDate, "error", err)
		}

		rawNumber := chapterNumberRegex.FindString(name)
		if rawNumber == "" {
			w.logger.Warn("Failed to find chapter number", "name", name)
		}
		number, err := strconv.ParseFloat(rawNumber, 64)
		if err != nil {
			w.logger.Warn("Failed to parse chapter number", "raw_number", rawNumber, "error", err)
		}

		chapters = append(chapters, sources.SourceSerieVolumeChapter{
			ID:            sources.SourceSerieVolumeChapterID(chapterID),
			Name:          name,
			DateUpload:    dateUpload,
			ChapterNumber: float64(int(number*1000)) / 1000,
			ExternalURL:   nil,
		})
	}

	serieDoc, err := goquery.NewDocumentFromReader(serieHTML)
	if err != nil {
		html, errRead := io.ReadAll(serieHTML)
		w.logger.Debug("Weird serie detail HTML received", "html", html, "error_read", errRead)

		return sources.SourceSerie{}, errors.Join(sources.ErrParsingHTML, err, fmt.Errorf("failed to parse serie detail html: %s", html))
	}

	sm := serieDoc.Find("body > main")

	rawType := sm.Find("li:has(strong:contains(Type)) > a").First().Text()
	parseType, err := ConvertWeebCentralType(WeebCentralType(rawType))
	if err != nil {
		w.logger.Warn("Failed to parse type", "raw_type", rawType, "error", err)
	}

	rawStatus := sm.Find("li:has(strong:contains(Status)) > a").First().Text()
	status, err := ConvertWeebCentralStatus(WeebCentralStatus(rawStatus))
	if err != nil {
		w.logger.Warn("Failed to parse status", "raw_status", rawStatus, "error", err)
	}

	rawGenres := sm.Find("li:has(strong:contains(Tags)) > span > a")
	genres := make([]sources.SourceSerieGenre, rawGenres.Length())
	for i := range genres {
		rawGenre := rawGenres.Eq(i).Text()
		genre, err := ConvertWeebCentralGenre(WeebCentralGenre(rawGenre))
		if err != nil {
			w.logger.Warn("Failed to parse genre", "raw_genre", rawGenre, "error", err)
		} else {
			genres[i] = genre
		}
	}

	rawCoverURL := sm.Find("img").First().AttrOr("src", "")
	if rawCoverURL == "" {
		w.logger.Warn("Empty cover URL")
	}
	coverURL, err := url.Parse(rawCoverURL)
	if err != nil {
		w.logger.Warn("Failed to parse cover URL", "raw_url", rawCoverURL, "error", err)
	}

	title := sm.Find("h1").First().Text()
	synopsis := sm.Find("li:has(strong:contains(Description)) > p").First().Text()
	authors := sm.Find("li:has(strong:contains(Author)) > span > a").Map(func(i int, s *goquery.Selection) string { return s.Text() })

	return sources.SourceSerie{
		ID:                serieID,
		Title:             sources.MultiLanguageString{EN: title},
		Cover:             coverURL.String(),
		Synopsis:          sources.MultiLanguageString{EN: synopsis},
		Type:              parseType,
		Status:            []sources.SourceSerieStatus{status},
		Authors:           authors,
		Artists:           []string{},
		AlternativeTitles: []sources.MultiLanguageString{},
		Genres:            genres,
		Volumes: []sources.SourceSerieVolume{
			{
				ID:           "volume-1",
				Name:         "Volume Unknown",
				VolumeNumber: 1,
				Chapters:     chapters,
			},
		},
	}, nil
}

func (w *weebCentral) FetchChapterData(ctx context.Context, serieID sources.SourceSerieID, volumeID sources.SourceSerieVolumeID, chapterID sources.SourceSerieVolumeChapterID) (sources.SourceSerieVolumeChapterData, error) {
	chapterDataURL := w.SourceAPIInformation.APIURL.JoinPath(fmt.Sprintf("/chapters/%s/images", chapterID))

	q := chapterDataURL.Query()
	q.Add("reading_style", "long_strip")

	chapterDataURL.RawQuery = q.Encode()

	w.logger.Info("Fetching chapter data", "url", chapterDataURL.String())

	req, err := http.NewRequest("GET", chapterDataURL.String(), nil)
	if err != nil {
		return sources.SourceSerieVolumeChapterData{}, errors.Join(sources.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", chapterDataURL))
	}

	req.WithContext(ctx)
	req.Header = w.SourceAPIInformation.Headers

	resp, err := w.httpClient.Do(req)
	if err != nil || resp.StatusCode != http.StatusOK {
		return sources.SourceSerieVolumeChapterData{}, errors.Join(sources.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch chapter images data: %+v", resp))
	}
	defer resp.Body.Close()

	w.logger.Debug("Fetched chapter data", "status", resp.Status, "header", resp.Header)

	return w.ParseFetchChapterData(resp.Body)
}

func (w *weebCentral) ParseFetchChapterData(html io.Reader) (sources.SourceSerieVolumeChapterData, error) {
	doc, err := goquery.NewDocumentFromReader(html)
	if err != nil {
		html, err := io.ReadAll(html)
		w.logger.Debug("Weird HTML received", "html", html, "error_read", err)

		return sources.SourceSerieVolumeChapterData{}, errors.Join(sources.ErrParsingHTML, err, fmt.Errorf("failed to parse chapters data html: %s", html))
	}

	var images []sources.SourceSerieVolumeChapterImage
	doc.Find("img").Each(func(i int, s *goquery.Selection) {
		rawURL := s.AttrOr("src", "")
		if rawURL == "" {
			w.logger.Warn("Empty image URL", "index", i)
			return
		}

		u, error := url.Parse(rawURL)
		if error != nil {
			w.logger.Warn("Failed to parse image URL", "index", i, "src", rawURL, "error", error)
		}

		images = append(images, sources.SourceSerieVolumeChapterImage{
			Index: i + 1,
			URL:   u.String(),
		})
	})

	return sources.SourceSerieVolumeChapterData{Images: images, Type: sources.IMAGE}, nil
}

func (w *weebCentral) SerieUrl(serieID sources.SourceSerieID) (*url.URL, error) {
	url, err := url.Parse(w.SourceInformation.URL)
	if err != nil {
		return nil, errors.Join(source_types.ErrBuildingURL, err, fmt.Errorf("failed to build URL: %s", w.SourceInformation.URL))
	}

	return url.JoinPath("series", string(serieID)), nil
}
