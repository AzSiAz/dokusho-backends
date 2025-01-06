package source_types

import "strings"

type FetchSearchSerieFilterOrder string

const (
	ASC  FetchSearchSerieFilterOrder = "asc"
	DESC FetchSearchSerieFilterOrder = "desc"
)

func NewFetchSearchSerieFilterOrder(order string) FetchSearchSerieFilterOrder {
	order = strings.Trim(order, "")

	switch order {
	case "asc":
		return ASC
	case "desc":
		return DESC
	default:
		return ASC
	}
}
