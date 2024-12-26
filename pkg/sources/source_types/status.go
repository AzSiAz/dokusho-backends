package source_types

import "strings"

type SourceSerieStatus string

const (
	STATUS_ONGOING    SourceSerieStatus = "ongoing"
	STATUS_COMPLETED  SourceSerieStatus = "completed"
	STATUS_HIATUS     SourceSerieStatus = "hiatus"
	STATUS_CANCELED   SourceSerieStatus = "canceled"
	STATUS_PUBLISHING SourceSerieStatus = "publishing"
	STATUS_PUBLISHED  SourceSerieStatus = "published"
	STATUS_SCANLATING SourceSerieStatus = "scanlating"
	STATUS_SCANLATED  SourceSerieStatus = "scanlated"
	STATUS_UNKNOWN    SourceSerieStatus = "unknown"
)

func (s SourceSerieStatus) String() string {
	return string(s)
}

func NewSourceSerieStatus(s string) SourceSerieStatus {
	s = strings.Trim(s, "")

	switch s {
	case "ongoing":
		return STATUS_ONGOING
	case "completed":
		return STATUS_COMPLETED
	case "hiatus":
		return STATUS_HIATUS
	case "canceled":
		return STATUS_CANCELED
	case "publishing":
		return STATUS_PUBLISHING
	case "publishing_done":
		return STATUS_PUBLISHED
	case "scanlating":
		return STATUS_SCANLATING
	case "scanlating_done":
		return STATUS_SCANLATED
	default:
		return STATUS_UNKNOWN
	}
}
