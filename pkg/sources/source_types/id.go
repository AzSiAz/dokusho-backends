package source_types

type SourceID string
type SourceSerieID string
type SourceSerieVolumeID string
type SourceSerieVolumeChapterID string
type SourceSerieVolumeChapterImageID string

func NewSourceID(id string) SourceID {
	return SourceID(id)
}

func NewSourceSerieID(id string) SourceSerieID {
	return SourceSerieID(id)
}

func NewSourceSerieVolumeID(id string) SourceSerieVolumeID {
	return SourceSerieVolumeID(id)
}

func NewSourceSerieVolumeChapterID(id string) SourceSerieVolumeChapterID {
	return SourceSerieVolumeChapterID(id)
}

func NewSourceSerieVolumeChapterImageID(id string) SourceSerieVolumeChapterImageID {
	return SourceSerieVolumeChapterImageID(id)
}
