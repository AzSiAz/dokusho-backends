use std::collections::HashSet;

/// Calculates missing integer chapter numbers from a slice of chapter numbers.
/// Supplementary chapters (exactly X.5) are ignored and not treated as representing chapter X.
pub fn calculate_missing_chapters(chapter_numbers: &[f64]) -> Vec<f64> {
    let mut unique = HashSet::new();
    
    for &num in chapter_numbers {
        let whole = num as i32;
        let fraction = num - whole as f64;
        
        // Skip supplementary chapters with fraction exactly 0.5
        if (fraction - 0.5).abs() < 1e-9 {
            continue;
        }
        
        unique.insert(whole);
    }
    
    if unique.is_empty() {
        return Vec::new();
    }
    
    let mut nums: Vec<i32> = unique.into_iter().collect();
    nums.sort_unstable();
    
    let min = nums[0];
    let max = nums[nums.len() - 1];
    
    let mut missing = Vec::new();
    for i in min..max {
        if !nums.contains(&i) {
            missing.push(i as f64);
        }
    }
    
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_missing_chapters() {
        let chapters = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, Vec::<f64>::new());
    }

    #[test]
    fn test_missing_chapters() {
        let chapters = vec![1.0, 3.0, 5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, vec![2.0, 4.0]);
    }

    #[test]
    fn test_supplementary_chapters_ignored() {
        let chapters = vec![1.0, 1.5, 3.0, 3.5, 5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, vec![2.0, 4.0]);
    }

    #[test]
    fn test_decimal_chapters() {
        let chapters = vec![1.0, 1.1, 1.2, 3.0, 3.3, 5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, vec![2.0, 4.0]);
    }

    #[test]
    fn test_empty_input() {
        let chapters = vec![];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, Vec::<f64>::new());
    }

    #[test]
    fn test_single_chapter() {
        let chapters = vec![5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, Vec::<f64>::new());
    }

    #[test]
    fn test_duplicates() {
        let chapters = vec![1.0, 1.0, 3.0, 3.0, 5.0, 5.0];
        let missing = calculate_missing_chapters(&chapters);
        assert_eq!(missing, vec![2.0, 4.0]);
    }
}