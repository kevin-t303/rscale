use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub quantity: f64,
    pub unit: String,
    pub name: String,
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Ingredient {
    pub fn scaled(&self, factor: f64) -> Ingredient {
        Ingredient {
            quantity: self.quantity * factor,
            unit: self.unit.clone(),
            name: self.name.clone(),
        }
    }
}

/// Accepts plain decimals ("1.5") and simple fractions ("1/2").
/// Mixed numbers ("1 1/2") are not supported yet since both line
/// formats use whitespace/comma as the field separator.
pub fn parse_quantity(token: &str) -> Option<f64> {
    if let Some((num, denom)) = token.split_once('/') {
        let num: f64 = num.trim().parse().ok()?;
        let denom: f64 = denom.trim().parse().ok()?;
        if denom == 0.0 {
            return None;
        }
        Some(num / denom)
    } else {
        token.trim().parse().ok()
    }
}

/// Rounds to two decimal places and drops trailing zeros, so scaling
/// doesn't turn "2 cups" into "2.0000000000000004 cups".
pub fn format_quantity(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < 1e-9 {
        format!("{}", rounded.round() as i64)
    } else {
        let text = format!("{:.2}", rounded);
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Recipe lines look like: "<quantity> <unit> <name>", e.g. "2 cups flour".
/// Unitless items still need a unit token, conventionally "count".
pub fn parse_recipe_line(line: &str) -> Result<Ingredient, ParseError> {
    let mut parts = line.trim().splitn(3, ' ');
    let quantity = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing quantity: {line:?}")))?;
    let unit = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing unit: {line:?}")))?;
    let name = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing ingredient name: {line:?}")))?;
    let quantity = parse_quantity(quantity)
        .ok_or_else(|| ParseError(format!("bad quantity {quantity:?} in {line:?}")))?;
    Ok(Ingredient {
        quantity,
        unit: unit.to_string(),
        name: name.trim().to_string(),
    })
}

pub fn format_recipe_line(ingredient: &Ingredient) -> String {
    format!(
        "{} {} {}",
        format_quantity(ingredient.quantity),
        ingredient.unit,
        ingredient.name
    )
}

/// CSV lines look like: "<quantity>,<unit>,<name>", e.g. "2,cups,flour".
pub fn parse_csv_line(line: &str) -> Result<Ingredient, ParseError> {
    let mut parts = line.trim().splitn(3, ',');
    let quantity = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing quantity: {line:?}")))?;
    let unit = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing unit: {line:?}")))?;
    let name = parts
        .next()
        .ok_or_else(|| ParseError(format!("missing ingredient name: {line:?}")))?;
    let quantity = parse_quantity(quantity)
        .ok_or_else(|| ParseError(format!("bad quantity {quantity:?} in {line:?}")))?;
    Ok(Ingredient {
        quantity,
        unit: unit.trim().to_string(),
        name: name.trim().to_string(),
    })
}

pub fn format_csv_line(ingredient: &Ingredient) -> String {
    format!(
        "{},{},{}",
        format_quantity(ingredient.quantity),
        ingredient.unit,
        ingredient.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_scales_recipe_line() {
        let ingredient = parse_recipe_line("2 cups flour").unwrap();
        assert_eq!(ingredient.quantity, 2.0);
        assert_eq!(ingredient.unit, "cups");
        assert_eq!(ingredient.name, "flour");

        let doubled = ingredient.scaled(2.0);
        assert_eq!(format_recipe_line(&doubled), "4 cups flour");
    }

    #[test]
    fn parses_fraction_quantity() {
        assert_eq!(parse_quantity("1/2"), Some(0.5));
        assert_eq!(parse_quantity("3/4"), Some(0.75));
        assert_eq!(parse_quantity("x/2"), None);
    }

    #[test]
    fn round_trips_recipe_to_csv() {
        let ingredient = parse_recipe_line("1/2 tsp salt").unwrap();
        assert_eq!(format_csv_line(&ingredient), "0.5,tsp,salt");
    }
}
