use rand::Rng;

/// A single die type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Die {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

impl Die {
    pub fn max_value(self) -> i32 {
        match self {
            Die::D4 => 4,
            Die::D6 => 6,
            Die::D8 => 8,
            Die::D10 => 10,
            Die::D12 => 12,
            Die::D20 => 20,
        }
    }

    /// Roll this die. RNG is injected — the mechanics layer never owns randomness.
    pub fn roll(self, rng: &mut impl Rng) -> i32 {
        rng.gen_range(1..=self.max_value())
    }
}

/// Result from rolling dice — preserves individual values for inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceRoll {
    pub die: Die,
    pub results: Vec<i32>,
}

impl DiceRoll {
    pub fn total(&self) -> i32 {
        self.results.iter().sum()
    }

    /// The natural value of the first die — relevant for d20 checks
    /// where natural 1 and natural 20 shift degree of success.
    pub fn natural_value(&self) -> Option<i32> {
        self.results.first().copied()
    }
}

/// A pool of dice to roll (e.g., 2d6, 1d20, 4d8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DicePool {
    pub count: u32,
    pub die: Die,
}

impl DicePool {
    pub fn new(count: u32, die: Die) -> Self {
        Self { count, die }
    }

    pub fn roll(&self, rng: &mut impl Rng) -> DiceRoll {
        let results = (0..self.count).map(|_| self.die.roll(rng)).collect();
        DiceRoll {
            die: self.die,
            results,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_max_values() {
        assert_eq!(Die::D4.max_value(), 4);
        assert_eq!(Die::D20.max_value(), 20);
    }

    #[test]
    fn dice_pool_roll_count() {
        let mut rng = rand::thread_rng();
        let pool = DicePool::new(3, Die::D6);
        let roll = pool.roll(&mut rng);
        assert_eq!(roll.results.len(), 3);
        for &r in &roll.results {
            assert!((1..=6).contains(&r));
        }
    }

    #[test]
    fn dice_roll_total() {
        let roll = DiceRoll {
            die: Die::D6,
            results: vec![3, 4, 2],
        };
        assert_eq!(roll.total(), 9);
    }
}
