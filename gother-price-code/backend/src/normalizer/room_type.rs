//! Room Type Normalizer
//!
//! Normalizes room type names across different OTAs.

/// Normalize room type name
pub fn normalize_room_type(room_type: &str) -> String {
    let lower = room_type.to_lowercase();

    // Standard room types
    if lower.contains("deluxe") && lower.contains("suite") {
        return "Deluxe Suite".to_string();
    }
    if lower.contains("junior") && lower.contains("suite") {
        return "Junior Suite".to_string();
    }
    if lower.contains("executive") && lower.contains("suite") {
        return "Executive Suite".to_string();
    }
    if lower.contains("presidential") || lower.contains("royal") {
        return "Presidential Suite".to_string();
    }
    if lower.contains("suite") {
        return "Suite".to_string();
    }

    if lower.contains("deluxe") {
        if lower.contains("king") {
            return "Deluxe King".to_string();
        }
        if lower.contains("twin") {
            return "Deluxe Twin".to_string();
        }
        return "Deluxe Room".to_string();
    }

    if lower.contains("superior") {
        if lower.contains("king") {
            return "Superior King".to_string();
        }
        if lower.contains("twin") {
            return "Superior Twin".to_string();
        }
        return "Superior Room".to_string();
    }

    if lower.contains("premier") || lower.contains("premium") {
        return "Premier Room".to_string();
    }

    if lower.contains("executive") {
        return "Executive Room".to_string();
    }

    if lower.contains("club") {
        return "Club Room".to_string();
    }

    if lower.contains("villa") {
        if lower.contains("pool") {
            return "Pool Villa".to_string();
        }
        return "Villa".to_string();
    }

    if lower.contains("bungalow") {
        return "Bungalow".to_string();
    }

    // Bed types
    if lower.contains("king") {
        return "King Room".to_string();
    }
    if lower.contains("queen") {
        return "Queen Room".to_string();
    }
    if lower.contains("twin") {
        return "Twin Room".to_string();
    }
    if lower.contains("double") {
        return "Double Room".to_string();
    }
    if lower.contains("single") {
        return "Single Room".to_string();
    }

    // Default: clean up and title case
    clean_room_type(room_type)
}

/// Clean up room type string
fn clean_room_type(room_type: &str) -> String {
    let cleaned = room_type
        .trim()
        .replace("  ", " ")
        .replace(" - ", " ");

    // Title case
    cleaned
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_deluxe() {
        assert_eq!(normalize_room_type("Deluxe Room"), "Deluxe Room");
        assert_eq!(normalize_room_type("DELUXE KING BED"), "Deluxe King");
        assert_eq!(normalize_room_type("deluxe twin room"), "Deluxe Twin");
    }

    #[test]
    fn test_normalize_suite() {
        assert_eq!(normalize_room_type("Junior Suite"), "Junior Suite");
        assert_eq!(normalize_room_type("Deluxe Suite Room"), "Deluxe Suite");
        assert_eq!(normalize_room_type("EXECUTIVE SUITE"), "Executive Suite");
    }

    #[test]
    fn test_normalize_standard() {
        assert_eq!(normalize_room_type("Standard King"), "King Room");
        assert_eq!(normalize_room_type("twin bed"), "Twin Room");
    }
}
