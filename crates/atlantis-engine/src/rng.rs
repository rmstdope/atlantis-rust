const RANDSIZ: usize = 1 << 8;
const GOLDEN_RATIO: u64 = 0x9e37_79b9;

/// Per-instance LP64 ISAAC state used by the original Atlantis engine.
pub struct Rng {
    randcnt: u64,
    randrsl: [u64; RANDSIZ],
    randmem: [u64; RANDSIZ],
    randa: u64,
    randb: u64,
    randc: u64,
    #[cfg(feature = "rng-trace")]
    trace_sequence: u64,
    #[cfg(feature = "rng-trace")]
    trace_records: Vec<(u64, i32, i32)>,
}

impl Rng {
    /// C++: `gameio.cpp` `seedrandom` initializes the ISAAC context from an `int` seed.
    pub fn new(seed: i32) -> Self {
        let mut rng = Self {
            randcnt: 0,
            randrsl: [0; RANDSIZ],
            randmem: [0; RANDSIZ],
            randa: 0,
            randb: 0,
            randc: 0,
            #[cfg(feature = "rng-trace")]
            trace_sequence: 0,
            #[cfg(feature = "rng-trace")]
            trace_records: Vec::new(),
        };
        rng.seed_random(seed);
        rng
    }

    /// C++: `gameio.cpp` `seedrandom`.
    pub fn seed_random(&mut self, seed: i32) {
        self.randa = 0;
        self.randb = 0;
        self.randc = 0;
        for (index, word) in self.randrsl.iter_mut().enumerate() {
            *word = (seed as u64).wrapping_add(index as u64);
        }
        self.randinit();
    }

    /// C++: `gameio.cpp` `getrandom`.
    pub fn get_random(&mut self, range: i32) -> i32 {
        if range == 0 {
            return 0;
        }

        let negative = range < 0;
        let magnitude = u64::from(range.unsigned_abs());
        let result = (self.next_u64() % magnitude) as i32;
        let result = if negative { -result } else { result };
        #[cfg(feature = "rng-trace")]
        {
            self.trace_records
                .push((self.trace_sequence, range, result));
            self.trace_sequence = self.trace_sequence.wrapping_add(1);
        }
        result
    }

    /// C++: `gameio.cpp` `makeRoll`.
    pub fn make_roll(&mut self, rolls: i32, sides: i32) -> i32 {
        let mut result = 0;
        for _ in 0..rolls {
            result += self.get_random(sides) + 1;
        }
        result
    }

    #[cfg(feature = "rng-trace")]
    pub fn trace(&self) -> String {
        let mut output = String::new();
        for (sequence, range, result) in &self.trace_records {
            output.push_str(&format!("{sequence} {range} {result}\n"));
        }
        output
    }

    fn next_u64(&mut self) -> u64 {
        if self.randcnt == 0 {
            self.isaac();
            self.randcnt = RANDSIZ as u64;
        }
        self.randcnt -= 1;
        self.randrsl[self.randcnt as usize]
    }

    fn randinit(&mut self) {
        let mut state = [GOLDEN_RATIO; 8];
        for _ in 0..4 {
            mix(&mut state);
        }

        for start in (0..RANDSIZ).step_by(8) {
            for (word, seed) in state.iter_mut().zip(&self.randrsl[start..start + 8]) {
                *word = word.wrapping_add(*seed);
            }
            mix(&mut state);
            self.randmem[start..start + 8].copy_from_slice(&state);
        }

        for start in (0..RANDSIZ).step_by(8) {
            for (word, seed) in state.iter_mut().zip(&self.randmem[start..start + 8]) {
                *word = word.wrapping_add(*seed);
            }
            mix(&mut state);
            self.randmem[start..start + 8].copy_from_slice(&state);
        }

        self.isaac();
        self.randcnt = RANDSIZ as u64;
    }

    fn isaac(&mut self) {
        let mut a = self.randa;
        self.randc = self.randc.wrapping_add(1);
        let mut b = self.randb.wrapping_add(self.randc);

        for index in 0..RANDSIZ {
            let x = self.randmem[index];
            let mixed = match index % 4 {
                0 => a << 13,
                1 => a >> 6,
                2 => a << 2,
                _ => a >> 16,
            };
            a = (a ^ mixed).wrapping_add(
                self.randmem[if index < RANDSIZ / 2 {
                    index + RANDSIZ / 2
                } else {
                    index - RANDSIZ / 2
                }],
            );
            let y = ind(&self.randmem, x).wrapping_add(a).wrapping_add(b);
            self.randmem[index] = y;
            b = ind(&self.randmem, y >> 8).wrapping_add(x);
            self.randrsl[index] = b;
        }

        self.randa = a;
        self.randb = b;
    }
}

fn mix(state: &mut [u64; 8]) {
    let [a, b, c, d, e, f, g, h] = state;
    *a ^= *b << 11;
    *d = d.wrapping_add(*a);
    *b = b.wrapping_add(*c);
    *b ^= *c >> 2;
    *e = e.wrapping_add(*b);
    *c = c.wrapping_add(*d);
    *c ^= *d << 8;
    *f = f.wrapping_add(*c);
    *d = d.wrapping_add(*e);
    *d ^= *e >> 16;
    *g = g.wrapping_add(*d);
    *e = e.wrapping_add(*f);
    *e ^= *f << 10;
    *h = h.wrapping_add(*e);
    *f = f.wrapping_add(*g);
    *f ^= *g >> 4;
    *a = a.wrapping_add(*f);
    *g = g.wrapping_add(*h);
    *g ^= *h << 8;
    *b = b.wrapping_add(*g);
    *h = h.wrapping_add(*a);
    *h ^= *a >> 9;
    *c = c.wrapping_add(*h);
    *a = a.wrapping_add(*b);
}

fn ind(randmem: &[u64; RANDSIZ], value: u64) -> u64 {
    let byte_offset = (value as usize) & ((RANDSIZ - 1) << 2);
    let word_index = byte_offset / 8;
    if byte_offset & 4 == 0 {
        randmem[word_index]
    } else {
        (randmem[word_index] >> 32) | (randmem[word_index + 1] << 32)
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    // C++: gameio.cpp getrandom/makeRoll.
    #[test]
    fn range_and_roll_operations_match_cpp_semantics() {
        let mut rng = Rng::new(424242);
        assert_eq!(rng.get_random(10), 4);
        assert_eq!(rng.get_random(-10), -4);
        assert_eq!(rng.get_random(0), 0);
        assert_eq!(rng.get_random(1), 0);
        assert_eq!(rng.get_random(-1), 0);
        assert_eq!(rng.get_random(100), 59);

        rng.seed_random(424242);
        assert_eq!(rng.make_roll(5, 6), 16);
    }

    #[cfg(feature = "rng-trace")]
    #[test]
    fn trace_matches_reference_format() {
        let mut rng = Rng::new(424242);
        assert_eq!(rng.get_random(10), 4);
        assert_eq!(rng.get_random(0), 0);
        assert_eq!(rng.get_random(-10), -4);
        rng.seed_random(424242);
        assert_eq!(rng.get_random(10), 4);

        assert_eq!(rng.trace(), "0 10 4\n1 -10 -4\n2 10 4\n");
    }

    // C++: i_rand.cpp randinit/isaac and gameio.cpp seedrandom.
    #[test]
    fn seeded_stream_matches_cpp_across_refill() {
        let mut rng = Rng::new(424242);
        let mut actual = Vec::new();
        for draw in 0..260 {
            let value = rng.next_u64();
            if matches!(draw, 0..=3 | 253..=259) {
                actual.push((draw, value));
            }
        }

        assert_eq!(
            actual,
            [
                (0, 6_523_884_831_072_444_874),
                (1, 14_885_545_167_966_918_774),
                (2, 3_417_006_282_221_936_643),
                (3, 15_329_384_992_638_300_399),
                (253, 1_890_752_787_698_058_420),
                (254, 14_673_385_515_255_544_974),
                (255, 2_667_497_648_583_980_685),
                (256, 10_492_259_330_534_081_848),
                (257, 17_127_974_547_975_373_111),
                (258, 1_302_271_656_001_229_038),
                (259, 6_499_384_312_691_588_942),
            ]
        );
    }
}
