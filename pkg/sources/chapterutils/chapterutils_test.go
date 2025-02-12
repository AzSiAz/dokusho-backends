package chapterutils

import (
	"reflect"
	"testing"
)

func TestCalculateMissingChapters(t *testing.T) {
	tests := []struct {
		name     string
		input    []float64
		expected []float64
	}{
		{
			name:     "empty slice",
			input:    []float64{},
			expected: []float64{},
		},
		{
			name:     "no missing chapters",
			input:    []float64{1, 2, 3, 4},
			expected: []float64{},
		},
		{
			name:  "with missing chapters",
			input: []float64{1, 2, 4, 6},
			// int values: from 1 to 6, missing 3 and 5.
			expected: []float64{3, 5},
		},
		{
			name:  "duplicate and unordered chapters",
			input: []float64{4.5, 2, 2, 2.1, 1, 5},
			// Converted to int: [4,2,2,1,5] sorted: [1,2,4,5], missing: 3 and 4.
			expected: []float64{3, 4},
		},
		{
			name:  "all chapters 1 same integer part",
			input: []float64{1.1, 1.5, 1.9, 3},
			// All converted to int 1 and 3, range between 1 and 3, missing: 2.
			expected: []float64{2},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := CalculateMissingChapters(tc.input)
			t.Log("input:", tc.input)
			t.Log("expected:", tc.expected)
			t.Log("result:", result)
			if !reflect.DeepEqual(result, tc.expected) {
				t.Errorf("CalculateMissingChapters(%v) = %v, expected %v", tc.input, result, tc.expected)
			}
		})
	}
}
