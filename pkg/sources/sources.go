package sources

import (
	"dokusho/pkg/sources/source_types"
	"dokusho/pkg/sources/weebcentral"
	"log/slog"
)

func BuildSources(logger *slog.Logger) []source_types.SourceAPI {
	return []source_types.SourceAPI{
		weebcentral.NewWeebCentral(logger),
	}
}
