//! Converts ingredient quantities between units. Volume-to-volume and
//! weight-to-weight conversions are exact unit math. Crossing between the
//! two categories (e.g. cups to grams) needs an ingredient's density, which
//! depends on what the ingredient actually is, so we keep a small table of
//! common baking ingredients and refuse to guess for anything else.

#[derive(Debug, Clone, Copy, PartialEq)]
enum Category {
    Volume,
    Weight,
}

fn volume_to_ml(unit: &str) -> Option<f64> {
    match unit {
        "ml" | "milliliter" | "milliliters" => Some(1.0),
        "l" | "liter" | "liters" => Some(1000.0),
        "tsp" | "teaspoon" | "teaspoons" => Some(4.92892),
        "tbsp" | "tablespoon" | "tablespoons" => Some(14.7868),
        "floz" => Some(29.5735),
        "cup" | "cups" => Some(236.588),
        "pint" | "pints" => Some(473.176),
        "quart" | "quarts" => Some(946.353),
        "gallon" | "gallons" => Some(3785.41),
        _ => None,
    }
}

fn weight_to_g(unit: &str) -> Option<f64> {
    match unit {
        "g" | "gram" | "grams" => Some(1.0),
        "kg" | "kilogram" | "kilograms" => Some(1000.0),
        "oz" | "ounce" | "ounces" => Some(28.3495),
        "lb" | "lbs" | "pound" | "pounds" => Some(453.592),
        _ => None,
    }
}

fn category(unit: &str) -> Option<Category> {
    if volume_to_ml(unit).is_some() {
        Some(Category::Volume)
    } else if weight_to_g(unit).is_some() {
        Some(Category::Weight)
    } else {
        None
    }
}

// Grams per milliliter, keyed by a substring to match against the
// ingredient name (checked lowercase). Order matters: more specific
// entries are listed before the generic ones they'd otherwise shadow.
const DENSITY_TABLE: &[(&str, f64)] = &[
    ("brown sugar", 0.90),
    ("powdered sugar", 0.56),
    ("sugar", 0.85),
    ("bread flour", 0.54),
    ("flour", 0.53),
    ("butter", 0.91),
    ("honey", 1.42),
    ("maple syrup", 1.37),
    ("vegetable oil", 0.92),
    ("olive oil", 0.91),
    ("oil", 0.92),
    ("milk", 1.03),
    ("water", 1.00),
    ("cocoa powder", 0.51),
    ("baking powder", 0.90),
    ("baking soda", 0.90),
    ("salt", 1.20),
    ("rice", 0.85),
    ("oats", 0.41),
];

fn density_g_per_ml(ingredient_name: &str) -> Option<f64> {
    let name = ingredient_name.to_lowercase();
    DENSITY_TABLE
        .iter()
        .find(|(needle, _)| name.contains(needle))
        .map(|(_, density)| *density)
}

/// Converts `quantity` of `ingredient_name` from `from_unit` to `to_unit`.
/// Same-category conversions (volume-to-volume, weight-to-weight) always
/// succeed. Crossing categories requires a density lookup by ingredient
/// name and fails if the ingredient isn't in the table.
pub fn convert(quantity: f64, from_unit: &str, to_unit: &str, ingredient_name: &str) -> Result<f64, String> {
    if from_unit.eq_ignore_ascii_case(to_unit) {
        return Ok(quantity);
    }

    let from_category =
        category(from_unit).ok_or_else(|| format!("unknown unit: {from_unit:?}"))?;
    let to_category = category(to_unit).ok_or_else(|| format!("unknown unit: {to_unit:?}"))?;

    match (from_category, to_category) {
        (Category::Volume, Category::Volume) => {
            let ml = quantity * volume_to_ml(from_unit).unwrap();
            Ok(ml / volume_to_ml(to_unit).unwrap())
        }
        (Category::Weight, Category::Weight) => {
            let grams = quantity * weight_to_g(from_unit).unwrap();
            Ok(grams / weight_to_g(to_unit).unwrap())
        }
        (Category::Volume, Category::Weight) => {
            let density = density_g_per_ml(ingredient_name).ok_or_else(|| {
                format!("no known density for {ingredient_name:?}, can't convert {from_unit} to {to_unit}")
            })?;
            let grams = quantity * volume_to_ml(from_unit).unwrap() * density;
            Ok(grams / weight_to_g(to_unit).unwrap())
        }
        (Category::Weight, Category::Volume) => {
            let density = density_g_per_ml(ingredient_name).ok_or_else(|| {
                format!("no known density for {ingredient_name:?}, can't convert {from_unit} to {to_unit}")
            })?;
            let ml = (quantity * weight_to_g(from_unit).unwrap()) / density;
            Ok(ml / volume_to_ml(to_unit).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_volume_to_volume() {
        let ml = convert(1.0, "cup", "ml", "flour").unwrap();
        assert!((ml - 236.588).abs() < 1e-6);

        let cups = convert(473.176, "ml", "cup", "milk").unwrap();
        assert!((cups - 2.0).abs() < 1e-6);
    }

    #[test]
    fn converts_weight_to_weight() {
        let g = convert(1.0, "lb", "g", "sugar").unwrap();
        assert!((g - 453.592).abs() < 1e-6);
    }

    #[test]
    fn converts_volume_to_weight_using_density() {
        // 1 cup of flour is roughly 125g at 0.53 g/ml.
        let grams = convert(1.0, "cup", "g", "flour").unwrap();
        assert!((grams - 125.6).abs() < 1.0);
    }

    #[test]
    fn converts_weight_to_volume_using_density() {
        let cups = convert(200.0, "g", "cup", "sugar").unwrap();
        assert!((cups - 0.994).abs() < 0.05);
    }

    #[test]
    fn same_unit_is_a_no_op() {
        assert_eq!(convert(3.0, "cup", "cup", "flour"), Ok(3.0));
        assert_eq!(convert(3.0, "Cup", "cup", "flour"), Ok(3.0));
    }

    #[test]
    fn unknown_unit_is_an_error() {
        assert!(convert(1.0, "cup", "smidgen", "flour").is_err());
        assert!(convert(1.0, "count", "g", "eggs").is_err());
    }

    #[test]
    fn cross_category_without_known_density_is_an_error() {
        assert!(convert(1.0, "cup", "g", "chopped walnuts").is_err());
    }

    #[test]
    fn density_lookup_prefers_more_specific_match() {
        let brown = convert(1.0, "cup", "g", "packed brown sugar").unwrap();
        let white = convert(1.0, "cup", "g", "granulated sugar").unwrap();
        assert!(brown > white);
    }
}
