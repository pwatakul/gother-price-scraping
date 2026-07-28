//! Meal Plan Normalizer
//!
//! Normalizes meal plan names across different OTAs.

/// Normalize meal plan name
pub fn normalize_meal_plan(meal_plan: &str) -> String {
    let lower = meal_plan.to_lowercase();

    // All inclusive
    if lower.contains("all inclusive") || lower.contains("all-inclusive") {
        return "All Inclusive".to_string();
    }

    // Full board
    if lower.contains("full board") || lower.contains("fb") && !lower.contains("breakfast") {
        return "Full Board".to_string();
    }

    // Half board
    if lower.contains("half board") || lower.contains("hb") && !lower.contains("breakfast") {
        return "Half Board".to_string();
    }

    // Breakfast included
    if lower.contains("breakfast included")
        || lower.contains("with breakfast")
        || lower.contains("incl. breakfast")
        || lower.contains("bb")
        || lower.contains("bed & breakfast")
        || lower.contains("bed and breakfast")
    {
        return "Breakfast Included".to_string();
    }

    // Room only
    if lower.contains("room only")
        || lower.contains("ro")
        || lower.contains("no meals")
        || lower.contains("accommodation only")
    {
        return "Room Only".to_string();
    }

    // Default
    if lower.contains("breakfast") {
        return "Breakfast Included".to_string();
    }

    "Room Only".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_breakfast() {
        assert_eq!(
            normalize_meal_plan("Breakfast Included"),
            "Breakfast Included"
        );
        assert_eq!(
            normalize_meal_plan("With Breakfast"),
            "Breakfast Included"
        );
        assert_eq!(
            normalize_meal_plan("Bed & Breakfast"),
            "Breakfast Included"
        );
    }

    #[test]
    fn test_normalize_room_only() {
        assert_eq!(normalize_meal_plan("Room Only"), "Room Only");
        assert_eq!(normalize_meal_plan("No Meals"), "Room Only");
    }

    #[test]
    fn test_normalize_boards() {
        assert_eq!(normalize_meal_plan("Half Board"), "Half Board");
        assert_eq!(normalize_meal_plan("Full Board"), "Full Board");
        assert_eq!(normalize_meal_plan("All Inclusive"), "All Inclusive");
    }
}
