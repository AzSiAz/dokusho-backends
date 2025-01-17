package sources

import (
	"dokusho/pkg/sources/mangadex"
	"dokusho/pkg/sources/mock"
	"dokusho/pkg/sources/source_types"
	"dokusho/pkg/sources/weebcentral"
)

func BuildSources(includeMock bool) []source_types.SourceAPI {
	sources := []source_types.SourceAPI{
		weebcentral.NewWeebCentral(),
		mangadex.NewMangadex(),
	}

	if includeMock {
		sources = append(sources, mock.NewMockSource())
	}

	return sources
}
