package weebcentral

import "errors"

var (
	ErrInvalidStatus = errors.New("invalid status")
	ErrInvalidSort   = errors.New("invalid sort")
	ErrInvalidType   = errors.New("invalid type")
	ErrInvalidGenre  = errors.New("invalid genre")
	ErrInvalidOrder  = errors.New("invalid order")
)
