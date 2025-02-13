package utils

import "strings"

func SplitAndTrim(str string, delimiter string) []string {
	var res []string

	for _, s := range strings.Split(str, delimiter) {
		res = append(res, strings.TrimSpace(s))
	}

	return res
}
