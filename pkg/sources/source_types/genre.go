package source_types

import "strings"

type SourceSerieGenre string

const (
	UNKNOWN               SourceSerieGenre = "Unknown"
	OTHER                 SourceSerieGenre = "Other"
	FOUR_KOMA             SourceSerieGenre = "4-Koma"
	ACTION                SourceSerieGenre = "Action"
	ADAPTATION            SourceSerieGenre = "Adaptation"
	ADULT                 SourceSerieGenre = "Adult"
	ADVENTURE             SourceSerieGenre = "Adventure"
	ALIENS                SourceSerieGenre = "Aliens"
	ANIMALS               SourceSerieGenre = "Animals"
	ANTHOLOGY             SourceSerieGenre = "Anthology"
	AWARD_WINNING         SourceSerieGenre = "Award Winning"
	BOYS_LOVE             SourceSerieGenre = "Boy's Love"
	COMEDY                SourceSerieGenre = "Comedy"
	COOKING               SourceSerieGenre = "Cooking"
	CRIME                 SourceSerieGenre = "Crime"
	CROSSDRESSING         SourceSerieGenre = "Crossdressing"
	DELINQUENTS           SourceSerieGenre = "Delinquents"
	DEMONS                SourceSerieGenre = "Demons"
	DOUJINSHI             SourceSerieGenre = "Doujinshi"
	DRAMA                 SourceSerieGenre = "Drama"
	ECCHI                 SourceSerieGenre = "Ecchi"
	FAN_COLORED           SourceSerieGenre = "Fan Colored"
	FANTASY               SourceSerieGenre = "Fantasy"
	FULL_COLOR            SourceSerieGenre = "Full Color"
	GENDER_BENDER         SourceSerieGenre = "Gender Bender"
	GENDERSWAP            SourceSerieGenre = "Genderswap"
	GHOST                 SourceSerieGenre = "Ghost"
	GIRLS_LOVE            SourceSerieGenre = "Girl's Love"
	GORE                  SourceSerieGenre = "Gore"
	GYARU                 SourceSerieGenre = "Gyaru"
	HAREM                 SourceSerieGenre = "Harem"
	HENTAI                SourceSerieGenre = "Hentai"
	HISTORICAL            SourceSerieGenre = "Historical"
	HORROR                SourceSerieGenre = "Horror"
	INCEST                SourceSerieGenre = "Incest"
	ISEKAI                SourceSerieGenre = "Isekai"
	JOSEI                 SourceSerieGenre = "Josei"
	KIDS                  SourceSerieGenre = "Kids"
	LOLICON               SourceSerieGenre = "Lolicon"
	LONG_STRIP            SourceSerieGenre = "Long Strip"
	MAFIA                 SourceSerieGenre = "Mafia"
	MAGIC                 SourceSerieGenre = "Magic"
	MAGICAL_GIRLS         SourceSerieGenre = "Magical Girls"
	MARTIAL_ARTS          SourceSerieGenre = "Martial Arts"
	MATURE                SourceSerieGenre = "Mature"
	MECHA                 SourceSerieGenre = "Mecha"
	MEDICAL               SourceSerieGenre = "Medical"
	MILITARY              SourceSerieGenre = "Military"
	MONSTER_GIRLS         SourceSerieGenre = "Monster Girls"
	MONSTERS              SourceSerieGenre = "Monsters"
	MUSIC                 SourceSerieGenre = "Music"
	MYSTERY               SourceSerieGenre = "Mystery"
	NINJA                 SourceSerieGenre = "Ninja"
	OFFICE_WORKERS        SourceSerieGenre = "Office Workers"
	OFFICIAL_COLORED      SourceSerieGenre = "Official Colored"
	ONE_SHOT              SourceSerieGenre = "One Shot"
	PHILOSOPHICAL         SourceSerieGenre = "Philosophical"
	POLICE                SourceSerieGenre = "Police"
	POST_APOCALYPTIC      SourceSerieGenre = "Post-Apocalyptic"
	PSYCHOLOGICAL         SourceSerieGenre = "Psychological"
	PSYCHOLOGICAL_ROMANCE SourceSerieGenre = "Psychological Romance"
	REINCARNATION         SourceSerieGenre = "Reincarnation"
	REVERSE_HAREM         SourceSerieGenre = "Reverse Harem"
	ROMANCE               SourceSerieGenre = "Romance"
	SAMURAI               SourceSerieGenre = "Samurai"
	SCHOOL_LIFE           SourceSerieGenre = "School Life"
	SCI_FI                SourceSerieGenre = "Sci-Fi"
	SEINEN                SourceSerieGenre = "Seinen"
	SELF_PUBLISHED        SourceSerieGenre = "Self Published"
	SEXUAL_VIOLENCE       SourceSerieGenre = "Sexual Violence"
	SHOTACON              SourceSerieGenre = "Shotacon"
	SHOUJO                SourceSerieGenre = "Shoujo"
	SHOUJO_AI             SourceSerieGenre = "Shoujo Ai"
	SHOUNEN               SourceSerieGenre = "Shounen"
	SHOUNEN_AI            SourceSerieGenre = "Shounen Ai"
	SLICE_OF_LIFE         SourceSerieGenre = "Slice of Life"
	SMUT                  SourceSerieGenre = "Smut"
	SPACE                 SourceSerieGenre = "Space"
	SPORTS                SourceSerieGenre = "Sports"
	SUPERHERO             SourceSerieGenre = "Superhero"
	SUPERNATURAL          SourceSerieGenre = "Supernatural"
	SURVIVAL              SourceSerieGenre = "Survival"
	SUSPENSE              SourceSerieGenre = "Suspense"
	THRILLER              SourceSerieGenre = "Thriller"
	TIME_TRAVEL           SourceSerieGenre = "Time Travel"
	TOOMICS               SourceSerieGenre = "Toomics"
	TRADITIONAL_GAMES     SourceSerieGenre = "Traditional Games"
	TRAGEDY               SourceSerieGenre = "Tragedy"
	VAMPIRES              SourceSerieGenre = "Vampires"
	VIDEO_GAMES           SourceSerieGenre = "Video Games"
	VILLAINESS            SourceSerieGenre = "Villainess"
	VIRTUAL_REALITY       SourceSerieGenre = "Virtual Reality"
	WEB_COMIC             SourceSerieGenre = "Web Comic"
	WUXIA                 SourceSerieGenre = "Wuxia"
	YAOI                  SourceSerieGenre = "Yaoi"
	YURI                  SourceSerieGenre = "Yuri"
	ZOMBIES               SourceSerieGenre = "Zombies"
)

func (s SourceSerieGenre) String() string {
	return string(s)
}

func NewSourceSerieGenre(genre string) SourceSerieGenre {
	genre = strings.Trim(genre, "")

	switch genre {
	case "4-Koma":
		return FOUR_KOMA
	case "Action":
		return ACTION
	case "Adaptation":
		return ADAPTATION
	case "Adult":
		return ADULT
	case "Adventure":
		return ADVENTURE
	case "Aliens":
		return ALIENS
	case "Animals":
		return ANIMALS
	case "Anthology":
		return ANTHOLOGY
	case "Award Winning":
		return AWARD_WINNING
	case "Boy's Love":
		return BOYS_LOVE
	case "Comedy":
		return COMEDY
	case "Cooking":
		return COOKING
	case "Crime":
		return CRIME
	case "Crossdressing":
		return CROSSDRESSING
	case "Delinquents":
		return DELINQUENTS
	case "Demons":
		return DEMONS
	case "Doujinshi":
		return DOUJINSHI
	case "Drama":
		return DRAMA
	case "Ecchi":
		return ECCHI
	case "Fan Colored":
		return FAN_COLORED
	case "Fantasy":
		return FANTASY
	case "Full Color":
		return FULL_COLOR
	case "Gender Bender":
		return GENDER_BENDER
	case "Genderswap":
		return GENDERSWAP
	case "Ghost":
		return GHOST
	case "Girl's Love":
		return GIRLS_LOVE
	case "Gore":
		return GORE
	case "Gyaru":
		return GYARU
	case "Harem":
		return HAREM
	case "Hentai":
		return HENTAI
	case "Historical":
		return HISTORICAL
	case "Horror":
		return HORROR
	case "Incest":
		return INCEST
	case "Isekai":
		return ISEKAI
	case "Josei":
		return JOSEI
	case "Kids":
		return KIDS
	case "Lolicon":
		return LOLICON
	case "Long Strip":
		return LONG_STRIP
	case "Mafia":
		return MAFIA
	case "Magic":
		return MAGIC
	case "Magical Girls":
		return MAGICAL_GIRLS
	case "Martial Arts":
		return MARTIAL_ARTS
	case "Mature":
		return MATURE
	case "Mecha":
		return MECHA
	case "Medical":
		return MEDICAL
	case "Military":
		return MILITARY
	case "Monster Girls":
		return MONSTER_GIRLS
	case "Monsters":
		return MONSTERS
	case "Music":
		return MUSIC
	case "Mystery":
		return MYSTERY
	case "Ninja":
		return NINJA
	case "Office Workers":
		return OFFICE_WORKERS
	case "Official Colored":
		return OFFICIAL_COLORED
	case "One Shot":
		return ONE_SHOT
	case "Philosophical":
		return PHILOSOPHICAL
	case "Police":
		return POLICE
	case "Post-Apocalyptic":
		return POST_APOCALYPTIC
	case "Psychological":
		return PSYCHOLOGICAL
	case "Psychological Romance":
		return PSYCHOLOGICAL_ROMANCE
	case "Reincarnation":
		return REINCARNATION
	case "Reverse Harem":
		return REVERSE_HAREM
	case "Romance":
		return ROMANCE
	case "Samurai":
		return SAMURAI
	case "School Life":
		return SCHOOL_LIFE
	case "Sci-Fi":
		return SCI_FI
	case "Seinen":
		return SEINEN
	case "Self Published":
		return SELF_PUBLISHED
	case "Sexual Violence":
		return SEXUAL_VIOLENCE
	case "Shotacon":
		return SHOTACON
	case "Shoujo":
		return SHOUJO
	case "Shoujo Ai":
		return SHOUJO_AI
	case "Shounen":
		return SHOUNEN
	case "Shounen Ai":
		return SHOUNEN_AI
	case "Slice of Life":
		return SLICE_OF_LIFE
	case "Smut":
		return SMUT
	case "Space":
		return SPACE
	case "Sports":
		return SPORTS
	case "Superhero":
		return SUPERHERO
	case "Supernatural":
		return SUPERNATURAL
	case "Survival":
		return SURVIVAL
	case "Suspense":
		return SUSPENSE
	case "Thriller":
		return THRILLER
	case "Time Travel":
		return TIME_TRAVEL
	case "Toomics":
		return TOOMICS
	case "Traditional Games":
		return TRADITIONAL_GAMES
	case "Tragedy":
		return TRAGEDY
	case "Vampires":
		return VAMPIRES
	case "Video Games":
		return VIDEO_GAMES
	case "Villainess":
		return VILLAINESS
	case "Virtual Reality":
		return VIRTUAL_REALITY
	case "Web Comic":
		return WEB_COMIC
	case "Wuxia":
		return WUXIA
	case "Yaoi":
		return YAOI
	case "Yuri":
		return YURI
	case "Zombies":
		return ZOMBIES
	case "Other":
		return OTHER
	default:
		return UNKNOWN
	}
}

var ALL_GENRES = []SourceSerieGenre{
	OTHER,
	FOUR_KOMA,
	ACTION,
	ADAPTATION,
	ADULT,
	ADVENTURE,
	ALIENS,
	ANIMALS,
	ANTHOLOGY,
	AWARD_WINNING,
	BOYS_LOVE,
	COMEDY,
	COOKING,
	CRIME,
	CROSSDRESSING,
	DELINQUENTS,
	DEMONS,
	DOUJINSHI,
	DRAMA,
	ECCHI,
	FAN_COLORED,
	FANTASY,
	FULL_COLOR,
	GENDER_BENDER,
	GENDERSWAP,
	GHOST,
	GIRLS_LOVE,
	GORE,
	GYARU,
	HAREM,
	HENTAI,
	HISTORICAL,
	HORROR,
	INCEST,
	ISEKAI,
	JOSEI,
	KIDS,
	LOLICON,
	LONG_STRIP,
	MAFIA,
	MAGIC,
	MAGICAL_GIRLS,
	MARTIAL_ARTS,
	MATURE,
	MECHA,
	MEDICAL,
	MILITARY,
	MONSTER_GIRLS,
	MONSTERS,
	MUSIC,
	MYSTERY,
	NINJA,
	OFFICE_WORKERS,
	OFFICIAL_COLORED,
	ONE_SHOT,
	PHILOSOPHICAL,
	POLICE,
	POST_APOCALYPTIC,
	PSYCHOLOGICAL,
	PSYCHOLOGICAL_ROMANCE,
	REINCARNATION,
	REVERSE_HAREM,
	ROMANCE,
	SAMURAI,
	SCHOOL_LIFE,
	SCI_FI,
	SEINEN,
	SELF_PUBLISHED,
	SEXUAL_VIOLENCE,
	SHOTACON,
	SHOUJO,
	SHOUJO_AI,
	SHOUNEN,
	SHOUNEN_AI,
	SLICE_OF_LIFE,
	SMUT,
	SPACE,
	SPORTS,
	SUPERHERO,
	SUPERNATURAL,
	SURVIVAL,
	SUSPENSE,
	THRILLER,
	TIME_TRAVEL,
	TOOMICS,
	TRADITIONAL_GAMES,
	TRAGEDY,
	VAMPIRES,
	VIDEO_GAMES,
	VILLAINESS,
	VIRTUAL_REALITY,
	WEB_COMIC,
	WUXIA,
	YAOI,
	YURI,
	ZOMBIES,
}
