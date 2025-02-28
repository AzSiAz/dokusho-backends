package source_types

import "errors"

var (
	ErrInvalidSearchQuery  = errors.New("invalid search query")
	ErrInvalidSearchOrder  = errors.New("invalid search order")
	ErrInvalidSearchSort   = errors.New("invalid search sort")
	ErrInvalidSearchTypes  = errors.New("invalid search types")
	ErrInvalidSearchGenres = errors.New("invalid search genres")
	ErrInvalidSearchStatus = errors.New("invalid search status")
	ErrInvalidLanguage     = errors.New("invalid language")

	ErrInvalidSerieID = errors.New("invalid serie id")
	ErrInvalidCover   = errors.New("invalid cover")

	ErrBuildingRequest   = errors.New("error building request")
	ErrHTTPRequestFailed = errors.New("http request failed")
	ErrParsingHTML       = errors.New("error parsing html")
	ErrParsingJSON       = errors.New("error parsing json")
	ErrParsingURL        = errors.New("error parsing url")
	ErrExtractingData    = errors.New("error extracting data")
	ErrTimeout           = errors.New("timeout")

	ErrBuildingURL = errors.New("error building url")
)
