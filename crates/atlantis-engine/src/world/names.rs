use std::collections::BTreeSet;

use crate::{
    defs::terrains::{
        TerrainDef, TerrainId, R_CAVERN, R_FOREST, R_LAKE, R_OCEAN, R_PLAIN, R_UFOREST,
    },
    rng::Rng,
    ruleset::WorldNameStyle,
    text,
};

#[path = "names_data.rs"]
mod names_data;

const NUMBER_OF_TOWN_NAMES: usize = 1_000;
const SYLLABLE_PROBABILITIES: [i32; 4] = [0, 60, 40, 0];

#[derive(Clone, Copy)]
struct WeightedWord {
    word: &'static str,
    probability: i32,
}

#[derive(Clone, Copy)]
struct TerrainSuffix {
    terrain: i32,
    word: &'static str,
    probability: i32,
    town: bool,
    port: bool,
}

enum NameGeneratorState {
    Standard {
        used: Vec<bool>,
    },
    NewOrigins {
        names: Vec<String>,
        name_keys: BTreeSet<Vec<u8>>,
    },
}

pub(super) struct NameGenerator {
    state: NameGeneratorState,
}

impl NameGenerator {
    pub(super) fn new(style: WorldNameStyle) -> Self {
        let state = match style {
            WorldNameStyle::Standard => NameGeneratorState::Standard {
                used: vec![false; names_data::STANDARD_REGION_NAMES.len()],
            },
            WorldNameStyle::NewOrigins => NameGeneratorState::NewOrigins {
                names: Vec::new(),
                name_keys: BTreeSet::new(),
            },
        };
        Self { state }
    }

    pub(super) fn next_region_name(
        &mut self,
        terrain: TerrainId,
        terrain_defs: &[TerrainDef],
        rng: &mut Rng,
    ) -> i32 {
        match &mut self.state {
            NameGeneratorState::Standard { used } => next_standard_name(used, rng),
            NameGeneratorState::NewOrigins { names, name_keys } => {
                next_neworigins_name(names, name_keys, terrain, terrain_defs, false, false, rng)
            }
        }
    }

    pub(super) fn next_town_name(
        &mut self,
        terrain: TerrainId,
        terrain_defs: &[TerrainDef],
        port: bool,
        rng: &mut Rng,
    ) -> String {
        match &mut self.state {
            NameGeneratorState::Standard { used } => {
                let index = next_standard_town_name(used, rng) as usize;
                names_data::STANDARD_REGION_NAMES[index].to_owned()
            }
            NameGeneratorState::NewOrigins { names, name_keys } => {
                next_neworigins_name(names, name_keys, terrain, terrain_defs, true, port, rng);
                names
                    .last()
                    .expect("town name generation appends a name")
                    .clone()
            }
        }
    }

    pub(super) fn resolve(&self, index: i32) -> Option<&str> {
        let index = usize::try_from(index).ok()?;
        match &self.state {
            NameGeneratorState::Standard { .. } => {
                names_data::STANDARD_REGION_NAMES.get(index).copied()
            }
            NameGeneratorState::NewOrigins { names, .. } => index
                .checked_sub(1)
                .and_then(|index| names.get(index).map(String::as_str)),
        }
    }
}

fn next_standard_name(used: &mut [bool], rng: &mut Rng) -> i32 {
    let region_name_count = names_data::STANDARD_REGION_NAMES.len() - NUMBER_OF_TOWN_NAMES;
    next_standard_name_in_range(used, NUMBER_OF_TOWN_NAMES, region_name_count, rng)
}

fn next_standard_town_name(used: &mut [bool], rng: &mut Rng) -> i32 {
    next_standard_name_in_range(used, 0, NUMBER_OF_TOWN_NAMES, rng)
}

fn next_standard_name_in_range(
    used: &mut [bool],
    offset: usize,
    count: usize,
    rng: &mut Rng,
) -> i32 {
    let mut index = rng.get_random(count as i32) as usize;
    for _ in 0..count {
        let absolute_index = index + offset;
        if !used[absolute_index] {
            used[absolute_index] = true;
            return absolute_index as i32;
        }
        index += 1;
        if index >= count {
            index = 0;
        }
    }
    used[offset..offset + count].fill(false);
    let absolute_index = rng.get_random(count as i32) as usize + offset;
    used[absolute_index] = true;
    absolute_index as i32
}

fn next_neworigins_name(
    names: &mut Vec<String>,
    name_keys: &mut BTreeSet<Vec<u8>>,
    terrain: TerrainId,
    terrain_defs: &[TerrainDef],
    town: bool,
    port: bool,
    rng: &mut Rng,
) -> i32 {
    let terrain = terrain_defs
        .get(terrain)
        .expect("generated terrain has a definition")
        .similar_type;
    let terrain = match terrain as usize {
        R_UFOREST => R_FOREST as i32,
        R_LAKE => R_OCEAN as i32,
        R_CAVERN => R_PLAIN as i32,
        _ => terrain,
    };
    let initial_consonant_total = total_probability(names_data::INITIAL_CONSONANTS);
    let vowel_total = total_probability(names_data::VOWELS);
    let final_consonant_total = total_probability(names_data::FINAL_CONSONANTS);

    loop {
        let mut syllables = choose_syllables(rng);
        let mut candidate = String::new();
        let mut trail = false;
        while syllables > 0 {
            syllables -= 1;
            if syllables == 0 {
                let mut roll = rng.get_random(400);
                let mut suffix = None;
                for entry in names_data::TERRAIN_SUFFIXES {
                    if entry.terrain == terrain
                        || entry.terrain == -1
                        || entry.town && town
                        || entry.port && port
                    {
                        if roll >= entry.probability {
                            roll -= entry.probability;
                        } else {
                            suffix = Some(entry.word);
                            break;
                        }
                    }
                }
                if let Some(suffix) = suffix {
                    if trail
                        && matches!(
                            suffix.as_bytes().first(),
                            Some(b'a' | b'e' | b'i' | b'o' | b'u')
                        )
                    {
                        candidate.push('\'');
                    }
                    candidate.push_str(suffix);
                    break;
                }
            }

            if rng.get_random(5) > 0 {
                candidate.push_str(pick_word(
                    names_data::INITIAL_CONSONANTS,
                    initial_consonant_total,
                    rng,
                ));
            } else if trail {
                candidate.push('\'');
            }
            candidate.push_str(pick_word(names_data::VOWELS, vowel_total, rng));
            if rng.get_random(5) > 1 {
                candidate.push_str(pick_word(
                    names_data::FINAL_CONSONANTS,
                    final_consonant_total,
                    rng,
                ));
                trail = false;
            } else {
                trail = true;
            }
        }

        if let Some(first) = candidate.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        if candidate.len() > 12 || !name_keys.insert(name_lookup_key(&candidate)) {
            continue;
        }
        names.push(candidate);
        return names.len() as i32;
    }
}

fn name_lookup_key(name: &str) -> Vec<u8> {
    name.bytes().map(text::fold_case_underscore_byte).collect()
}

fn choose_syllables(rng: &mut Rng) -> usize {
    let mut roll = rng.get_random(SYLLABLE_PROBABILITIES.iter().sum());
    let mut syllables = 0;
    while roll >= SYLLABLE_PROBABILITIES[syllables] {
        roll -= SYLLABLE_PROBABILITIES[syllables];
        syllables += 1;
    }
    syllables + 1
}

fn total_probability(words: &[WeightedWord]) -> i32 {
    words.iter().map(|word| word.probability).sum()
}

fn pick_word<'a>(words: &'a [WeightedWord], total: i32, rng: &mut Rng) -> &'a str {
    let mut roll = rng.get_random(total);
    for entry in words {
        if roll >= entry.probability {
            roll -= entry.probability;
        } else {
            return entry.word;
        }
    }
    unreachable!("weighted-name probabilities cover every random result")
}

#[cfg(test)]
mod tests {
    use super::name_lookup_key;
    use crate::text;

    #[test]
    fn name_lookup_key_matches_atlantis_text_equality() {
        for (left, right) in [
            ("River_Guard", "river guard"),
            ("FOO__BAR", "foo  bar"),
            ("river-guard", "river guard"),
            ("é", "É"),
        ] {
            assert_eq!(
                text::is_equal(left, right),
                name_lookup_key(left) == name_lookup_key(right),
                "{left:?} versus {right:?}"
            );
        }
    }
}
