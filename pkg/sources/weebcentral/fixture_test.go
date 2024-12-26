package weebcentral_test

import (
	_ "embed"
)

//go:embed fixtures/chapter_images.html
var chapterImagesHTML string

//go:embed fixtures/serie.html
var serieHTML string

//go:embed fixtures/chapters_list.html
var chaptersListHTML string

//go:embed fixtures/search.html
var searchHTML string
