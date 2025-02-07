package sources

import (
	"dokusho/pkg/config"
	"dokusho/pkg/sources/mangadex"
	"dokusho/pkg/sources/mock"
	"dokusho/pkg/sources/source_types"
	"dokusho/pkg/sources/weebcentral"
)

type SourceConfig struct {
	UseFlareSolver bool
	FlareSolverURL string
	FileServerURL  string
}

func BuildSources(cfg config.SourceConfig) []source_types.SourceAPI {
	sources := []source_types.SourceAPI{
		weebcentral.NewWeebCentral(),
		mangadex.NewMangadex(),
	}

	if cfg.SourceUseMock {
		sources = append(sources, mock.NewMockSource())
	}

	return sources
}
