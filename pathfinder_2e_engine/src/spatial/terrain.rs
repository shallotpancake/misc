/// Terrain types — a property of the spatial field, not of entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    Normal,
    Difficult,
    GreaterDifficult,
    Impassable,
    Hazardous,
}

/// Mechanical effects of terrain.
#[derive(Debug, Clone, PartialEq)]
pub enum TerrainEffect {
    MovementMultiplier(u32),
    Blocked,
    DamageOnEntry(String),
}

impl Terrain {
    pub fn effects(&self) -> Vec<TerrainEffect> {
        match self {
            Terrain::Normal => vec![TerrainEffect::MovementMultiplier(1)],
            Terrain::Difficult => vec![TerrainEffect::MovementMultiplier(2)],
            Terrain::GreaterDifficult => vec![TerrainEffect::MovementMultiplier(3)],
            Terrain::Impassable => vec![TerrainEffect::Blocked],
            Terrain::Hazardous => vec![
                TerrainEffect::MovementMultiplier(1),
                TerrainEffect::DamageOnEntry("hazardous terrain".into()),
            ],
        }
    }

    pub fn movement_cost(&self) -> Option<u32> {
        match self {
            Terrain::Normal => Some(1),
            Terrain::Difficult => Some(2),
            Terrain::GreaterDifficult => Some(3),
            Terrain::Impassable => None,
            Terrain::Hazardous => Some(1),
        }
    }
}
