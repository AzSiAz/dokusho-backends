package chapterutils

import (
	"math"
	"sort"
)

// CalculateMissingChapters returns missing integer chapter numbers from a slice of chapter numbers.
// Supplementary chapters (exactly X.5) are ignored and not treated as representing chapter X.
func CalculateMissingChapters(chapterNumbers []float64) []float64 {
	unique := map[int]bool{}
	for _, num := range chapterNumbers {
		whole := int(num)
		fraction := num - float64(whole)
		// Skip supplementary chapters with fraction exactly 0.5.
		if math.Abs(fraction-0.5) < 1e-9 {
			continue
		}
		unique[whole] = true
	}
	var nums []int
	for k := range unique {
		nums = append(nums, k)
	}
	sort.Ints(nums)
	if len(nums) == 0 {
		return []float64{}
	}
	missing := []float64{}
	for i := nums[0]; i < nums[len(nums)-1]; i++ {
		if !unique[i] {
			missing = append(missing, float64(i))
		}
	}
	return missing
}
