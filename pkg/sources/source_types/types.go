package source_types

import (
	"context"
	"net/http"
	"net/url"
	"time"
)

type SourceID string
type SourceSerieID string
type SourceSerieVolumeID string
type SourceSerieVolumeChapterID string
type SourceSerieVolumeChapterImageID string

type SourceLanguage string

const (
	EN  SourceLanguage = "en"
	JP  SourceLanguage = "jp"
	FR  SourceLanguage = "fr"
	KR  SourceLanguage = "kr"
	CH  SourceLanguage = "ch"
	ALL SourceLanguage = "all"
)

type SourceSerieStatus string

const (
	STATUS_ONGOING         SourceSerieStatus = "ongoing"
	STATUS_COMPLETED       SourceSerieStatus = "completed"
	STATUS_HIATUS          SourceSerieStatus = "hiatus"
	STATUS_CANCELED        SourceSerieStatus = "canceled"
	STATUS_UNKNOWN         SourceSerieStatus = "unknown"
	STATUS_PUBLISHING      SourceSerieStatus = "publishing"
	STATUS_PUBLISHING_DONE SourceSerieStatus = "publishing_done"
	STATUS_SCANLATING      SourceSerieStatus = "scanlating"
	STATUS_SCANLATING_DONE SourceSerieStatus = "scanlating_done"
)

type SourceSerieType string

const (
	TYPE_MANGA      SourceSerieType = "manga"
	TYPE_MANHWA     SourceSerieType = "manhwa"
	TYPE_MANHUA     SourceSerieType = "manhua"
	TYPE_WEBTOON    SourceSerieType = "webtoon"
	TYPE_LIGHTNOVEL SourceSerieType = "lightnovel"
	TYPE_NOVEL      SourceSerieType = "novel"
	TYPE_DOUJINSHI  SourceSerieType = "doujinshi"
	TYPE_COMIC      SourceSerieType = "comic"
	TYPE_OEL        SourceSerieType = "oel"
	TYPE_UNKNOWN    SourceSerieType = "unknown"
)

type SourceSerieGenre string

const (
	UNKNOWN               SourceSerieGenre = "Unknown"
	OTHER                 SourceSerieGenre = "Other"
	FOUR_KOMA             SourceSerieGenre = "4-Koma"
	ACTION                SourceSerieGenre = "Action"
	ADAPTATION            SourceSerieGenre = "Adaptation"
	ADULT                 SourceSerieGenre = "Adult"
	ADVENTURE             SourceSerieGenre = "Adventure"
	ALIENS                SourceSerieGenre = "Aliens"
	ANIMALS               SourceSerieGenre = "Animals"
	ANTHOLOGY             SourceSerieGenre = "Anthology"
	AWARD_WINNING         SourceSerieGenre = "Award Winning"
	BOYS_LOVE             SourceSerieGenre = "Boy's Love"
	COMEDY                SourceSerieGenre = "Comedy"
	COOKING               SourceSerieGenre = "Cooking"
	CRIME                 SourceSerieGenre = "Crime"
	CROSSDRESSING         SourceSerieGenre = "Crossdressing"
	DELINQUENTS           SourceSerieGenre = "Delinquents"
	DEMONS                SourceSerieGenre = "Demons"
	DOUJINSHI             SourceSerieGenre = "Doujinshi"
	DRAMA                 SourceSerieGenre = "Drama"
	ECCHI                 SourceSerieGenre = "Ecchi"
	FAN_COLORED           SourceSerieGenre = "Fan Colored"
	FANTASY               SourceSerieGenre = "Fantasy"
	FULL_COLOR            SourceSerieGenre = "Full Color"
	GENDER_BENDER         SourceSerieGenre = "Gender Bender"
	GENDERSWAP            SourceSerieGenre = "Genderswap"
	GHOST                 SourceSerieGenre = "Ghost"
	GIRLS_LOVE            SourceSerieGenre = "Girl's Love"
	GORE                  SourceSerieGenre = "Gore"
	GYARU                 SourceSerieGenre = "Gyaru"
	HAREM                 SourceSerieGenre = "Harem"
	HENTAI                SourceSerieGenre = "Hentai"
	HISTORICAL            SourceSerieGenre = "Historical"
	HORROR                SourceSerieGenre = "Horror"
	INCEST                SourceSerieGenre = "Incest"
	ISEKAI                SourceSerieGenre = "Isekai"
	JOSEI                 SourceSerieGenre = "Josei"
	KIDS                  SourceSerieGenre = "Kids"
	LOLICON               SourceSerieGenre = "Lolicon"
	LONG_STRIP            SourceSerieGenre = "Long Strip"
	MAFIA                 SourceSerieGenre = "Mafia"
	MAGIC                 SourceSerieGenre = "Magic"
	MAGICAL_GIRLS         SourceSerieGenre = "Magical Girls"
	MARTIAL_ARTS          SourceSerieGenre = "Martial Arts"
	MATURE                SourceSerieGenre = "Mature"
	MECHA                 SourceSerieGenre = "Mecha"
	MEDICAL               SourceSerieGenre = "Medical"
	MILITARY              SourceSerieGenre = "Military"
	MONSTER_GIRLS         SourceSerieGenre = "Monster Girls"
	MONSTERS              SourceSerieGenre = "Monsters"
	MUSIC                 SourceSerieGenre = "Music"
	MYSTERY               SourceSerieGenre = "Mystery"
	NINJA                 SourceSerieGenre = "Ninja"
	OFFICE_WORKERS        SourceSerieGenre = "Office Workers"
	OFFICIAL_COLORED      SourceSerieGenre = "Official Colored"
	ONE_SHOT              SourceSerieGenre = "One Shot"
	PHILOSOPHICAL         SourceSerieGenre = "Philosophical"
	POLICE                SourceSerieGenre = "Police"
	POST_APOCALYPTIC      SourceSerieGenre = "Post-Apocalyptic"
	PSYCHOLOGICAL         SourceSerieGenre = "Psychological"
	PSYCHOLOGICAL_ROMANCE SourceSerieGenre = "Psychological Romance"
	REINCARNATION         SourceSerieGenre = "Reincarnation"
	REVERSE_HAREM         SourceSerieGenre = "Reverse Harem"
	ROMANCE               SourceSerieGenre = "Romance"
	SAMURAI               SourceSerieGenre = "Samurai"
	SCHOOL_LIFE           SourceSerieGenre = "School Life"
	SCI_FI                SourceSerieGenre = "Sci-Fi"
	SEINEN                SourceSerieGenre = "Seinen"
	SELF_PUBLISHED        SourceSerieGenre = "Self Published"
	SEXUAL_VIOLENCE       SourceSerieGenre = "Sexual Violence"
	SHOTACON              SourceSerieGenre = "Shotacon"
	SHOUJO                SourceSerieGenre = "Shoujo"
	SHOUJO_AI             SourceSerieGenre = "Shoujo Ai"
	SHOUNEN               SourceSerieGenre = "Shounen"
	SHOUNEN_AI            SourceSerieGenre = "Shounen Ai"
	SLICE_OF_LIFE         SourceSerieGenre = "Slice of Life"
	SMUT                  SourceSerieGenre = "Smut"
	SPACE                 SourceSerieGenre = "Space"
	SPORTS                SourceSerieGenre = "Sports"
	SUPERHERO             SourceSerieGenre = "Superhero"
	SUPERNATURAL          SourceSerieGenre = "Supernatural"
	SURVIVAL              SourceSerieGenre = "Survival"
	SUSPENSE              SourceSerieGenre = "Suspense"
	THRILLER              SourceSerieGenre = "Thriller"
	TIME_TRAVEL           SourceSerieGenre = "Time Travel"
	TOOMICS               SourceSerieGenre = "Toomics"
	TRADITIONAL_GAMES     SourceSerieGenre = "Traditional Games"
	TRAGEDY               SourceSerieGenre = "Tragedy"
	VAMPIRES              SourceSerieGenre = "Vampires"
	VIDEO_GAMES           SourceSerieGenre = "Video Games"
	VILLAINESS            SourceSerieGenre = "Villainess"
	VIRTUAL_REALITY       SourceSerieGenre = "Virtual Reality"
	WEB_COMIC             SourceSerieGenre = "Web Comic"
	WUXIA                 SourceSerieGenre = "Wuxia"
	YAOI                  SourceSerieGenre = "Yaoi"
	YURI                  SourceSerieGenre = "Yuri"
	ZOMBIES               SourceSerieGenre = "Zombies"
)

type SourceSerieVolumeChapterImage struct {
	Index int      `json:"index"`
	URL   *url.URL `json:"url"`
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
	Images []SourceSerieVolumeChapterImage  `json:"images"`
	Texts  []SourceSerieVolumeChapterText   `json:"texts"`
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

type MultiLanguageString struct {
	EN    string `json:"en"`
	JP    string `json:"jp"`
	JP_RO string `json:"jp_ro"`
	FR    string `json:"fr"`
	KR    string `json:"kr"`
	CH    string `json:"ch"`
}

type SourceSerie struct {
	ID                SourceSerieID         `json:"id"`
	Title             MultiLanguageString   `json:"title"`
	AlternativeTitles []MultiLanguageString `json:"alternativeTitles"`
	Cover             *url.URL              `json:"cover"`
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
	Cover *url.URL            `json:"cover"`
}

type SourcePaginatedSmallSerie struct {
	HasNextPage bool               `json:"hasNextPage"`
	Series      []SourceSmallSerie `json:"series"`
}

type FetchSearchSerieFilterOrder string

const (
	ASC  FetchSearchSerieFilterOrder = "asc"
	DESC FetchSearchSerieFilterOrder = "desc"
)

type FetchSearchSerieFilterSort string

const (
	LATEST     FetchSearchSerieFilterSort = "Latest"
	POPULARITY FetchSearchSerieFilterSort = "Popularity"
	RELEVANCE  FetchSearchSerieFilterSort = "Relevance"
	ALPHABETIC FetchSearchSerieFilterSort = "Alphabetic"
)

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
	PossibleValues []SourceSerieGenre `json:"possibleValues"`
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
	Icon          string           `json:"iconURL"`
	Languages     []SourceLanguage `json:"languages"`
	UpdatedAt     time.Time        `json:"updatedAt"`
	Version       string           `json:"version"`
	NSFW          bool             `json:"nsfw"`
	SearchFilters SupportedFilters `json:"searchFilters"`
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
	SerieUrl(serieID SourceSerieID) *url.URL
}
