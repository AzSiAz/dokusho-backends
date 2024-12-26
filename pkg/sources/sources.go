package sources

import (
	"dokusho/pkg/config"
	"dokusho/pkg/sources/mangadex"
	"dokusho/pkg/sources/mock"
	"dokusho/pkg/sources/source_types"
	"dokusho/pkg/sources/weebcentral"
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
