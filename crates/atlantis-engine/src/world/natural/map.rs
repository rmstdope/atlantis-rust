use std::collections::{BTreeMap, VecDeque};

use crate::rng::Rng;

use super::super::WorldGenerationError;

const ELEVATION_RANGE: i32 = 16_000;
const MIN_TEMP: i32 = 0;
const MAX_TEMP: i32 = 60;
const MAX_RAINFALL: i32 = 1_000;
const TEMP_ALT_CHANGE: f64 = 6.0 / 1_000.0;
const MOISTURE_FADE: f64 = 4.0;
const MOISTURE_TO_MM: f64 = 20.0;
const RAINFALL_FACTOR: f64 = 0.1;
const WATER_PERCENT: f64 = 0.1;
const MOUNTAIN_PERCENT: f64 = 0.1;
const REDISTRIBUTION: f64 = 1.5;
const EVAPORATION: f64 = 0.75;
const WIND_OFFSETS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Biome {
    Unknown,
    Tundra,
    Mountains,
    Swamp,
    Forest,
    Plains,
    Jungle,
    Desert,
    Water,
}

impl Biome {
    pub(super) fn statistics_index(self) -> Option<usize> {
        match self {
            Self::Unknown => None,
            Self::Tundra => Some(0),
            Self::Mountains => Some(1),
            Self::Swamp => Some(2),
            Self::Forest => Some(3),
            Self::Plains => Some(4),
            Self::Jungle => Some(5),
            Self::Desert => Some(6),
            Self::Water => Some(7),
        }
    }

    pub(super) fn terrain_id(self) -> Option<usize> {
        use crate::defs::terrains::*;

        match self {
            Self::Unknown => None,
            Self::Tundra => Some(R_TUNDRA),
            Self::Mountains => Some(R_MOUNTAIN),
            Self::Swamp => Some(R_SWAMP),
            Self::Forest => Some(R_FOREST),
            Self::Plains => Some(R_PLAIN),
            Self::Jungle => Some(R_JUNGLE),
            Self::Desert => Some(R_DESERT),
            Self::Water => Some(R_OCEAN),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ClimateCell {
    pub(super) elevation: i32,
    pub(super) temperature: i32,
    pub(super) saturation: i32,
    pub(super) evaporation: i32,
    pub(super) rainfall: i32,
    pub(super) biome: Biome,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NaturalMap {
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) cells: Vec<ClimateCell>,
}

impl NaturalMap {
    /// C++: `mapgen.cpp` `Map::Generate`.
    pub(super) fn generate(
        width: i32,
        height: i32,
        seed: i32,
    ) -> Result<Self, WorldGenerationError> {
        if width <= 0 || height <= 0 {
            return Err(WorldGenerationError::NaturalMapDimensionsOverflow { width, height });
        }
        let count = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or(WorldGenerationError::NaturalMapDimensionsOverflow { width, height })?;
        let mut map = Self {
            width,
            height,
            cells: vec![
                ClimateCell {
                    elevation: 0,
                    temperature: 0,
                    saturation: 0,
                    evaporation: 0,
                    rainfall: 0,
                    biome: Biome::Unknown,
                };
                count
            ],
        };
        map.generate_climate(seed);
        Ok(map)
    }

    pub(super) fn get(&self, x: i32, y: i32) -> Option<&ClimateCell> {
        self.index(x, y).map(|index| &self.cells[index])
    }

    fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut ClimateCell> {
        self.index(x, y).map(|index| &mut self.cells[index])
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if y < 0 || y >= self.height {
            return None;
        }
        let x = x.rem_euclid(self.width);
        usize::try_from(x + y * self.width).ok()
    }

    fn generate_climate(&mut self, seed: i32) {
        self.generate_elevation(seed);
        self.set_temperature_and_evaporation();
        self.simulate_rainfall();
        self.assign_biomes();
    }

    fn generate_elevation(&mut self, seed: i32) {
        let mut noise = SimplexNoise::new(5.0, 0.5, seed);
        let mut histogram = BTreeMap::<i32, usize>::new();
        let mut min_elevation = ELEVATION_RANGE;
        let mut max_elevation = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let nx = f64::from(x) / f64::from(self.width);
                let ny = f64::from(y) / f64::from(self.height);
                let normalized = (noise.cylinder_fractal(3, nx, ny) + 1.0) / 2.0;
                let elevation =
                    (normalized.powf(REDISTRIBUTION) * f64::from(ELEVATION_RANGE)).round() as i32;
                let cell = self.get_mut(x, y).expect("map coordinate is in bounds");
                cell.elevation = elevation;
                min_elevation = min_elevation.min(elevation);
                max_elevation = max_elevation.max(elevation);
                *histogram.entry(elevation).or_default() += 1;
            }
        }

        let max_water_cells = (self.cells.len() as f64 * WATER_PERCENT) as usize;
        let mut water_cells = 0;
        let mut sea_level = min_elevation;
        for (&elevation, &count) in &histogram {
            if water_cells >= max_water_cells {
                break;
            }
            sea_level = elevation;
            water_cells += count;
        }

        let max_mountain_cells = (self.cells.len() as f64 * MOUNTAIN_PERCENT) as usize;
        let mut mountain_cells = 0;
        let mut mountain_level = max_elevation;
        for (&elevation, &count) in histogram.iter().rev() {
            if mountain_cells >= max_mountain_cells {
                break;
            }
            mountain_level = elevation;
            mountain_cells += count;
        }

        self.fill_by_elevation(Biome::Water, min_elevation, sea_level);
        self.fill_by_elevation(Biome::Mountains, mountain_level, max_elevation);
        for cell in &mut self.cells {
            cell.elevation -= sea_level;
        }
    }

    fn fill_by_elevation(&mut self, biome: Biome, min: i32, max: i32) {
        for start in 0..self.cells.len() {
            if self.cells[start].biome != Biome::Unknown
                || !(min..=max).contains(&self.cells[start].elevation)
            {
                continue;
            }
            self.fill_blob(start, biome, |cell| (min..=max).contains(&cell.elevation));
        }
    }

    fn set_temperature_and_evaporation(&mut self) {
        let half_height = f64::from(self.height) / 2.0;
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get_mut(x, y).expect("map coordinate is in bounds");
                let is_water = cell.biome == Biome::Water;
                let latitude = ((half_height - f64::from(y)) / half_height) * 90.0;
                let elevation = if is_water { 0 } else { cell.elevation };
                let temperature = f64::from(MAX_TEMP - MIN_TEMP)
                    * (latitude.to_radians() / 1.5).cos()
                    - f64::from(elevation) * TEMP_ALT_CHANGE
                    + f64::from(MIN_TEMP);
                cell.temperature = temperature.round().clamp(-1_000.0, 1_000.0) as i32;
                cell.saturation = saturation(cell.temperature).round() as i32;
                cell.evaporation = if is_water {
                    f64::from(cell.saturation).powf(EVAPORATION).round() as i32
                } else {
                    0
                };
            }
        }
    }

    fn simulate_rainfall(&mut self) {
        for _ in 0..2 {
            for x in 0..self.width {
                for y in 0..self.height {
                    let rainfall = self.rainfall_at(x, y);
                    self.get_mut(x, y)
                        .expect("map coordinate is in bounds")
                        .rainfall = rainfall;
                }
            }
        }
    }

    fn rainfall_at(&self, x: i32, y: i32) -> i32 {
        let mut rainfall = 0.0;
        for dx in -4..=4 {
            for dy in -4..=4 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let Some(source) = self.get(x + dx, y + dy) else {
                    continue;
                };
                let distance = f64::from(((dx * dx + dy * dy) as f64).sqrt() as i32);
                if distance > 4.0 || source.evaporation == 0 {
                    continue;
                }
                let moisture = f64::from(source.evaporation) * (MOISTURE_FADE / distance);
                let target_elevation = self
                    .get(x, y)
                    .expect("target map coordinate is in bounds")
                    .elevation;
                let elevation_change = if target_elevation <= 0 {
                    0
                } else {
                    target_elevation.max(0) - source.elevation.max(0)
                };
                if elevation_change > 0 && moisture > 0.0 {
                    rainfall += (f64::from(elevation_change) * RAINFALL_FACTOR * moisture) / 500.0;
                }
            }
        }
        (rainfall / MOISTURE_TO_MM)
            .round()
            .min(f64::from(MAX_RAINFALL)) as i32
    }

    fn assign_biomes(&mut self) {
        for start in 0..self.cells.len() {
            if self.cells[start].biome != Biome::Unknown {
                continue;
            }
            let biome = BIOMES
                .iter()
                .find(|biome| biome.matches(&self.cells[start]))
                .map(|biome| biome.biome)
                .expect("climate ranges cover every generated cell");
            self.fill_blob(start, biome, |cell| {
                BIOMES
                    .iter()
                    .find(|candidate| candidate.matches(cell))
                    .is_some_and(|candidate| candidate.biome == biome)
            });
        }
    }

    fn fill_blob(&mut self, start: usize, biome: Biome, include: impl Fn(&ClimateCell) -> bool) {
        let start_x = start as i32 % self.width;
        let start_y = start as i32 / self.width;
        let mut pending = VecDeque::from([(start_x, start_y)]);
        self.cells[start].biome = biome;
        while let Some((x, y)) = pending.pop_front() {
            for (dx, dy) in WIND_OFFSETS {
                let Some(index) = self.index(x + dx, y + dy) else {
                    continue;
                };
                if self.cells[index].biome == Biome::Unknown && include(&self.cells[index]) {
                    self.cells[index].biome = biome;
                    pending.push_back((x + dx, y + dy));
                }
            }
        }
    }
}

fn saturation(temperature: i32) -> f64 {
    let absolute_temperature = f64::from(temperature) + 273.0;
    std::f64::consts::E
        .powf(77.345 + 0.0057 * absolute_temperature - 7_235.0 / absolute_temperature)
        / absolute_temperature.powf(8.2)
}

#[derive(Clone, Copy)]
struct BiomeRange {
    biome: Biome,
    temperature: (i32, i32),
    rainfall: (i32, i32),
}

impl BiomeRange {
    fn matches(self, cell: &ClimateCell) -> bool {
        (self.temperature.0..=self.temperature.1).contains(&cell.temperature)
            && (self.rainfall.0..=self.rainfall.1).contains(&cell.rainfall)
    }
}

const BIOMES: &[BiomeRange] = &[
    BiomeRange {
        biome: Biome::Tundra,
        temperature: (-1_000, 0),
        rainfall: (0, 200),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (-1_000, 0),
        rainfall: (201, 300),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (-1_000, 0),
        rainfall: (301, 500),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (-1_000, 0),
        rainfall: (501, 1_000),
    },
    BiomeRange {
        biome: Biome::Tundra,
        temperature: (1, 10),
        rainfall: (0, 100),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (1, 30),
        rainfall: (101, 200),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (1, 10),
        rainfall: (201, 400),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (1, 10),
        rainfall: (401, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (11, 30),
        rainfall: (0, 200),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (11, 20),
        rainfall: (201, 300),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (11, 20),
        rainfall: (301, 500),
    },
    BiomeRange {
        biome: Biome::Swamp,
        temperature: (11, 20),
        rainfall: (501, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (21, 30),
        rainfall: (0, 100),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (21, 30),
        rainfall: (101, 200),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (21, 30),
        rainfall: (201, 400),
    },
    BiomeRange {
        biome: Biome::Jungle,
        temperature: (21, 30),
        rainfall: (401, 500),
    },
    BiomeRange {
        biome: Biome::Swamp,
        temperature: (21, 30),
        rainfall: (501, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (31, 40),
        rainfall: (0, 100),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (31, 40),
        rainfall: (101, 300),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (31, 40),
        rainfall: (301, 400),
    },
    BiomeRange {
        biome: Biome::Jungle,
        temperature: (31, 40),
        rainfall: (401, 600),
    },
    BiomeRange {
        biome: Biome::Swamp,
        temperature: (31, 40),
        rainfall: (601, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (41, 50),
        rainfall: (0, 200),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (41, 50),
        rainfall: (201, 300),
    },
    BiomeRange {
        biome: Biome::Forest,
        temperature: (41, 50),
        rainfall: (301, 400),
    },
    BiomeRange {
        biome: Biome::Jungle,
        temperature: (41, 50),
        rainfall: (401, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (51, 60),
        rainfall: (0, 300),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (51, 60),
        rainfall: (301, 400),
    },
    BiomeRange {
        biome: Biome::Jungle,
        temperature: (51, 60),
        rainfall: (401, 1_000),
    },
    BiomeRange {
        biome: Biome::Desert,
        temperature: (61, 1_000),
        rainfall: (0, 400),
    },
    BiomeRange {
        biome: Biome::Plains,
        temperature: (61, 1_000),
        rainfall: (401, 500),
    },
    BiomeRange {
        biome: Biome::Jungle,
        temperature: (61, 1_000),
        rainfall: (501, 1_000),
    },
];

struct SimplexNoise {
    permutation: [u8; 256],
    frequency: f64,
    amplitude: f64,
}

impl SimplexNoise {
    /// C++: `simplex.cpp` `SimplexNoise::SimplexNoise`; keeps permutation
    /// shuffling in its own seed stream.
    fn new(frequency: f64, amplitude: f64, seed: i32) -> Self {
        let mut permutation = std::array::from_fn(|index| index as u8);
        let mut shuffle_rng = Rng::new(seed);
        for index in (1..permutation.len()).rev() {
            let other = shuffle_rng.get_random((index + 1) as i32) as usize;
            permutation.swap(index, other);
        }
        Self {
            permutation,
            frequency,
            amplitude,
        }
    }

    fn hash(&self, index: i32) -> i32 {
        i32::from(self.permutation[index.rem_euclid(256) as usize])
    }

    fn grad(&self, hash: i32, x: f64, y: f64, z: f64) -> f64 {
        let hash = hash & 15;
        let u = if hash < 8 { x } else { y };
        let v = if hash < 4 {
            y
        } else if hash == 12 || hash == 14 {
            x
        } else {
            z
        };
        (if hash & 1 != 0 { -u } else { u }) + (if hash & 2 != 0 { -v } else { v })
    }

    fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        let skew = (x + y + z) / 3.0;
        let i = (x + skew).floor() as i32;
        let j = (y + skew).floor() as i32;
        let k = (z + skew).floor() as i32;
        let unskew = f64::from(i + j + k) / 6.0;
        let x0 = x - (f64::from(i) - unskew);
        let y0 = y - (f64::from(j) - unskew);
        let z0 = z - (f64::from(k) - unskew);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let x1 = x0 - f64::from(i1) + 1.0 / 6.0;
        let y1 = y0 - f64::from(j1) + 1.0 / 6.0;
        let z1 = z0 - f64::from(k1) + 1.0 / 6.0;
        let x2 = x0 - f64::from(i2) + 2.0 / 6.0;
        let y2 = y0 - f64::from(j2) + 2.0 / 6.0;
        let z2 = z0 - f64::from(k2) + 2.0 / 6.0;
        let x3 = x0 - 1.0 + 3.0 / 6.0;
        let y3 = y0 - 1.0 + 3.0 / 6.0;
        let z3 = z0 - 1.0 + 3.0 / 6.0;

        let gi0 = self.hash(i + self.hash(j + self.hash(k)));
        let gi1 = self.hash(i + i1 + self.hash(j + j1 + self.hash(k + k1)));
        let gi2 = self.hash(i + i2 + self.hash(j + j2 + self.hash(k + k2)));
        let gi3 = self.hash(i + 1 + self.hash(j + 1 + self.hash(k + 1)));
        let contribution = |radius: f64, hash: i32, x: f64, y: f64, z: f64| {
            if radius < 0.0 {
                0.0
            } else {
                let radius_squared = radius * radius;
                radius_squared * radius_squared * self.grad(hash, x, y, z)
            }
        };
        32.0 * (contribution(0.6 - x0 * x0 - y0 * y0 - z0 * z0, gi0, x0, y0, z0)
            + contribution(0.6 - x1 * x1 - y1 * y1 - z1 * z1, gi1, x1, y1, z1)
            + contribution(0.6 - x2 * x2 - y2 * y2 - z2 * z2, gi2, x2, y2, z2)
            + contribution(0.6 - x3 * x3 - y3 * y3 - z3 * z3, gi3, x3, y3, z3))
    }

    fn cylinder_fractal(&mut self, octaves: usize, nx: f64, ny: f64) -> f64 {
        let angle = std::f64::consts::TAU * nx;
        let x = angle.cos() / std::f64::consts::TAU;
        let y = angle.sin() / std::f64::consts::TAU;
        let mut output = 0.0;
        let mut denominator = 0.0;
        let mut frequency = self.frequency;
        let mut amplitude = self.amplitude;
        for _ in 0..octaves {
            output += amplitude * self.noise(x * frequency, y * frequency, ny * frequency);
            denominator += amplitude;
            frequency *= 2.0;
            amplitude *= 0.5;
        }
        output / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::NaturalMap;

    #[test]
    fn fixed_seed_map_assigns_supported_biomes() {
        let first = NaturalMap::generate(16, 16, 5).expect("natural map generation succeeds");
        let repeated = NaturalMap::generate(16, 16, 5).expect("same seed succeeds");

        assert_eq!(first, repeated);
        assert_eq!(first.width, 16);
        assert_eq!(first.height, 16);
        assert_eq!(first.cells.len(), 256);
        assert!(first.cells.iter().all(|cell| {
            cell.biome.terrain_id().is_some()
                && (-1_000..=1_000).contains(&cell.temperature)
                && (0..=1_000).contains(&cell.rainfall)
                && cell.evaporation >= 0
                && cell.saturation >= 0
        }));
        use crate::defs::terrains::{
            R_DESERT, R_FOREST, R_JUNGLE, R_MOUNTAIN, R_OCEAN, R_PLAIN, R_SWAMP, R_TUNDRA,
        };
        assert_eq!(super::Biome::Tundra.terrain_id(), Some(R_TUNDRA));
        assert_eq!(super::Biome::Mountains.terrain_id(), Some(R_MOUNTAIN));
        assert_eq!(super::Biome::Swamp.terrain_id(), Some(R_SWAMP));
        assert_eq!(super::Biome::Forest.terrain_id(), Some(R_FOREST));
        assert_eq!(super::Biome::Plains.terrain_id(), Some(R_PLAIN));
        assert_eq!(super::Biome::Jungle.terrain_id(), Some(R_JUNGLE));
        assert_eq!(super::Biome::Desert.terrain_id(), Some(R_DESERT));
        assert_eq!(super::Biome::Water.terrain_id(), Some(R_OCEAN));
    }
}
