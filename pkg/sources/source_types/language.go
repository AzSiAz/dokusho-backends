package source_types

import "strings"

type SourceLanguage string

const (
	EN    SourceLanguage = "en"
	JP    SourceLanguage = "jp"
	FR    SourceLanguage = "fr"
	KO    SourceLanguage = "ko"
	ZH_HK SourceLanguage = "zh-hk"
	ZH    SourceLanguage = "zh"
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
	case "ko":
		return KO
	case "zh-hk":
		return ZH_HK
	case "zh":
		return ZH
	default:
		return EN
	}
}

type MultiLanguageString struct {
	EN    string `json:"en,omitempty"`
	JP    string `json:"jp,omitempty"`
	JP_RO string `json:"jp_ro,omitempty"`
	FR    string `json:"fr,omitempty"`
	KO    string `json:"ko,omitempty"`
	ZH    string `json:"zh,omitempty"`
	ZH_HK string `json:"zh_hk,omitempty"`
}
