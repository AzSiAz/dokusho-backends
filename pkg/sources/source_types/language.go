package source_types

import "strings"

type SourceLanguage string

const (
	EN  SourceLanguage = "en"
	JP  SourceLanguage = "jp"
	FR  SourceLanguage = "fr"
	KR  SourceLanguage = "kr"
	CH  SourceLanguage = "ch"
	ALL SourceLanguage = "all"
)

func (s SourceLanguage) String() string {
	return string(s)
}

func NewSourceLanguage(s string) SourceLanguage {
	s = strings.Trim(s, "")

	switch s {
	case "en":
		return EN
	case "jp":
		return JP
	case "fr":
		return FR
	case "kr":
		return KR
	case "ch":
		return CH
	case "all":
		return ALL
	default:
		return EN
	}
}

type MultiLanguageString struct {
	EN    string `json:"en,omitempty"`
	JP    string `json:"jp,omitempty"`
	JP_RO string `json:"jp_ro,omitempty"`
	FR    string `json:"fr,omitempty"`
	KR    string `json:"kr,omitempty"`
	CH    string `json:"ch,omitempty"`
}
