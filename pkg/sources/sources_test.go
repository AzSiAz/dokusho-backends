package sources_test

import (
	"dokusho/pkg/sources"
	"testing"
)

func TestGetSources(t *testing.T) {
	t.Parallel()

	sources := sources.BuildSources()

	if len(sources) == 0 {
		t.Error("No sources found")
	}

	for _, source := range sources {
		t.Run(string(source.GetInformation().ID), func(t *testing.T) {
			t.Parallel()

			if source.GetInformation().ID == "" {
				t.Error("ID is empty")
			}

			if source.GetInformation().Name == "" {
				t.Error("Name is empty")
			}

			if source.GetInformation().Icon == "" {
				t.Error("Icon is empty")
			}

			if source.GetInformation().Version == "" {
				t.Error("Version is empty")
			}

			if len(source.GetInformation().Languages) == 0 {
				t.Error("Languages is empty")
			}

			if source.GetInformation().UpdatedAt.IsZero() {
				t.Error("UpdatedAt is zero")
			}
		})
	}
}
