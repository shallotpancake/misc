/// Trait categories affect how the mechanics layer processes actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraitCategory {
    Attack,
    Move,
    Manipulate,
    Concentrate,
    Sensory,
    Incapacitation,
    Flourish,
    Open,
    Press,
    Tradition,
    DamageType,
    Custom,
}

/// A trait is a named tag with a category. The mechanics layer uses the
/// category to determine processing; the name is for identification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameTrait {
    pub name: String,
    pub category: TraitCategory,
}

impl GameTrait {
    pub fn new(name: impl Into<String>, category: TraitCategory) -> Self {
        Self {
            name: name.into(),
            category,
        }
    }

    pub fn attack() -> Self {
        Self::new("Attack", TraitCategory::Attack)
    }

    pub fn move_trait() -> Self {
        Self::new("Move", TraitCategory::Move)
    }

    pub fn manipulate() -> Self {
        Self::new("Manipulate", TraitCategory::Manipulate)
    }

    pub fn concentrate() -> Self {
        Self::new("Concentrate", TraitCategory::Concentrate)
    }

    pub fn flourish() -> Self {
        Self::new("Flourish", TraitCategory::Flourish)
    }

    pub fn open() -> Self {
        Self::new("Open", TraitCategory::Open)
    }

    pub fn press() -> Self {
        Self::new("Press", TraitCategory::Press)
    }

    pub fn incapacitation() -> Self {
        Self::new("Incapacitation", TraitCategory::Incapacitation)
    }
}
