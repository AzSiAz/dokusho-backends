package sources

import (
	"dokusho/pkg/config"
	"dokusho/pkg/sources/mock"
	"dokusho/pkg/sources/scrapers/mangadex"
	"dokusho/pkg/sources/scrapers/weebcentral"
	"dokusho/pkg/sources/source_types"
)

func BuildSources(cfg *config.SourceBaseConfig) []source_types.SourceAPI {
	sources := []source_types.SourceAPI{
		weebcentral.NewWeebCentral(),
		mangadex.NewMangadex(),
	}

	if cfg.SourceUseMock {
		sources = append(sources, mock.NewMockSource())
	}

	return sources
}
