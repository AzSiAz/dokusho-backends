package source_types

import "strings"

type FetchSearchSerieFilterSort string

const (
	LATEST     FetchSearchSerieFilterSort = "Latest"
	POPULARITY FetchSearchSerieFilterSort = "Popularity"
	RELEVANCE  FetchSearchSerieFilterSort = "Relevance"
	ALPHABETIC FetchSearchSerieFilterSort = "Alphabetic"
)

func NewFetchSearchSerieFilterSort(sort string) FetchSearchSerieFilterSort {
	sort = strings.Trim(sort, "")

	switch sort {
	case "Latest":
		return LATEST
	case "Popularity":
		return POPULARITY
	case "Relevance":
		return RELEVANCE
	case "Alphabetic":
		return ALPHABETIC
	default:
		return LATEST
	}
}
