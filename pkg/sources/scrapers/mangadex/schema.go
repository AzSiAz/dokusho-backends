package mangadex

import "time"

type langField struct {
	De   string `json:"de,omitempty"`
	Fr   string `json:"fr,omitempty"`
	It   string `json:"it,omitempty"`
	Pl   string `json:"pl,omitempty"`
	Th   string `json:"th,omitempty"`
	EsLa string `json:"es-la,omitempty"`
	Ko   string `json:"ko,omitempty"`
	En   string `json:"en,omitempty"`
	KoRo string `json:"ko-ro,omitempty"`
	JaRo string `json:"ja-ro,omitempty"`
	Ru   string `json:"ru,omitempty"`
	Ja   string `json:"ja,omitempty"`
	Zh   string `json:"zh,omitempty"`
	ZhHk string `json:"zh-hk,omitempty"`
	La   string `json:"la,omitempty"`
	Tr   string `json:"tr,omitempty"`
	PtBr string `json:"pt-br,omitempty"`
	Ro   string `json:"ro,omitempty"`
	Uk   string `json:"uk,omitempty"`
	Fa   string `json:"fa,omitempty"`
	Hi   string `json:"hi,omitempty"`
	Ne   string `json:"ne,omitempty"`
	ID   string `json:"id,omitempty"`
	Ka   string `json:"ka,omitempty"`
	Ar   string `json:"ar,omitempty"`
	Bn   string `json:"bn,omitempty"`
	He   string `json:"he,omitempty"`
	Hu   string `json:"hu,omitempty"`
	Ta   string `json:"ta,omitempty"`
}

type serieDetailRelationship struct {
	ID         string `json:"id"`
	Type       string `json:"type"`
	Attributes struct {
		Name      string    `json:"name,omitempty"`
		FileName  string    `json:"fileName,omitempty"`
		CreatedAt time.Time `json:"createdAt,omitempty"`
		UpdatedAt time.Time `json:"updatedAt,omitempty"`
		Version   int       `json:"version,omitempty"`
	} `json:"attributes,omitempty"`
	Related string `json:"related,omitempty"`
}

type serieDetailTag struct {
	ID         string `json:"id"`
	Type       string `json:"type"`
	Attributes struct {
		Name        langField `json:"name"`
		Description langField `json:"description"`
		Group       string    `json:"group"`
		Version     int       `json:"version"`
	} `json:"attributes"`
	Relationships []any `json:"relationships"`
}

type serieDetailResponse struct {
	Result   string `json:"result"`
	Response string `json:"response"`
	Data     struct {
		ID         string `json:"id"`
		Type       string `json:"type"`
		Attributes struct {
			Title       langField   `json:"title"`
			AltTitles   []langField `json:"altTitles"`
			Description langField   `json:"description"`
			IsLocked    bool        `json:"isLocked"`
			Links       struct {
				Al    string `json:"al"`
				Ap    string `json:"ap"`
				Bw    string `json:"bw"`
				Kt    string `json:"kt"`
				Mu    string `json:"mu"`
				Nu    string `json:"nu"`
				Amz   string `json:"amz"`
				Ebj   string `json:"ebj"`
				Mal   string `json:"mal"`
				Raw   string `json:"raw"`
				Engtl string `json:"engtl"`
			} `json:"links"`
			OriginalLanguage               string           `json:"originalLanguage"`
			LastVolume                     string           `json:"lastVolume"`
			LastChapter                    string           `json:"lastChapter"`
			PublicationDemographic         any              `json:"publicationDemographic"`
			Status                         string           `json:"status"`
			Year                           int              `json:"year"`
			ContentRating                  string           `json:"contentRating"`
			Tags                           []serieDetailTag `json:"tags"`
			State                          string           `json:"state"`
			ChapterNumbersResetOnNewVolume bool             `json:"chapterNumbersResetOnNewVolume"`
			CreatedAt                      time.Time        `json:"createdAt"`
			UpdatedAt                      time.Time        `json:"updatedAt"`
			Version                        int              `json:"version"`
			AvailableTranslatedLanguages   []string         `json:"availableTranslatedLanguages"`
			LatestUploadedChapter          string           `json:"latestUploadedChapter"`
		} `json:"attributes"`
		Relationships []serieDetailRelationship `json:"relationships"`
	} `json:"data"`
}

type serieDetailChapterResponse struct {
	Result   string                             `json:"result"`
	Response string                             `json:"response"`
	Data     []serieDetailChapterDetailResponse `json:"data"`
	Limit    int                                `json:"limit"`
	Offset   int                                `json:"offset"`
	Total    int                                `json:"total"`
}

type serieDetailChapterDetailResponse struct {
	ID         string `json:"id"`
	Type       string `json:"type"`
	Attributes struct {
		Volume             string    `json:"volume"`
		Chapter            string    `json:"chapter"`
		Title              string    `json:"title"`
		TranslatedLanguage string    `json:"translatedLanguage"`
		ExternalURL        string    `json:"externalUrl"`
		PublishAt          time.Time `json:"publishAt"`
		ReadableAt         time.Time `json:"readableAt"`
		CreatedAt          time.Time `json:"createdAt"`
		UpdatedAt          time.Time `json:"updatedAt"`
		Pages              int       `json:"pages"`
		Version            int       `json:"version"`
	} `json:"attributes"`
}

type chapterImagesResponse struct {
	Result  string `json:"result"`
	BaseURL string `json:"baseUrl"`
	Chapter struct {
		Hash      string   `json:"hash"`
		Data      []string `json:"data"`
		DataSaver []string `json:"dataSaver"`
	} `json:"chapter"`
}

type searchResponse struct {
	Result   string `json:"result,omitempty"`
	Response string `json:"response,omitempty"`
	Data     []struct {
		ID         string `json:"id,omitempty"`
		Type       string `json:"type,omitempty"`
		Attributes struct {
			Title                          langField        `json:"title,omitempty"`
			AltTitles                      []langField      `json:"altTitles,omitempty"`
			Description                    langField        `json:"description,omitempty"`
			IsLocked                       bool             `json:"isLocked,omitempty"`
			OriginalLanguage               string           `json:"originalLanguage,omitempty"`
			LastVolume                     string           `json:"lastVolume,omitempty"`
			LastChapter                    string           `json:"lastChapter,omitempty"`
			PublicationDemographic         string           `json:"publicationDemographic,omitempty"`
			Status                         string           `json:"status,omitempty"`
			Year                           int              `json:"year,omitempty"`
			ContentRating                  string           `json:"contentRating,omitempty"`
			Tags                           []serieDetailTag `json:"tags,omitempty"`
			State                          string           `json:"state,omitempty"`
			ChapterNumbersResetOnNewVolume bool             `json:"chapterNumbersResetOnNewVolume,omitempty"`
			CreatedAt                      time.Time        `json:"createdAt,omitempty"`
			UpdatedAt                      time.Time        `json:"updatedAt,omitempty"`
			Version                        int              `json:"version,omitempty"`
			AvailableTranslatedLanguages   []string         `json:"availableTranslatedLanguages,omitempty"`
			LatestUploadedChapter          string           `json:"latestUploadedChapter,omitempty"`
		} `json:"attributes,omitempty"`
		Relationships []serieDetailRelationship `json:"relationships,omitempty"`
	} `json:"data,omitempty"`
	Limit  int `json:"limit,omitempty"`
	Offset int `json:"offset,omitempty"`
	Total  int `json:"total,omitempty"`
}
