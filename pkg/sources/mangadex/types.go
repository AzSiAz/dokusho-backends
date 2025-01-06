package mangadex

import (
	"dokusho/pkg/sources/source_types"
	"errors"
	"fmt"
	"strings"
)

type MangadexGenre string

const (
	GYARU             MangadexGenre = "fad12b5e-68ba-460e-b933-9ae8318f5b65"
	TRAGEDY           MangadexGenre = "f8f62932-27da-4fe4-8ee1-6779a8c5edba"
	FULL_COLOR        MangadexGenre = "f5ba408b-0e7a-484d-8d49-4e9125ac96de"
	MUSIC             MangadexGenre = "f42fbf9e-188a-447b-9fdc-f19dc1e4d685"
	ADAPTATION        MangadexGenre = "f4122d1c-3b44-44d0-9936-ff7502c39ad3"
	MYSTERY           MangadexGenre = "ee968100-4191-4968-93d3-f82d72be7e46"
	SUPERNATURAL      MangadexGenre = "eabc5b4c-6aff-42f3-b657-3e90cbd00b75"
	COOKING           MangadexGenre = "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869"
	ALIENS            MangadexGenre = "e64f6742-c834-471d-8d72-dd51fc02b835"
	SLICE_OF_LIFE     MangadexGenre = "e5301a23-ebd9-49dd-a0cb-2add944c7fe9"
	WEB_COMIC         MangadexGenre = "e197df38-d0e7-43b5-9b09-2842d0c326dd"
	POLICE            MangadexGenre = "df33b754-73a3-4c54-80e6-1a74a8058539"
	AWARD_WINNING     MangadexGenre = "0a39b5a1-b235-4886-a747-1d05d216532d"
	REINCARNATION     MangadexGenre = "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0"
	GENDERSWAP        MangadexGenre = "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a"
	LOLICON           MangadexGenre = "2d1f5d56-a1e5-4d0d-a961-2193588b08ec"
	PSYCHOLOGICAL     MangadexGenre = "3b60b75c-a2d7-4860-ab56-05f391bb889c"
	GHOST             MangadexGenre = "3bb26d85-09d5-4d2e-880c-c34b974339e9"
	ANIMALS           MangadexGenre = "3de8c75d-8ee3-48ff-98ee-e20a65c86451"
	LONG_STRIP        MangadexGenre = "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d"
	COMEDY            MangadexGenre = "4d32cc48-9f00-4cca-9b5a-a839f0764984"
	INCEST            MangadexGenre = "5bd0e105-4481-44ca-b6e7-7544da56b1a3"
	CRIME             MangadexGenre = "5ca48985-9a9d-4bd8-be29-80dc0303db72"
	SURVIVAL          MangadexGenre = "5fff9cde-849c-4d78-aab0-0d52b2ee1d25"
	FAN_COLORED       MangadexGenre = "7b2ce280-79ef-4c09-9b58-12b7c23a9b78"
	VIRTUAL_REALITY   MangadexGenre = "8c86611e-fab7-4986-9dec-d1a2f44acdd5"
	CROSSDRESSING     MangadexGenre = "9ab53f92-3eed-4e9b-903a-917c86035ee3"
	MONSTERS          MangadexGenre = "36fd93ea-e8b8-445e-b836-358f02b3d33d"
	ANTHOLOGY         MangadexGenre = "51d83883-4103-437c-b4b1-731cb73d786c"
	MAGICAL_GIRLS     MangadexGenre = "81c836c9-914a-4eca-981a-560dad663e73"
	MAFIA             MangadexGenre = "85daba54-a71c-4554-8a28-9901a8b0afad"
	ADVENTURE         MangadexGenre = "87cc87cd-a395-47af-b27a-93258283bbc6"
	OFFICE_WORKERS    MangadexGenre = "92d6d951-ca5e-429c-ac78-451071cbf064"
	ONE_SHOT          MangadexGenre = "0234a31e-a729-4e28-9d6a-3f87c4966b9e"
	SCI_FI            MangadexGenre = "256c8bd9-4904-4360-bf4f-508a76d67183"
	TIME_TRAVEL       MangadexGenre = "292e862b-2d17-4062-90a2-0356caa4ae27"
	ACTION            MangadexGenre = "391b0423-d847-456f-aff0-8b0cfc03066b"
	ROMANCE           MangadexGenre = "423e2eae-a7a2-4a8b-ac03-a8351462d71d"
	NINJA             MangadexGenre = "489dd859-9b61-4c37-af75-5b18e88daafc"
	ZOMBIES           MangadexGenre = "631ef465-9aba-4afb-b0fc-ea10efe274a8"
	MARTIAL_ARTS      MangadexGenre = "799c202e-7daa-44eb-9cf7-8a3c0441531e"
	SELF_PUBLISHED    MangadexGenre = "891cf039-b895-47f0-9229-bef4c96eccd4"
	BOYS_LOVE         MangadexGenre = "5920b825-4181-4a17-beeb-9918b0ff7a30"
	SUPERHERO         MangadexGenre = "7064a261-a137-4d3a-8848-2d385de3a99c"
	VIDEO_GAMES       MangadexGenre = "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8"
	TRADITIONAL_GAMES MangadexGenre = "31932a7e-5b8e-49a6-9f12-2afa39dc544c"
	MECHA             MangadexGenre = "50880a9d-5440-4732-9afb-8f457127e836"
	REVERSE_HAREM     MangadexGenre = "65761a2a-415e-47f3-bef2-a9dababba7a6"
	SPORTS            MangadexGenre = "69964a64-2f90-4d33-beeb-f3ed2875eb4c"
	SEXUAL_VIOLENCE   MangadexGenre = "97893a4c-12af-4dac-b6be-0dffb353568e"
	OFFICIAL_COLORED  MangadexGenre = "320831a8-4026-470b-94f6-8353740e6f04"
	THRILLER          MangadexGenre = "07251805-a27e-4d59-b488-f0bfbec15168"
	POST_APOCALYPTIC  MangadexGenre = "9467335a-1b83-4497-9231-765337a00b96"
	HISTORICAL        MangadexGenre = "33771934-028e-4cb3-8744-691e866a923e"
	DEMONS            MangadexGenre = "39730448-9a5f-48a2-85b0-a70db87b1233"
	SAMURAI           MangadexGenre = "81183756-1453-4c81-aa9e-f6e1b63be016"
	MAGIC             MangadexGenre = "a1f53773-c69a-4ce5-8cab-fffcd90b1565"
	GIRLS_LOVE        MangadexGenre = "a3c67850-4684-404e-9b7f-c69850ee5da6"
	HAREM             MangadexGenre = "aafb99c1-7f60-43fa-b75f-fc9502ce29c7"
	MILITARY          MangadexGenre = "ac72833b-c4e9-4878-b9db-6c8a4a99444a"
	WUXIA             MangadexGenre = "acc803a4-c95a-4c22-86fc-eb6b582d82a2"
	ISEKAI            MangadexGenre = "ace04997-f6bd-436e-b261-779182193d3d"
	PHILOSOPHICAL     MangadexGenre = "b1e97889-25b4-4258-b28b-cd7f4d28ea9b"
	DRAMA             MangadexGenre = "b9af3a63-f058-46de-a9a0-e0c13906197a"
	FOUR_KOMA         MangadexGenre = "b11fda93-8f1d-4bef-b2ed-8803d3733170"
	DOUJINSHI         MangadexGenre = "b13b2a48-c720-44a9-9c77-39c9979373fb"
	GORE              MangadexGenre = "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d"
	MEDICAL           MangadexGenre = "c8cbe35b-1b2b-4a3f-9c37-db84c4514856"
	SCHOOL_LIFE       MangadexGenre = "caaa44eb-cd40-4177-b930-79d3ef2afe87"
	HORROR            MangadexGenre = "cdad7e68-1419-41dd-bdce-27753074a640"
	FANTASY           MangadexGenre = "cdc58593-87dd-415e-bbc0-2ec27bf404cc"
	VAMPIRES          MangadexGenre = "d7d1730f-6eb0-4ba6-9437-602cac38664c"
	VILLAINESS        MangadexGenre = "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41"
	DELINQUENTS       MangadexGenre = "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea"
	MONSTER_GIRLS     MangadexGenre = "dd1f77c5-dea9-4e2b-97ae-224af09caf99"
	SHOTACON          MangadexGenre = "ddefd648-5140-4e5f-ba18-4eca4071d19b"
)

var MANGADEX_TO_SOURCE_SERIE_GENRE = map[MangadexGenre]source_types.SourceSerieGenre{
	GYARU:             source_types.GYARU,
	TRAGEDY:           source_types.TRAGEDY,
	FULL_COLOR:        source_types.FULL_COLOR,
	MUSIC:             source_types.MUSIC,
	ADAPTATION:        source_types.ADAPTATION,
	MYSTERY:           source_types.MYSTERY,
	SUPERNATURAL:      source_types.SUPERNATURAL,
	COOKING:           source_types.COOKING,
	ALIENS:            source_types.ALIENS,
	SLICE_OF_LIFE:     source_types.SLICE_OF_LIFE,
	WEB_COMIC:         source_types.WEB_COMIC,
	POLICE:            source_types.POLICE,
	AWARD_WINNING:     source_types.AWARD_WINNING,
	REINCARNATION:     source_types.REINCARNATION,
	GENDERSWAP:        source_types.GENDERSWAP,
	LOLICON:           source_types.LOLICON,
	PSYCHOLOGICAL:     source_types.PSYCHOLOGICAL,
	GHOST:             source_types.GHOST,
	ANIMALS:           source_types.ANIMALS,
	LONG_STRIP:        source_types.LONG_STRIP,
	COMEDY:            source_types.COMEDY,
	INCEST:            source_types.INCEST,
	CRIME:             source_types.CRIME,
	SURVIVAL:          source_types.SURVIVAL,
	FAN_COLORED:       source_types.FAN_COLORED,
	VIRTUAL_REALITY:   source_types.VIRTUAL_REALITY,
	CROSSDRESSING:     source_types.CROSSDRESSING,
	MONSTERS:          source_types.MONSTERS,
	ANTHOLOGY:         source_types.ANTHOLOGY,
	MAGICAL_GIRLS:     source_types.MAGICAL_GIRLS,
	MAFIA:             source_types.MAFIA,
	ADVENTURE:         source_types.ADVENTURE,
	OFFICE_WORKERS:    source_types.OFFICE_WORKERS,
	ONE_SHOT:          source_types.ONE_SHOT,
	SCI_FI:            source_types.SCI_FI,
	TIME_TRAVEL:       source_types.TIME_TRAVEL,
	ACTION:            source_types.ACTION,
	ROMANCE:           source_types.ROMANCE,
	NINJA:             source_types.NINJA,
	ZOMBIES:           source_types.ZOMBIES,
	MARTIAL_ARTS:      source_types.MARTIAL_ARTS,
	SELF_PUBLISHED:    source_types.SELF_PUBLISHED,
	BOYS_LOVE:         source_types.BOYS_LOVE,
	SUPERHERO:         source_types.SUPERHERO,
	VIDEO_GAMES:       source_types.VIDEO_GAMES,
	TRADITIONAL_GAMES: source_types.TRADITIONAL_GAMES,
	MECHA:             source_types.MECHA,
	REVERSE_HAREM:     source_types.REVERSE_HAREM,
	SPORTS:            source_types.SPORTS,
	SEXUAL_VIOLENCE:   source_types.SEXUAL_VIOLENCE,
	OFFICIAL_COLORED:  source_types.OFFICIAL_COLORED,
	THRILLER:          source_types.THRILLER,
	POST_APOCALYPTIC:  source_types.POST_APOCALYPTIC,
	HISTORICAL:        source_types.HISTORICAL,
	DEMONS:            source_types.DEMONS,
	SAMURAI:           source_types.SAMURAI,
	MAGIC:             source_types.MAGIC,
	GIRLS_LOVE:        source_types.GIRLS_LOVE,
	HAREM:             source_types.HAREM,
	MILITARY:          source_types.MILITARY,
	WUXIA:             source_types.WUXIA,
	ISEKAI:            source_types.ISEKAI,
	PHILOSOPHICAL:     source_types.PHILOSOPHICAL,
	DRAMA:             source_types.DRAMA,
	FOUR_KOMA:         source_types.FOUR_KOMA,
	DOUJINSHI:         source_types.DOUJINSHI,
	GORE:              source_types.GORE,
	MEDICAL:           source_types.MEDICAL,
	SCHOOL_LIFE:       source_types.SCHOOL_LIFE,
	HORROR:            source_types.HORROR,
	FANTASY:           source_types.FANTASY,
	VAMPIRES:          source_types.VAMPIRES,
	VILLAINESS:        source_types.VILLAINESS,
	DELINQUENTS:       source_types.DELINQUENTS,
	MONSTER_GIRLS:     source_types.MONSTER_GIRLS,
	SHOTACON:          source_types.SHOTACON,
}

func GetSearchableGenres() []source_types.SourceSerieGenre {
	var genres []source_types.SourceSerieGenre

	for _, genre := range MANGADEX_TO_SOURCE_SERIE_GENRE {
		genres = append(genres, genre)
	}

	return genres
}

func ConvertMangadexGenre(genre MangadexGenre) (source_types.SourceSerieGenre, error) {
	if v, ok := MANGADEX_TO_SOURCE_SERIE_GENRE[genre]; ok {
		return v, nil
	}

	return source_types.OTHER, fmt.Errorf("Genre %s is not a valid Mangadex genre for Source: %w", genre, ErrInvalidGenre)
}

func ConvertMangadexGenres(genres []MangadexGenre) ([]source_types.SourceSerieGenre, error) {
	var error error
	var sourceGenres []source_types.SourceSerieGenre

	for _, genre := range genres {
		if sourceGenre, err := ConvertMangadexGenre(genre); err != nil {
			errors.Join(error, err)
		} else {
			sourceGenres = append(sourceGenres, sourceGenre)
		}
	}

	return sourceGenres, error
}

func ConvertSourceSerieGenre(genre source_types.SourceSerieGenre) (MangadexGenre, error) {
	for k, v := range MANGADEX_TO_SOURCE_SERIE_GENRE {
		if v == genre {
			return k, nil
		}
	}

	return "", fmt.Errorf("Genre %s is not a valid Source genre for Mangadex: %w", genre, ErrInvalidGenre)
}

func ConvertSourceSerieGenres(genres []source_types.SourceSerieGenre) ([]MangadexGenre, error) {
	var error error
	var mangadexGenres []MangadexGenre

	for _, genre := range genres {
		if mangadexGenre, err := ConvertSourceSerieGenre(genre); err != nil {
			errors.Join(error, err)
		} else {
			mangadexGenres = append(mangadexGenres, mangadexGenre)
		}
	}

	return mangadexGenres, error
}

type MangadexStatus string

const (
	ONGOING   MangadexStatus = "ongoing"
	COMPLETED MangadexStatus = "completed"
	HIATUS    MangadexStatus = "hiatus"
	CANCELLED MangadexStatus = "cancelled"
	PUBLISHED MangadexStatus = "published"
	UNKNOWN   MangadexStatus = "unknown"
)

var MANGADEX_TO_SOURCE_SERIE_STATUS = map[MangadexStatus]source_types.SourceSerieStatus{
	ONGOING:   source_types.STATUS_ONGOING,
	COMPLETED: source_types.STATUS_COMPLETED,
	HIATUS:    source_types.STATUS_HIATUS,
	CANCELLED: source_types.STATUS_CANCELED,
	PUBLISHED: source_types.STATUS_PUBLISHED,
}

func GetSearchableStatus() []source_types.SourceSerieStatus {
	var statuses []source_types.SourceSerieStatus

	for _, status := range MANGADEX_TO_SOURCE_SERIE_STATUS {
		statuses = append(statuses, status)
	}

	return statuses
}

func ConvertMangadexStatus(status MangadexStatus) (source_types.SourceSerieStatus, error) {
	if v, ok := MANGADEX_TO_SOURCE_SERIE_STATUS[status]; ok {
		return v, nil
	}

	return source_types.STATUS_UNKNOWN, fmt.Errorf("Status %s is not a valid Mangadex status for Source: %w", status, ErrInvalidStatus)
}

func ConvertSourceSerieStatus(status source_types.SourceSerieStatus) (MangadexStatus, error) {
	for k, v := range MANGADEX_TO_SOURCE_SERIE_STATUS {
		if v == status {
			return k, nil
		}
	}

	return "", fmt.Errorf("Status %s is not a valid Source status for Mangadex: %w", status, ErrInvalidStatus)
}

func ConvertSourceSerieStatuses(statuses []source_types.SourceSerieStatus) ([]MangadexStatus, error) {
	var error error
	var mangadexStatuses []MangadexStatus

	for _, status := range statuses {
		if mangadexStatus, err := ConvertSourceSerieStatus(status); err != nil {
			errors.Join(error, err)
		} else {
			mangadexStatuses = append(mangadexStatuses, mangadexStatus)
		}
	}

	return mangadexStatuses, error
}

type MangadexSort string

const (
	FOLLOWER_COUNT          MangadexSort = "followedCount"
	LATEST_UPLOADED_CHAPTER MangadexSort = "latestUploadedChapter"
	RELEVANCE               MangadexSort = "relevance"
	TITLE                   MangadexSort = "title"
)

var MANGADEX_TO_SOURCE_SERIE_SORT = map[MangadexSort]source_types.FetchSearchSerieFilterSort{
	RELEVANCE:               source_types.RELEVANCE,
	FOLLOWER_COUNT:          source_types.POPULARITY,
	LATEST_UPLOADED_CHAPTER: source_types.LATEST,
	TITLE:                   source_types.ALPHABETIC,
}

func GetSearchableSorts() []source_types.FetchSearchSerieFilterSort {
	var sorts []source_types.FetchSearchSerieFilterSort

	for _, sort := range MANGADEX_TO_SOURCE_SERIE_SORT {
		sorts = append(sorts, sort)
	}

	return sorts
}

func ConvertMangadexSort(sort MangadexSort) (source_types.FetchSearchSerieFilterSort, error) {
	if v, ok := MANGADEX_TO_SOURCE_SERIE_SORT[sort]; ok {
		return v, nil
	}

	return source_types.RELEVANCE, fmt.Errorf("Sort %s is not a valid Mangadex sort for Source: %w", sort, ErrInvalidSort)
}

func ConvertSourceSerieSort(sort source_types.FetchSearchSerieFilterSort) (MangadexSort, error) {
	for k, v := range MANGADEX_TO_SOURCE_SERIE_SORT {
		if v == sort {
			return k, nil
		}
	}

	return "", fmt.Errorf("Sort %s is not a valid Source sort for Mangadex: %w", sort, ErrInvalidSort)
}

type MangadexOrder string

const (
	ASC  MangadexOrder = "asc"
	DESC MangadexOrder = "desc"
)

var MANGADEX_TO_SOURCE_SERIE_ORDER = map[MangadexOrder]source_types.FetchSearchSerieFilterOrder{
	ASC:  source_types.ASC,
	DESC: source_types.DESC,
}

func GetSearchableOrders() []source_types.FetchSearchSerieFilterOrder {
	var orders []source_types.FetchSearchSerieFilterOrder

	for _, order := range MANGADEX_TO_SOURCE_SERIE_ORDER {
		orders = append(orders, order)
	}

	return orders
}

func ConvertMangadexOrder(order MangadexOrder) (source_types.FetchSearchSerieFilterOrder, error) {
	if v, ok := MANGADEX_TO_SOURCE_SERIE_ORDER[order]; ok {
		return v, nil
	}

	return source_types.ASC, fmt.Errorf("Order %s is not a valid Mangadex order for Source: %w", order, ErrInvalidOrder)
}

func ConvertSourceSerieOrder(order source_types.FetchSearchSerieFilterOrder) (MangadexOrder, error) {
	for k, v := range MANGADEX_TO_SOURCE_SERIE_ORDER {
		if v == order {
			return k, nil
		}
	}

	return "", fmt.Errorf("Order %s is not a valid Source order for Mangadex: %w", order, ErrInvalidOrder)
}

type MangadexLanguage string

const (
	EN MangadexLanguage = "en"
	FR MangadexLanguage = "fr"
	// Korean
	KO MangadexLanguage = "ko"
	// Japanese
	JA MangadexLanguage = "ja"
	// Chinese (traditional)
	ZH_HK MangadexLanguage = "zh-hk"
	// Chinese (simplified)
	ZH MangadexLanguage = "zh"
)

func (t MangadexLanguage) String() string {
	return string(t)
}

func NewMangadexLanguage(language string) (MangadexLanguage, error) {
	language = strings.Trim(language, "")

	switch language {
	case "en":
		return EN, nil
	case "fr":
		return FR, nil
	case "ko":
		return KO, nil
	case "ja":
		return JA, nil
	case "zh-hk":
		return ZH_HK, nil
	case "zh":
		return ZH, nil
	default:
		return "", fmt.Errorf("Language %s is not a supported Mangadex language: %w", language, source_types.ErrInvalidLanguage)
	}
}

var MANGADEX_TO_SOURCE_SERIE_LANGUAGE = map[MangadexLanguage]source_types.SourceLanguage{
	EN:    source_types.EN,
	FR:    source_types.FR,
	KO:    source_types.KO,
	JA:    source_types.JP,
	ZH_HK: source_types.ZH_HK,
	ZH:    source_types.ZH,
}

func ConvertMangadexLanguage(language MangadexLanguage) (source_types.SourceLanguage, error) {
	if v, ok := MANGADEX_TO_SOURCE_SERIE_LANGUAGE[language]; ok {
		return v, nil
	}

	return "", fmt.Errorf("Language %s is not a valid Mangadex language for Source: %w", language, source_types.ErrInvalidLanguage)
}

func ConvertSourceSerieLanguage(language source_types.SourceLanguage) (MangadexLanguage, error) {
	for k, v := range MANGADEX_TO_SOURCE_SERIE_LANGUAGE {
		if v == language {
			return k, nil
		}
	}

	return "", fmt.Errorf("Language %s is not a valid Source language for Mangadex: %w", language, source_types.ErrInvalidLanguage)
}
