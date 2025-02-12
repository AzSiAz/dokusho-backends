package mangadex

import (
	"context"
	"dokusho/pkg/sources/source_types"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"

	"dokusho/pkg/sources/chapterutils" // new import
)

const (
	noImageURL = "https://i.imgur.com/6TrIues.jpeg"
)

type mangadex struct {
	source_types.Source

	httpClient *http.Client
	logger     *slog.Logger
}

func NewMangadex() *mangadex {
	timeout := 5 * time.Second

	client := http.Client{Timeout: timeout}
	logger := slog.Default().WithGroup("mangadex")

	return &mangadex{
		httpClient: &client,
		logger:     logger,
		Source: source_types.Source{
			SourceInformation: source_types.SourceInformation{
				ID:        "mangadex",
				Name:      "MangaDex",
				URL:       "https://mangadex.org",
				Icon:      "https://mangadex.org/favicon.ico",
				Version:   "1.0.0",
				Languages: []source_types.SourceLanguage{source_types.EN, source_types.FR, source_types.JP, source_types.KO, source_types.ZH, source_types.ZH_HK},
				UpdatedAt: time.Date(2025, time.January, 07, 18, 0, 0, 0, time.UTC),
				NSFW:      false,
				SearchFilters: source_types.SupportedFilters{
					Query:   true,
					Artists: false,
					Authors: false,
					Orders:  GetSearchableOrders(),
					Sorts:   GetSearchableSorts(),
					Types:   []source_types.SourceSerieType{},
					Status:  GetSearchableStatus(),
					Genres: source_types.SupportedFiltersGenres{
						Included:       true,
						Excluded:       true,
						PossibleValues: GetSearchableGenres(),
					},
				},
			},
			SourceAPIInformation: source_types.SourceAPIInformation{
				APIURL:                &url.URL{Scheme: "https", Host: "api.mangadex.org"},
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

func (m *mangadex) GetInformation() source_types.SourceInformation {
	return m.Source.SourceInformation
}

func (m *mangadex) GetAPIInformation() source_types.SourceAPIInformation {
	return m.Source.SourceAPIInformation
}

func (m *mangadex) FetchPopularSerie(context context.Context, page int) (source_types.SourcePaginatedSmallSerie, error) {
	return m.FetchSearchSerie(context, page, source_types.FetchSearchSerieFilter{Sort: source_types.POPULARITY, Order: source_types.DESC})
}

func (m *mangadex) FetchLatestUpdates(context context.Context, page int) (source_types.SourcePaginatedSmallSerie, error) {
	return m.FetchSearchSerie(context, page, source_types.FetchSearchSerieFilter{Sort: source_types.LATEST, Order: source_types.DESC})
}

func (m *mangadex) FetchSearchSerie(context context.Context, page int, filter source_types.FetchSearchSerieFilter) (source_types.SourcePaginatedSmallSerie, error) {
	url := m.Source.SourceAPIInformation.APIURL.JoinPath("manga")

	limit := 20
	offset := (page - 1) * limit

	q := url.Query()
	q.Set("limit", strconv.Itoa(limit))
	q.Set("offset", strconv.Itoa(offset))
	q.Set("includedTagsMode", "AND")
	q.Set("excludedTagsMode", "OR")
	q.Add("contentRating[]", "safe")
	q.Add("contentRating[]", "suggestive")
	q.Add("contentRating[]", "erotica")

	for _, sourcelang := range m.Source.SourceInformation.Languages {
		lang, err := ConvertSourceSerieLanguage(sourcelang)
		if err != nil {
			continue
		}

		q.Add("availableTranslatedLanguage[]", string(lang))
	}

	q.Set("includes[]", "cover_art")

	if filter.Query != "" {
		q.Set("title", filter.Query)
	}

	if filter.Order != "" && filter.Sort != "" {
		mangadexsort, err := ConvertSourceSerieSort(filter.Sort)
		if err != nil {
			return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrInvalidSearchSort, err, fmt.Errorf("failed to convert sort: %s", filter.Sort))
		}

		mangadexorder, err := ConvertSourceSerieOrder(filter.Order)
		if err != nil {
			return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrInvalidSearchOrder, err, fmt.Errorf("failed to convert order: %s", filter.Order))
		}

		q.Set("order["+string(mangadexsort)+"]", string(mangadexorder))
	}

	if filter.Genres.Include != nil {
		for _, genre := range filter.Genres.Include {
			mangadexgenre, err := ConvertSourceSerieGenre(genre)
			if err != nil {
				continue
			}

			q.Add("includedTags[]", string(mangadexgenre))
		}
	}

	if filter.Genres.Exclude != nil {
		for _, genre := range filter.Genres.Exclude {
			mangadexgenre, err := ConvertSourceSerieGenre(genre)
			if err != nil {
				continue
			}

			q.Add("excludedTags[]", string(mangadexgenre))
		}
	}

	if filter.Status != nil {
		for _, status := range filter.Status {
			mangadexstatus, err := ConvertSourceSerieStatus(status)
			if err != nil {
				continue
			}

			q.Add("status[]", string(mangadexstatus))
		}
	}

	// Need to fetch authors and artists first

	// if filter.Artists != nil {
	// 	for _, artist := range filter.Artists {
	// 		q.Add("authorOrArtist", artist)
	// 	}
	// }

	// if filter.Authors != nil {
	// 	for _, author := range filter.Authors {
	// 		q.Add("authorOrArtist", author)
	// 	}
	// }

	url.RawQuery = q.Encode()

	req, err := http.NewRequestWithContext(context, http.MethodGet, url.String(), nil)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", url.String()))
	}

	m.logger.Info("Fetching search serie", "url", url.String())

	resp, err := m.httpClient.Do(req)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch data: %s", url.String()))
	}
	defer resp.Body.Close()

	m.logger.Debug("Fetched search serie", "status", resp.Status, "header", resp.Header)

	return m.ParseFetchSearchSerie(resp.Body)
}

func (m *mangadex) ParseFetchSearchSerie(html io.Reader) (source_types.SourcePaginatedSmallSerie, error) {
	data, err := io.ReadAll(html)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrParsingHTML, err, fmt.Errorf("failed to read response"))
	}

	var sr searchResponse
	err = json.Unmarshal(data, &sr)
	if err != nil {
		return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrParsingJSON, err, fmt.Errorf("failed to parse response"))
	}

	if sr.Result != "ok" {
		return source_types.SourcePaginatedSmallSerie{}, errors.Join(source_types.ErrHTTPRequestFailed, fmt.Errorf("response not ok"))
	}

	var series []source_types.SourceSmallSerie
	for _, s := range sr.Data {
		id := source_types.SourceSerieID(s.ID)
		title := m.getSerieTitle(s.Attributes.Title)
		cover := m.getSerieCover(id, s.Relationships)

		series = append(series, source_types.SourceSmallSerie{
			ID:    id,
			Title: title,
			Cover: cover,
		})
	}

	return source_types.SourcePaginatedSmallSerie{
		HasNextPage: sr.Offset < sr.Total,
		Series:      series,
	}, nil
}

func (m *mangadex) FetchSerieDetail(context context.Context, serieID source_types.SourceSerieID) (source_types.SourceSerie, error) {
	serieURL := m.Source.SourceAPIInformation.APIURL.JoinPath("manga", string(serieID))

	q := serieURL.Query()
	q.Add("includes[]", "author")
	q.Add("includes[]", "artist")
	q.Add("includes[]", "cover_art")
	serieURL.RawQuery = q.Encode()

	req, err := http.NewRequestWithContext(context, http.MethodGet, serieURL.String(), nil)
	if err != nil {
		return source_types.SourceSerie{}, errors.Join(source_types.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", serieURL.String()))
	}

	m.logger.Info("Fetching serie detail", "url", serieURL.String())

	resp, err := m.httpClient.Do(req)
	if err != nil {
		return source_types.SourceSerie{}, errors.Join(source_types.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch data: %s", serieURL.String()))
	}
	defer resp.Body.Close()

	m.logger.Debug("Fetched serie detail", "status", resp.Status, "header", resp.Header)

	serie, lang, err := m.ParseFetchSerieDetail(resp.Body)
	if err != nil {
		return source_types.SourceSerie{}, errors.Join(source_types.ErrExtractingData, err, fmt.Errorf("failed to extract data from response"))
	}

	var chapters []serieDetailChapterDetailResponse
	offset := 0
	limit := 500
	continueFetching := true

	volumesURL := m.Source.SourceAPIInformation.APIURL.JoinPath("manga", string(serieID), "feed")
	q = volumesURL.Query()
	q.Set("order[volume]", "desc")
	q.Set("order[chapter]", "desc")
	q.Set("limit", strconv.Itoa(limit))
	for _, lang := range lang {
		q.Add("translatedLanguage[]", string(lang))
	}
	volumesURL.RawQuery = q.Encode()

	// TODO(AzSiAz): Add some point there will be a need add a check to see if the chapter is already in the list, since the total could change if another chapter is added
	for {
		q := volumesURL.Query()
		q.Set("offset", strconv.Itoa(offset))
		volumesURL.RawQuery = q.Encode()

		req, err := http.NewRequestWithContext(context, http.MethodGet, volumesURL.String(), nil)
		if err != nil {
			return source_types.SourceSerie{}, errors.Join(source_types.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", volumesURL.String()))
		}

		m.logger.Info("Fetching serie volumes", "url", volumesURL.String())

		resp, err := m.httpClient.Do(req)
		if err != nil {
			return source_types.SourceSerie{}, errors.Join(source_types.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch data: %s", volumesURL.String()))
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			break
		}

		m.logger.Debug("Fetched serie volumes", "status", resp.Status, "header", resp.Header)

		parsedChapters, total, err := m.ParseFetchSerieDetailVolume(resp.Body)
		if err != nil {
			return source_types.SourceSerie{}, errors.Join(source_types.ErrExtractingData, err, fmt.Errorf("failed to extract data from response"))
		}

		chapters = append(chapters, parsedChapters...)

		continueFetching = limit+offset < total
		offset += limit

		if !continueFetching {
			break
		}

		time.Sleep(500 * time.Millisecond)
	}

	serie.Volumes = m.convertMangadexChapters(chapters)

	return serie, nil
}

func (m *mangadex) ParseFetchSerieDetail(html io.Reader) (source_types.SourceSerie, []MangadexLanguage, error) {
	data, err := io.ReadAll(html)
	if err != nil {
		return source_types.SourceSerie{}, nil, errors.Join(source_types.ErrParsingHTML, err, fmt.Errorf("failed to read response"))
	}

	var sdr serieDetailResponse
	err = json.Unmarshal(data, &sdr)
	if err != nil {
		return source_types.SourceSerie{}, nil, errors.Join(source_types.ErrParsingJSON, err, fmt.Errorf("failed to parse response"))
	}

	if sdr.Result != "ok" {
		return source_types.SourceSerie{}, nil, errors.Join(source_types.ErrHTTPRequestFailed, fmt.Errorf("response not ok"))
	}

	// Only use supported languages to fetch the chapters
	var langs []MangadexLanguage
	for _, lang := range sdr.Data.Attributes.AvailableTranslatedLanguages {
		pl, err := NewMangadexLanguage(lang)
		if err != nil {
			continue
		}

		langs = append(langs, pl)
	}

	id := source_types.SourceSerieID(sdr.Data.ID)
	status, err := ConvertMangadexStatus(MangadexStatus(sdr.Data.Attributes.Status))
	if err != nil {
		m.logger.Warn("Failed to convert status", "status", sdr.Data.Attributes.Status)
		status = source_types.STATUS_UNKNOWN
	}

	state, err := ConvertMangadexStatus(MangadexStatus(sdr.Data.Attributes.State))
	if err != nil {
		m.logger.Warn("Failed to convert state", "state", sdr.Data.Attributes.State)
		state = source_types.STATUS_UNKNOWN
	}

	genres := m.getGenres(sdr.Data.Attributes.Tags)

	return source_types.SourceSerie{
		ID:                id,
		Cover:             m.getSerieCover(id, sdr.Data.Relationships),
		Title:             m.getSerieTitle(sdr.Data.Attributes.Title),
		AlternativeTitles: m.getSerieAltTitle(sdr.Data.Attributes.AltTitles),
		Synopsis:          m.getSerieDescription(sdr.Data.Attributes.Description),
		Status:            []source_types.SourceSerieStatus{status, state},
		Type:              m.getType(sdr.Data.Attributes.OriginalLanguage, genres),
		Genres:            genres,
		Authors:           m.getAuthors(sdr.Data.Relationships),
		Artists:           m.getArtists(sdr.Data.Relationships),
	}, langs, nil
}

func (m *mangadex) ParseFetchSerieDetailVolume(html io.Reader) ([]serieDetailChapterDetailResponse, int, error) {
	rd, err := io.ReadAll(html)
	if err != nil {
		return nil, 0, errors.Join(source_types.ErrParsingHTML, err, fmt.Errorf("failed to read response"))
	}

	var data serieDetailChapterResponse
	err = json.Unmarshal(rd, &data)
	if err != nil {
		return nil, 0, errors.Join(source_types.ErrParsingJSON, err, fmt.Errorf("failed to parse response"))
	}

	if data.Result != "ok" {
		return nil, 0, errors.Join(source_types.ErrHTTPRequestFailed, fmt.Errorf("response not ok"))
	}

	total := data.Total

	return data.Data, total, nil
}

func (m *mangadex) FetchChapterData(context context.Context, serieID source_types.SourceSerieID, volumeID source_types.SourceSerieVolumeID, chapterID source_types.SourceSerieVolumeChapterID) (source_types.SourceSerieVolumeChapterData, error) {
	url := m.Source.SourceAPIInformation.APIURL.JoinPath("at-home/server", string(chapterID))

	q := url.Query()
	q.Set("forcePort443", "false")
	url.RawQuery = q.Encode()

	req, err := http.NewRequestWithContext(context, http.MethodGet, url.String(), nil)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, errors.Join(source_types.ErrBuildingRequest, err, fmt.Errorf("failed to build request: %s", url.String()))
	}

	m.logger.Info("Fetching chapter data", "url", url.String())

	resp, err := m.httpClient.Do(req)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, errors.Join(source_types.ErrHTTPRequestFailed, err, fmt.Errorf("failed to fetch data: %s", url.String()))
	}
	defer resp.Body.Close()

	m.logger.Debug("Fetched chapter data", "status", resp.Status, "header", resp.Header)

	return m.ParseFetchChapterData(resp.Body)
}

func (m *mangadex) ParseFetchChapterData(html io.Reader) (source_types.SourceSerieVolumeChapterData, error) {
	raw, err := io.ReadAll(html)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, errors.Join(source_types.ErrParsingHTML, err, fmt.Errorf("failed to read response"))
	}

	var data chapterImagesResponse
	err = json.Unmarshal(raw, &data)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, errors.Join(source_types.ErrParsingJSON, err, fmt.Errorf("failed to parse response"))
	}

	hash := data.Chapter.Hash
	baseUrl, err := url.Parse(data.BaseURL)
	if err != nil {
		return source_types.SourceSerieVolumeChapterData{}, errors.Join(source_types.ErrParsingURL, err, fmt.Errorf("failed to parse base url"))
	}

	images := make([]source_types.SourceSerieVolumeChapterImage, len(data.Chapter.Data))
	for i, img := range data.Chapter.Data {
		images[i] = source_types.SourceSerieVolumeChapterImage{
			Index: i + 1,
			URL:   baseUrl.JoinPath("data", hash, img).String(),
		}
	}

	return source_types.SourceSerieVolumeChapterData{
		Type:   source_types.IMAGE,
		Images: images,
	}, nil
}

func (m *mangadex) SerieUrl(serieID source_types.SourceSerieID) (*url.URL, error) {
	url, err := url.Parse("https://mangadex.org")
	if err != nil {
		return nil, errors.Join(source_types.ErrBuildingURL, err, fmt.Errorf("failed to build URL: %s", m.SourceInformation.URL))
	}

	return url.JoinPath("title", string(serieID)), nil
}

func (m *mangadex) getSerieTitle(title langField) source_types.MultiLanguageString {
	return source_types.MultiLanguageString{
		EN:    title.En,
		JP:    title.Ja,
		JP_RO: title.JaRo,
		KO:    title.Ko,
		ZH:    title.Zh,
		ZH_HK: title.ZhHk,
		FR:    title.Fr,
	}
}

func (m *mangadex) getSerieAltTitle(titles []langField) []source_types.MultiLanguageString {

	var altTitles []source_types.MultiLanguageString

	for _, title := range titles {
		altTitle := source_types.MultiLanguageString{
			EN:    title.En,
			JP:    title.Ja,
			JP_RO: title.JaRo,
			KO:    title.Ko,
			ZH:    title.Zh,
			ZH_HK: title.ZhHk,
			FR:    title.Fr,
		}

		if altTitle == (source_types.MultiLanguageString{}) {
			continue
		}

		altTitles = append(altTitles, altTitle)
	}

	return altTitles
}

func (m *mangadex) getSerieDescription(description langField) source_types.MultiLanguageString {
	return source_types.MultiLanguageString{
		EN:    description.En,
		JP:    description.Ja,
		JP_RO: description.JaRo,
		KO:    description.Ko,
		ZH:    description.Zh,
		ZH_HK: description.ZhHk,
		FR:    description.Fr,
	}
}

func (m *mangadex) getSerieCover(serieID source_types.SourceSerieID, relationship []serieDetailRelationship) string {
	if serieID == "" {
		return noImageURL
	}

	for _, r := range relationship {
		if r.Type == "cover_art" {
			m.logger.Debug("Cover", "name", r.Attributes.FileName, "r", r)

			filename := r.Attributes.FileName
			if filename == "" {
				continue
			}

			return fmt.Sprintf("https://uploads.mangadex.org/covers/%s/%s", serieID, filename)
		}
	}

	return noImageURL
}

func (m *mangadex) getGenres(tags []serieDetailTag) []source_types.SourceSerieGenre {
	var genres []source_types.SourceSerieGenre

	for _, tag := range tags {
		genre, err := ConvertMangadexGenre(MangadexGenre(tag.ID))
		if err != nil {
			continue
		}

		genres = append(genres, genre)
	}

	return genres
}

func (m *mangadex) getAuthors(relationship []serieDetailRelationship) []string {
	var authors []string

	for _, r := range relationship {
		if r.Type == "author" {
			m.logger.Debug("Author", "name", r.Attributes.Name, "r", r)

			authors = append(authors, r.Attributes.Name)
		}
	}

	return authors
}

func (m *mangadex) getArtists(relationship []serieDetailRelationship) []string {
	var artists []string

	for _, r := range relationship {
		if r.Type == "artist" {
			m.logger.Debug("Artist", "name", r.Attributes.Name, "r", r)

			artists = append(artists, r.Attributes.Name)
		}
	}

	return artists
}

func (m *mangadex) getType(originalLang string, genres []source_types.SourceSerieGenre) source_types.SourceSerieType {
	isLongStrip := false
	isDoujinshi := false
	isWebComic := false

	for _, genre := range genres {
		if genre == source_types.LONG_STRIP {
			isLongStrip = true
		}

		if genre == source_types.WEB_COMIC {
			isWebComic = true
		}

		if genre == source_types.DOUJINSHI {
			isDoujinshi = true
		}
	}

	m.logger.Debug("Type", "originalLang", originalLang, "isLongStrip", isLongStrip, "isDoujinshi", isDoujinshi, "isWebComic", isWebComic)

	if originalLang == JA.String() {
		return source_types.TYPE_MANGA
	}

	if originalLang == ZH.String() || originalLang == ZH_HK.String() {
		return source_types.TYPE_MANHUA
	}

	if (isWebComic) && originalLang == EN.String() {
		return source_types.TYPE_COMIC
	}

	if (isLongStrip || isWebComic) && originalLang == KO.String() {
		return source_types.TYPE_WEBTOON
	}

	if originalLang == KO.String() {
		return source_types.TYPE_MANHWA
	}

	if isDoujinshi {
		return source_types.TYPE_DOUJINSHI
	}

	return source_types.TYPE_UNKNOWN
}

// Add chapters to a map of volumes by volume id/number and in a second convert the map to a slice
func (m *mangadex) convertMangadexChapters(chapters []serieDetailChapterDetailResponse) []source_types.SourceSerieVolume {
	volumeMap := make(map[source_types.SourceSerieVolumeID][]source_types.SourceSerieVolumeChapter)

	for _, c := range chapters {
		chapterID := source_types.SourceSerieVolumeChapterID(c.ID)
		chapterTitle := c.Attributes.Title
		rawChapterNumber := c.Attributes.Chapter

		volumeID := source_types.SourceSerieVolumeID(c.Attributes.Volume)
		if volumeID == "" || strings.ToLower(string(volumeID)) == "unknown" {
			volumeID = "1"
		}

		chapterNumber, err := strconv.ParseFloat(rawChapterNumber, 64)
		if err != nil {
			m.logger.Warn("Failed to parse chapter number", "rawChapterNumber", rawChapterNumber, "error", err)
		}

		chapterLang, err := ConvertMangadexLanguage(MangadexLanguage(c.Attributes.TranslatedLanguage))
		if err != nil {
			m.logger.Warn("Failed to convert language", "lang", c.Attributes.TranslatedLanguage)
			continue
		}

		if volumeMap[volumeID] == nil {
			volumeMap[volumeID] = []source_types.SourceSerieVolumeChapter{}
		}

		volumeMap[volumeID] = append(volumeMap[volumeID], source_types.SourceSerieVolumeChapter{
			ID:            chapterID,
			Name:          chapterTitle,
			ChapterNumber: float64(int(chapterNumber*1000)) / 1000,
			Language:      chapterLang,
			DateUpload:    c.Attributes.CreatedAt,
			ExternalURL:   c.Attributes.ExternalURL,
		})
	}

	var volumes []source_types.SourceSerieVolume

	for key, v := range volumeMap {
		volumeNumber, err := strconv.ParseFloat(string(key), 64)
		if err != nil {
			m.logger.Warn("Failed to parse chapter number", "rawChapterNumber", key, "error", err)
		}

		// compute missing chapters for this volume
		var chapterNumbers []float64
		for _, ch := range v {
			chapterNumbers = append(chapterNumbers, ch.ChapterNumber)
		}
		missing := chapterutils.CalculateMissingChapters(chapterNumbers)

		volumes = append(volumes, source_types.SourceSerieVolume{
			ID:              source_types.SourceSerieVolumeID(fmt.Sprintf("volume-%s", key)),
			Name:            fmt.Sprintf("Volume %s", key),
			VolumeNumber:    float64(int(volumeNumber*1000)) / 1000,
			Chapters:        v,
			MissingChapters: missing,
		})
	}

	return volumes
}
