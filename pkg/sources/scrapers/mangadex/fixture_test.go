package mangadex_test

import (
	_ "embed"
)

//go:embed fixtures/chapter_images.json
var chapterImagesJSON string

//go:embed fixtures/serie_detail.json
var serieHTML string

//go:embed fixtures/serie_detail_volume.json
var serieVolume string

//go:embed fixtures/search_serie.json
var searchSerie string
