/// Numeric terrain index matching the C++ `R_*` values in `gamedata.h`.
pub type TerrainId = usize;

pub const R_NUM: usize = 61;

pub mod terrain_flags {
    pub const RIDINGMOUNTS: i32 = 0x1;
    pub const FLYINGMOUNTS: i32 = 0x2;
    pub const BARREN: i32 = 0x4;
    pub const SHOW_RULES: i32 = 0x8;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainProduct {
    pub item: i32,
    pub chance: i32,
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainDef {
    pub name: &'static str,
    pub plural: &'static str,
    pub terrain_type: &'static str,
    pub marker: char,
    pub similar_type: i32,
    pub flags: i32,
    pub population: i32,
    pub wages: i32,
    pub economy: i32,
    pub movement_points: i32,
    pub products: [TerrainProduct; 7],
    pub races: [i32; 4],
    pub coastal_races: [i32; 3],
    pub wandering_monster_frequency: i32,
    pub small_monster: i32,
    pub big_monster: i32,
    pub humanoid: i32,
    pub lair_chance: i32,
    pub lairs: [i32; 6],
}

include!("terrains_data.rs");

pub fn terrain_table_jsonl(terrains: &[TerrainDef]) -> String {
    let mut output = format!(
        "{{\"table\":\"counts\",\"name\":\"terrains\",\"value\":{}}}\n",
        terrains.len()
    );
    for (index, terrain) in terrains.iter().enumerate() {
        let products = terrain
            .products
            .iter()
            .map(|product| {
                format!(
                    "{{\"item\":{},\"chance\":{},\"amount\":{}}}",
                    product.item, product.chance, product.amount
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!(
            "{{\"table\":\"terrains\",\"index\":{index},\"name\":{},\"plural\":{},\"type\":{},\"marker\":{},\"similar_type\":{},\"flags\":{},\"population\":{},\"wages\":{},\"economy\":{},\"movement_points\":{},\"products\":[{products}],\"races\":[{}],\"coastal_races\":[{}],\"wandering_monster_frequency\":{},\"small_monster\":{},\"big_monster\":{},\"humanoid\":{},\"lair_chance\":{},\"lairs\":[{}]}}\n",
            json_string(terrain.name),
            json_string(terrain.plural),
            json_string(terrain.terrain_type),
            json_string(&terrain.marker.to_string()),
            terrain.similar_type,
            terrain.flags,
            terrain.population,
            terrain.wages,
            terrain.economy,
            terrain.movement_points,
            int_array(&terrain.races),
            int_array(&terrain.coastal_races),
            terrain.wandering_monster_frequency,
            terrain.small_monster,
            terrain.big_monster,
            terrain.humanoid,
            terrain.lair_chance,
            int_array(&terrain.lairs),
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

fn int_array(values: &[i32]) -> String {
    values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_defaults_match_standard_reference() {
        let actual = terrain_table_jsonl(&TERRAIN_DEFS)
            .lines()
            .skip(1)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let expected = include_str!("../../../../fixtures/tables/standard.jsonl")
            .lines()
            .filter(|line| line.starts_with("{\"table\":\"terrains\""))
            .collect::<Vec<_>>();

        assert_eq!(TERRAIN_DEFS.len(), 61);
        assert_eq!(actual, expected);
    }
}
