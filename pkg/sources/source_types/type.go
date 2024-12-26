package source_types

import "strings"

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

func (t SourceSerieType) String() string {
	return string(t)
}

func NewSourceSerieType(t string) SourceSerieType {
	t = strings.Trim(t, "")

	switch t {
	case "manga":
		return TYPE_MANGA
	case "manhwa":
		return TYPE_MANHWA
	case "manhua":
		return TYPE_MANHUA
	case "webtoon":
		return TYPE_WEBTOON
	case "lightnovel":
		return TYPE_LIGHTNOVEL
	case "novel":
		return TYPE_NOVEL
	case "doujinshi":
		return TYPE_DOUJINSHI
	case "comic":
		return TYPE_COMIC
	case "oel":
		return TYPE_OEL
	default:
		return TYPE_UNKNOWN
	}
}
