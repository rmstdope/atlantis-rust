use crate::text::is_equal;

/// Numeric object index matching the C++ `O_*` values in `gamedata.h`.
pub type ObjectId = usize;

pub const NOBJECTS: usize = 76;

pub mod object_flags {
    pub const DISABLED: i32 = 0x001;
    pub const NOMONSTERGROWTH: i32 = 0x002;
    pub const NEVERDECAY: i32 = 0x004;
    pub const CANENTER: i32 = 0x008;
    pub const CANMODIFY: i32 = 0x020;
    pub const TRANSPORT: i32 = 0x040;
    pub const GROUP: i32 = 0x080;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectDef {
    pub name: &'static str,
    pub flags: i32,
    pub protection: i32,
    pub capacity: i32,
    pub sailors: i32,
    pub maximum_mages: i32,
    pub item: i32,
    pub cost: i32,
    pub skill: Option<&'static str>,
    pub level: i32,
    pub maximum_maintenance: i32,
    pub maximum_monthly_decay: i32,
    pub maintenance_factor: i32,
    pub monster: i32,
    pub production_aided: i32,
    pub defense: [i32; 6],
}

pub trait ObjectItemLookup {
    fn name(&self) -> &str;
    fn abbreviation(&self) -> &str;
    fn is_ship(&self) -> bool;
    fn is_disabled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildTarget {
    Object(ObjectId),
    ShipItem(usize),
}

include!("objects_data.rs");

pub fn object_table_jsonl(objects: &[ObjectDef]) -> String {
    let mut output = format!(
        "{{\"table\":\"counts\",\"name\":\"objects\",\"value\":{}}}\n",
        objects.len()
    );
    for (index, object) in objects.iter().enumerate() {
        let skill = object
            .skill
            .map(json_string)
            .unwrap_or_else(|| "null".to_owned());
        let defense = object
            .defense
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!(
            "{{\"table\":\"objects\",\"index\":{index},\"name\":{},\"flags\":{},\"protection\":{},\"capacity\":{},\"sailors\":{},\"maximum_mages\":{},\"item\":{},\"cost\":{},\"skill\":{},\"level\":{},\"maximum_maintenance\":{},\"maximum_monthly_decay\":{},\"maintenance_factor\":{},\"monster\":{},\"production_aided\":{},\"defense\":[{defense}]}}\n",
            json_string(object.name),
            object.flags,
            object.protection,
            object.capacity,
            object.sailors,
            object.maximum_mages,
            object.item,
            object.cost,
            skill,
            object.level,
            object.maximum_maintenance,
            object.maximum_monthly_decay,
            object.maintenance_factor,
            object.monster,
            object.production_aided,
        ));
    }
    output
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

/// C++: `object.cpp` `LookupObject`. It deliberately does not filter disabled entries.
pub fn lookup_object(token: &str, object_defs: &[ObjectDef]) -> Option<ObjectId> {
    object_defs
        .iter()
        .position(|object| is_equal(token, object.name))
}

/// C++: `object.cpp` `ParseObject`. Ship items take precedence when enabled.
pub fn parse_object<T: ObjectItemLookup>(
    token: &str,
    ships: i32,
    object_defs: &[ObjectDef],
    item_defs: &[T],
) -> Option<BuildTarget> {
    if ships > 0 {
        for (index, item) in item_defs.iter().enumerate() {
            if item.is_ship()
                && (is_equal(token, item.name()) || is_equal(token, item.abbreviation()))
                && !item.is_disabled()
            {
                return Some(BuildTarget::ShipItem(index));
            }
        }
    }

    for (index, object) in object_defs.iter().enumerate().skip(1) {
        if is_equal(token, object.name) {
            if object.flags & object_flags::DISABLED != 0 {
                return None;
            }
            return Some(BuildTarget::Object(index));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestItem {
        name: &'static str,
        abbreviation: &'static str,
        ship: bool,
        disabled: bool,
    }

    impl ObjectItemLookup for TestItem {
        fn name(&self) -> &str {
            self.name
        }

        fn abbreviation(&self) -> &str {
            self.abbreviation
        }

        fn is_ship(&self) -> bool {
            self.ship
        }

        fn is_disabled(&self) -> bool {
            self.disabled
        }
    }

    #[test]
    fn object_defaults_match_standard_reference() {
        let actual = object_table_jsonl(&OBJECT_DEFS)
            .lines()
            .skip(1)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let expected = include_str!("../../../../fixtures/tables/standard.jsonl")
            .lines()
            .filter(|line| line.starts_with("{\"table\":\"objects\""))
            .collect::<Vec<_>>();

        assert_eq!(OBJECT_DEFS.len(), 76);
        assert_eq!(actual, expected);
    }

    #[test]
    fn lookup_object_matches_case_and_underscore_insensitive_names() {
        assert_eq!(lookup_object("mAgIcAl_tOwEr", &OBJECT_DEFS), Some(O_MTOWER));
        assert_eq!(lookup_object("None", &OBJECT_DEFS), Some(O_DUMMY));

        let mut disabled = OBJECT_DEFS.to_vec();
        disabled[O_MTOWER].flags |= object_flags::DISABLED;
        assert_eq!(lookup_object("Magical Tower", &disabled), Some(O_MTOWER));
    }

    #[test]
    fn parse_object_preserves_ship_and_disabled_rules() {
        let ship = [TestItem {
            name: "Longboat",
            abbreviation: "LB",
            ship: true,
            disabled: false,
        }];
        assert_eq!(
            parse_object("LB", 1, &OBJECT_DEFS, &ship),
            Some(BuildTarget::ShipItem(0))
        );
        assert_eq!(
            parse_object("Longboat", 1, &OBJECT_DEFS, &ship),
            Some(BuildTarget::ShipItem(0))
        );
        assert_eq!(parse_object("LB", 0, &OBJECT_DEFS, &ship), None);
        assert_eq!(
            parse_object("Tower", 0, &OBJECT_DEFS, &ship),
            Some(BuildTarget::Object(O_TOWER))
        );
        assert_eq!(parse_object("None", 0, &OBJECT_DEFS, &ship), None);

        let disabled_ship = [TestItem {
            disabled: true,
            ..ship[0]
        }];
        assert_eq!(
            parse_object("Longboat", 1, &OBJECT_DEFS, &disabled_ship),
            None
        );

        let mut disabled_objects = OBJECT_DEFS.to_vec();
        disabled_objects[O_TOWER].flags |= object_flags::DISABLED;
        assert_eq!(
            parse_object("Tower", 0, &disabled_objects, &[] as &[TestItem]),
            None
        );
    }
}
