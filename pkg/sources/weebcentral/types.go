package weebcentral

import (
	"errors"
	"fmt"

	sources "dokusho/pkg/sources/source_types"
)

type WeebCentralGenre string

const (
	ACTION        WeebCentralGenre = "Action"
	ADULT         WeebCentralGenre = "Adult"
	ADVENTURE     WeebCentralGenre = "Adventure"
	COMEDY        WeebCentralGenre = "Comedy"
	DOUJINSHI     WeebCentralGenre = "Doujinshi"
	DRAMA         WeebCentralGenre = "Drama"
	ECCHI         WeebCentralGenre = "Ecchi"
	FANTASY       WeebCentralGenre = "Fantasy"
	GENDER_BENDER WeebCentralGenre = "Gender Bender"
	HAREM         WeebCentralGenre = "Harem"
	HENTAI        WeebCentralGenre = "Hentai"
	HISTORICAL    WeebCentralGenre = "Historical"
	HORROR        WeebCentralGenre = "Horror"
	ISEKAI        WeebCentralGenre = "Isekai"
	JOSEI         WeebCentralGenre = "Josei"
	LOLICON       WeebCentralGenre = "Lolicon"
	MARTIAL_ARTS  WeebCentralGenre = "Martial Arts"
	MATURE        WeebCentralGenre = "Mature"
	MECHA         WeebCentralGenre = "Mecha"
	MYSTERY       WeebCentralGenre = "Mystery"
	PSYCHOLOGICAL WeebCentralGenre = "Psychological"
	ROMANCE       WeebCentralGenre = "Romance"
	SCHOOL_LIFE   WeebCentralGenre = "School Life"
	SCI_FI        WeebCentralGenre = "Sci-fi"
	SEINEN        WeebCentralGenre = "Seinen"
	SHOTACON      WeebCentralGenre = "Shotacon"
	SHOUJO        WeebCentralGenre = "Shoujo"
	SHOUJO_AI     WeebCentralGenre = "Shoujo Ai"
	SHOUNEN       WeebCentralGenre = "Shounen"
	SHOUNEN_AI    WeebCentralGenre = "Shounen Ai"
	SLICE_OF_LIFE WeebCentralGenre = "Slice of Life"
	SMUT          WeebCentralGenre = "Smut"
	SPORTS        WeebCentralGenre = "Sports"
	SUPERNATURAL  WeebCentralGenre = "Supernatural"
	TRAGEDY       WeebCentralGenre = "Tragedy"
	YAOI          WeebCentralGenre = "Yaoi"
	YURI          WeebCentralGenre = "Yuri"
	OTHER         WeebCentralGenre = "Other"
)

var WEEBCENTRAL_TO_SOURCE_SERIE_GENRE = map[WeebCentralGenre]sources.SourceSerieGenre{
	ACTION:        sources.ACTION,
	ADULT:         sources.ADULT,
	ADVENTURE:     sources.ADVENTURE,
	COMEDY:        sources.COMEDY,
	DOUJINSHI:     sources.DOUJINSHI,
	DRAMA:         sources.DRAMA,
	ECCHI:         sources.ECCHI,
	FANTASY:       sources.FANTASY,
	GENDER_BENDER: sources.GENDER_BENDER,
	HAREM:         sources.HAREM,
	HENTAI:        sources.HENTAI,
	HISTORICAL:    sources.HISTORICAL,
	HORROR:        sources.HORROR,
	ISEKAI:        sources.ISEKAI,
	JOSEI:         sources.JOSEI,
	LOLICON:       sources.LOLICON,
	MARTIAL_ARTS:  sources.MARTIAL_ARTS,
	MATURE:        sources.MATURE,
	MECHA:         sources.MECHA,
	MYSTERY:       sources.MYSTERY,
	PSYCHOLOGICAL: sources.PSYCHOLOGICAL,
	ROMANCE:       sources.ROMANCE,
	SCHOOL_LIFE:   sources.SCHOOL_LIFE,
	SCI_FI:        sources.SCI_FI,
	SEINEN:        sources.SEINEN,
	SHOTACON:      sources.SHOTACON,
	SHOUJO:        sources.SHOUJO,
	SHOUJO_AI:     sources.SHOUJO_AI,
	SHOUNEN:       sources.SHOUNEN,
	SHOUNEN_AI:    sources.SHOUNEN_AI,
	SLICE_OF_LIFE: sources.SLICE_OF_LIFE,
	SMUT:          sources.SMUT,
	SPORTS:        sources.SPORTS,
	SUPERNATURAL:  sources.SUPERNATURAL,
	TRAGEDY:       sources.TRAGEDY,
	YAOI:          sources.YAOI,
	YURI:          sources.YURI,
	OTHER:         sources.OTHER,
}

func GetSearchableGenres() []sources.SourceSerieGenre {
	var genres []sources.SourceSerieGenre

	for _, genre := range WEEBCENTRAL_TO_SOURCE_SERIE_GENRE {
		genres = append(genres, genre)
	}

	return genres
}

func ConvertWeebCentralGenre(genre WeebCentralGenre) (sources.SourceSerieGenre, error) {
	if v, ok := WEEBCENTRAL_TO_SOURCE_SERIE_GENRE[genre]; ok {
		return v, nil
	}

	return sources.OTHER, fmt.Errorf("Genre %s is not a valid WeebCentral genre for Source: %w", genre, ErrInvalidGenre)
}

func ConvertWeebCentralGenres(genres []WeebCentralGenre) ([]sources.SourceSerieGenre, error) {
	var error error
	var sourceGenres []sources.SourceSerieGenre

	for _, genre := range genres {
		if sourceGenre, err := ConvertWeebCentralGenre(genre); err != nil {
			errors.Join(error, err)
		} else {
			sourceGenres = append(sourceGenres, sourceGenre)
		}
	}

	return sourceGenres, error
}

func ConvertSourceSerieGenre(genre sources.SourceSerieGenre) (WeebCentralGenre, error) {
	for k, v := range WEEBCENTRAL_TO_SOURCE_SERIE_GENRE {
		if v == genre {
			return k, nil
		}
	}

	return "", fmt.Errorf("Genre %s is not a valid Source genre for WeebCentral: %w", genre, ErrInvalidGenre)
}

func ConvertSourceSerieGenres(genres []sources.SourceSerieGenre) ([]WeebCentralGenre, error) {
	var error error
	var weebCentralGenres []WeebCentralGenre

	for _, genre := range genres {
		if weebCentralGenre, err := ConvertSourceSerieGenre(genre); err != nil {
			errors.Join(error, err)
		} else {
			weebCentralGenres = append(weebCentralGenres, weebCentralGenre)
		}
	}

	return weebCentralGenres, error
}

type WeebCentralType string

const (
	MANGA  WeebCentralType = "Manga"
	MANHWA WeebCentralType = "Manhwa"
	MANHUA WeebCentralType = "Manhua"
	OEL    WeebCentralType = "OEL"
)

var WEEBCENTRAL_TO_SOURCE_SERIE_TYPE = map[WeebCentralType]sources.SourceSerieType{
	MANGA:  sources.TYPE_MANGA,
	MANHWA: sources.TYPE_MANHWA,
	MANHUA: sources.TYPE_MANHUA,
	OEL:    sources.TYPE_OEL,
}

func GetSearchableTypes() []sources.SourceSerieType {
	var types []sources.SourceSerieType

	for _, t := range WEEBCENTRAL_TO_SOURCE_SERIE_TYPE {
		types = append(types, t)
	}

	return types
}

func ConvertWeebCentralType(t WeebCentralType) (sources.SourceSerieType, error) {
	if v, ok := WEEBCENTRAL_TO_SOURCE_SERIE_TYPE[t]; ok {
		return v, nil
	}

	return sources.TYPE_UNKNOWN, fmt.Errorf("Type %s is not a valid WeebCentral type for Source: %w", t, ErrInvalidType)
}

func ConvertSourceSerieType(t sources.SourceSerieType) (WeebCentralType, error) {
	for k, v := range WEEBCENTRAL_TO_SOURCE_SERIE_TYPE {
		if v == t {
			return k, nil
		}
	}

	return "", fmt.Errorf("Type %s is not a valid Source type for WeebCentral: %w", t, ErrInvalidType)
}

func ConvertSourceSerieTypes(types []sources.SourceSerieType) ([]WeebCentralType, error) {
	var error error
	var weebCentralTypes []WeebCentralType

	for _, t := range types {
		if weebCentralType, err := ConvertSourceSerieType(t); err != nil {
			errors.Join(error, err)
		} else {
			weebCentralTypes = append(weebCentralTypes, weebCentralType)
		}
	}

	return weebCentralTypes, error
}

type WeebCentralStatus string

const (
	ONGOING  WeebCentralStatus = "Ongoing"
	COMPLETE WeebCentralStatus = "Complete"
	HIATUS   WeebCentralStatus = "Hiatus"
	CANCELED WeebCentralStatus = "Canceled"
)

var WEEBCENTRAL_TO_SOURCE_SERIE_STATUS = map[WeebCentralStatus]sources.SourceSerieStatus{
	ONGOING:  sources.STATUS_ONGOING,
	COMPLETE: sources.STATUS_COMPLETED,
	HIATUS:   sources.STATUS_HIATUS,
	CANCELED: sources.STATUS_CANCELED,
}

func GetSearchableStatus() []sources.SourceSerieStatus {
	var statuses []sources.SourceSerieStatus

	for _, status := range WEEBCENTRAL_TO_SOURCE_SERIE_STATUS {
		statuses = append(statuses, status)
	}

	return statuses
}

func ConvertWeebCentralStatus(status WeebCentralStatus) (sources.SourceSerieStatus, error) {
	if v, ok := WEEBCENTRAL_TO_SOURCE_SERIE_STATUS[status]; ok {
		return v, nil
	}

	return sources.STATUS_UNKNOWN, fmt.Errorf("Status %s is not a valid WeebCentral status for Source: %w", status, ErrInvalidStatus)
}

func ConvertSourceSerieStatus(status sources.SourceSerieStatus) (WeebCentralStatus, error) {
	for k, v := range WEEBCENTRAL_TO_SOURCE_SERIE_STATUS {
		if v == status {
			return k, nil
		}
	}

	return "", fmt.Errorf("Status %s is not a valid Source status for WeebCentral: %w", status, ErrInvalidStatus)
}

func ConvertSourceSerieStatuses(statuses []sources.SourceSerieStatus) ([]WeebCentralStatus, error) {
	var error error
	var weebCentralStatuses []WeebCentralStatus

	for _, status := range statuses {
		if weebCentralStatus, err := ConvertSourceSerieStatus(status); err != nil {
			errors.Join(error, err)
		} else {
			weebCentralStatuses = append(weebCentralStatuses, weebCentralStatus)
		}
	}

	return weebCentralStatuses, error
}

type WeebCentralSort string

const (
	BEST_MATCH   WeebCentralSort = "Best Match"
	POPULARITY   WeebCentralSort = "Popularity"
	LAST_UPDATED WeebCentralSort = "Latest Updates"
	ALPHABET     WeebCentralSort = "Alphabet"
)

var WEEBCENTRAL_TO_SOURCE_SERIE_SORT = map[WeebCentralSort]sources.FetchSearchSerieFilterSort{
	BEST_MATCH:   sources.RELEVANCE,
	POPULARITY:   sources.POPULARITY,
	LAST_UPDATED: sources.LATEST,
	ALPHABET:     sources.ALPHABETIC,
}

func GetSearchableSorts() []sources.FetchSearchSerieFilterSort {
	var sorts []sources.FetchSearchSerieFilterSort

	for _, sort := range WEEBCENTRAL_TO_SOURCE_SERIE_SORT {
		sorts = append(sorts, sort)
	}

	return sorts
}

func ConvertWeebCentralSort(sort WeebCentralSort) (sources.FetchSearchSerieFilterSort, error) {
	if v, ok := WEEBCENTRAL_TO_SOURCE_SERIE_SORT[sort]; ok {
		return v, nil
	}

	return sources.RELEVANCE, fmt.Errorf("Sort %s is not a valid WeebCentral sort for Source: %w", sort, ErrInvalidSort)
}

func ConvertSourceSerieSort(sort sources.FetchSearchSerieFilterSort) (WeebCentralSort, error) {
	for k, v := range WEEBCENTRAL_TO_SOURCE_SERIE_SORT {
		if v == sort {
			return k, nil
		}
	}

	return "", fmt.Errorf("Sort %s is not a valid Source sort for WeebCentral: %w", sort, ErrInvalidSort)
}

type WeebCentralOrder string

const (
	ASC  WeebCentralOrder = "Ascending"
	DESC WeebCentralOrder = "Descending"
)

var WEEBCENTRAL_TO_SOURCE_SERIE_ORDER = map[WeebCentralOrder]sources.FetchSearchSerieFilterOrder{
	ASC:  sources.ASC,
	DESC: sources.DESC,
}

func GetSearchableOrders() []sources.FetchSearchSerieFilterOrder {
	var orders []sources.FetchSearchSerieFilterOrder

	for _, order := range WEEBCENTRAL_TO_SOURCE_SERIE_ORDER {
		orders = append(orders, order)
	}

	return orders
}

func ConvertWeebCentralOrder(order WeebCentralOrder) (sources.FetchSearchSerieFilterOrder, error) {
	if v, ok := WEEBCENTRAL_TO_SOURCE_SERIE_ORDER[order]; ok {
		return v, nil
	}

	return sources.ASC, fmt.Errorf("Order %s is not a valid WeebCentral order for Source: %w", order, ErrInvalidOrder)
}

func ConvertSourceSerieOrder(order sources.FetchSearchSerieFilterOrder) (WeebCentralOrder, error) {
	for k, v := range WEEBCENTRAL_TO_SOURCE_SERIE_ORDER {
		if v == order {
			return k, nil
		}
	}

	return "", fmt.Errorf("Order %s is not a valid Source order for WeebCentral: %w", order, ErrInvalidOrder)
}
