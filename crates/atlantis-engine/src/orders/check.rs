use super::parser::{parse_order_file_with_check_output, CheckOutputEvent};
use super::{OrderParseError, OrderParseMode};
use crate::model::{Faction, Game, Object, Region, Unit};
use crate::ruleset::Ruleset;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderCheckResult {
    pub output: String,
    pub error_count: usize,
}

/// C++: `parseorders.cpp` `DummyGame` and `DoOrdersCheck`.
pub fn check_orders(
    ruleset: &dyn Ruleset,
    input: &[u8],
) -> Result<OrderCheckResult, OrderParseError> {
    let mut game = dummy_game(ruleset)?;
    let tables = ruleset.tables();
    let (result, check_output) = parse_order_file_with_check_output(
        &mut game,
        ruleset.game_defs(),
        &tables,
        input,
        OrderParseMode::Check,
    )?;
    let error_count = result.errors.len();
    let mut output = String::new();
    for event in check_output {
        match event {
            CheckOutputEvent::Error(message) => {
                output.push_str("\n\n*** Error: ");
                output.push_str(&message);
                output.push_str(" ***\n");
            }
            CheckOutputEvent::Order(line) => {
                for _ in 0..line.indent {
                    output.push_str("  ");
                }
                output.push_str(&line.raw);
                output.push('\n');
            }
        }
    }
    output.push('\n');
    if error_count == 0 {
        output.push_str("No errors found.\n");
    } else if error_count == 1 {
        output.push_str("1 error found!\n");
    } else {
        output.push_str(&format!("{error_count} errors found!\n"));
    }

    Ok(OrderCheckResult {
        output,
        error_count,
    })
}

fn dummy_game(ruleset: &dyn Ruleset) -> Result<Game, OrderParseError> {
    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(7, &[]));
    let region_id = game.add_region(Region::default());
    let object_id = game.add_object(region_id, Object::default())?;
    game.add_unit(
        object_id,
        Unit::new(10, Some(faction_id), 1, ruleset.game_defs().max_speed),
    )?;
    game.setup_unit_nums()?;
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::check_orders;
    use crate::ruleset::Ruleset;

    #[test]
    fn check_orders_preserves_reference_error_order_and_turn_form_indentation() {
        let cases = [
            (
                "standard",
                include_bytes!("../../../../fixtures/check/standard/clean.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/standard/clean.check").as_slice(),
                0,
            ),
            (
                "standard",
                include_bytes!("../../../../fixtures/check/standard/one-error.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/standard/one-error.check").as_slice(),
                1,
            ),
            (
                "standard",
                include_bytes!("../../../../fixtures/check/standard/two-errors.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/standard/two-errors.check").as_slice(),
                2,
            ),
            (
                "standard",
                include_bytes!("../../../../fixtures/check/standard/errors.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/standard/errors.check").as_slice(),
                5,
            ),
            (
                "neworigins",
                include_bytes!("../../../../fixtures/check/neworigins/clean.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/neworigins/clean.check").as_slice(),
                0,
            ),
            (
                "neworigins",
                include_bytes!("../../../../fixtures/check/neworigins/one-error.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/neworigins/one-error.check").as_slice(),
                1,
            ),
            (
                "neworigins",
                include_bytes!("../../../../fixtures/check/neworigins/two-errors.orders")
                    .as_slice(),
                include_bytes!("../../../../fixtures/check/neworigins/two-errors.check").as_slice(),
                2,
            ),
            (
                "neworigins",
                include_bytes!("../../../../fixtures/check/neworigins/errors.orders").as_slice(),
                include_bytes!("../../../../fixtures/check/neworigins/errors.check").as_slice(),
                5,
            ),
        ];

        for (ruleset_name, input, expected, expected_errors) in cases {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let result = check_orders(ruleset, input).unwrap();

            assert_eq!(result.error_count, expected_errors, "{ruleset_name}");
            assert_eq!(result.output.as_bytes(), expected, "{ruleset_name}");
        }
    }
}
