use atlantis_engine::orders::{OrderParseResult, ParsedOrderLine};

#[test]
fn order_parse_result_remains_constructible_from_public_fields() {
    let result = OrderParseResult {
        errors: vec!["invalid order".to_owned()],
        lines: vec![ParsedOrderLine {
            raw: "move north".to_owned(),
            indent: 0,
        }],
    };

    assert_eq!(result.errors, ["invalid order"]);
    assert_eq!(result.lines[0].raw, "move north");
}
