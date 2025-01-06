package sources

import (
	"dokusho/pkg/sources/mangadex"
	"dokusho/pkg/sources/source_types"
	"dokusho/pkg/sources/weebcentral"
)

func BuildSources() []source_types.SourceAPI {
	return []source_types.SourceAPI{
		weebcentral.NewWeebCentral(),
		mangadex.NewMangadex(),
	}
}
