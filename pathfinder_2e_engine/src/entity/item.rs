use bevy::prelude::*;
use std::ops::Add;

use crate::mechanics::dice::Die;
use crate::mechanics::modifier::{Modifier, ModifierType};
use crate::mechanics::traits::GameTrait;

// ---------------------------------------------------------------------------
// Item classification
// ---------------------------------------------------------------------------

/// The broad category an item falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Consumable,
    WornItem,
    HeldItem,
}

/// PF2e rarity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Unique,
}

// ---------------------------------------------------------------------------
// Bulk
// ---------------------------------------------------------------------------

/// PF2e bulk: whole-number items have bulk in tenths internally so that
/// light (L) items (= 1 tenth) can be added without floating point.
/// 1 Bulk = 10 units. L = 1 unit. Negligible = 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Bulk(pub u32);

impl Bulk {
    /// Negligible bulk.
    pub fn negligible() -> Self {
        Self(0)
    }

    /// Light bulk (L).
    pub fn light() -> Self {
        Self(1)
    }

    /// Whole-number bulk (e.g., 1 Bulk = 10 internal units).
    pub fn whole(n: u32) -> Self {
        Self(n * 10)
    }

    /// Return the whole-bulk portion (integer division by 10).
    pub fn whole_value(&self) -> u32 {
        self.0 / 10
    }

    /// Return the remaining light items after full bulks are accounted for.
    pub fn light_remainder(&self) -> u32 {
        self.0 % 10
    }
}

impl Add for Bulk {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// ---------------------------------------------------------------------------
// Damage types
// ---------------------------------------------------------------------------

/// PF2e damage types, split into physical and energy categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    // Physical
    Slashing,
    Piercing,
    Bludgeoning,
    // Energy
    Fire,
    Cold,
    Electric,
    Acid,
    Poison,
    Mental,
    Sonic,
}

impl DamageType {
    /// Whether this damage type is physical (slashing, piercing, bludgeoning).
    pub fn is_physical(&self) -> bool {
        matches!(
            self,
            DamageType::Slashing | DamageType::Piercing | DamageType::Bludgeoning
        )
    }

    /// Whether this damage type is energy.
    pub fn is_energy(&self) -> bool {
        !self.is_physical()
    }
}

// ---------------------------------------------------------------------------
// Armor category
// ---------------------------------------------------------------------------

/// PF2e armor categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorCategory {
    Unarmored,
    Light,
    Medium,
    Heavy,
}

// ---------------------------------------------------------------------------
// Consumable type
// ---------------------------------------------------------------------------

/// Sub-categories of consumable items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumableType {
    Potion,
    Scroll,
    Elixir,
    Talisman,
    Oil,
}

// ---------------------------------------------------------------------------
// Core item component
// ---------------------------------------------------------------------------

/// Component: the base data every item has.
/// Items are inert data — the mechanics layer interprets them.
#[derive(Component, Debug, Clone)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub level: i32,
    pub rarity: ItemRarity,
    pub bulk: Bulk,
    pub traits: Vec<GameTrait>,
}

// ---------------------------------------------------------------------------
// Weapon data
// ---------------------------------------------------------------------------

/// Component: weapon-specific data attached alongside `Item`.
#[derive(Component, Debug, Clone)]
pub struct WeaponData {
    pub damage_die: Die,
    pub damage_die_count: u32,
    pub damage_type: DamageType,
    pub weapon_group: String,
    pub range: Option<u32>,
    pub hands_required: u32,
}

// ---------------------------------------------------------------------------
// Armor data
// ---------------------------------------------------------------------------

/// Component: armor-specific data attached alongside `Item`.
#[derive(Component, Debug, Clone)]
pub struct ArmorData {
    pub ac_bonus: i32,
    pub dex_cap: Option<i32>,
    pub check_penalty: i32,
    pub speed_penalty: i32,
    pub strength_threshold: i32,
    pub armor_category: ArmorCategory,
}

// ---------------------------------------------------------------------------
// Shield data
// ---------------------------------------------------------------------------

/// Component: shield-specific data attached alongside `Item`.
#[derive(Component, Debug, Clone)]
pub struct ShieldData {
    pub ac_bonus: i32,
    pub hardness: i32,
    pub hp: i32,
    pub bt: i32,
}

// ---------------------------------------------------------------------------
// Consumable data
// ---------------------------------------------------------------------------

/// Component: consumable-specific data attached alongside `Item`.
#[derive(Component, Debug, Clone)]
pub struct ConsumableData {
    pub consumable_type: ConsumableType,
    pub uses_remaining: u32,
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Component: a container for item entities held by a creature.
/// Each entry is an `Entity` handle referencing a spawned item.
#[derive(Component, Debug, Clone, Default)]
pub struct Inventory {
    pub items: Vec<Entity>,
    pub bulk_limit: Bulk,
}

// ---------------------------------------------------------------------------
// Helper functions — create standard Modifiers from item properties
// ---------------------------------------------------------------------------

/// Create an item-type bonus modifier for a weapon's potency rune.
/// +1/+2/+3 potency runes give an item bonus to attack rolls.
pub fn weapon_modifier_bonus(potency_rune: u32) -> Modifier {
    Modifier::new(
        potency_rune as i32,
        ModifierType::Item,
        format!("+{} weapon potency", potency_rune),
    )
}

/// Create an item-type bonus modifier for armor's AC bonus.
pub fn armor_modifier(ac_bonus: i32) -> Modifier {
    Modifier::new(ac_bonus, ModifierType::Item, "armor")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_damage_types() {
        assert!(DamageType::Slashing.is_physical());
        assert!(DamageType::Piercing.is_physical());
        assert!(DamageType::Bludgeoning.is_physical());
        assert!(!DamageType::Slashing.is_energy());
    }

    #[test]
    fn energy_damage_types() {
        assert!(DamageType::Fire.is_energy());
        assert!(DamageType::Cold.is_energy());
        assert!(DamageType::Electric.is_energy());
        assert!(DamageType::Acid.is_energy());
        assert!(DamageType::Poison.is_energy());
        assert!(DamageType::Mental.is_energy());
        assert!(DamageType::Sonic.is_energy());
        assert!(!DamageType::Fire.is_physical());
    }

    #[test]
    fn bulk_arithmetic() {
        // Two light items
        let two_light = Bulk::light() + Bulk::light();
        assert_eq!(two_light, Bulk(2));

        // Ten light items = 1 Bulk
        let mut total = Bulk::negligible();
        for _ in 0..10 {
            total = total + Bulk::light();
        }
        assert_eq!(total, Bulk::whole(1));
        assert_eq!(total.whole_value(), 1);
        assert_eq!(total.light_remainder(), 0);

        // 1 Bulk + 3 Light
        let combined = Bulk::whole(1) + Bulk(3);
        assert_eq!(combined.whole_value(), 1);
        assert_eq!(combined.light_remainder(), 3);

        // Two whole-number bulks
        let heavy = Bulk::whole(2) + Bulk::whole(3);
        assert_eq!(heavy, Bulk::whole(5));
    }

    #[test]
    fn weapon_potency_modifier() {
        let m1 = weapon_modifier_bonus(1);
        assert_eq!(m1.value, 1);
        assert_eq!(m1.modifier_type, ModifierType::Item);

        let m2 = weapon_modifier_bonus(2);
        assert_eq!(m2.value, 2);
        assert_eq!(m2.modifier_type, ModifierType::Item);

        let m3 = weapon_modifier_bonus(3);
        assert_eq!(m3.value, 3);
        assert_eq!(m3.modifier_type, ModifierType::Item);
        assert_eq!(m3.source, "+3 weapon potency");
    }

    #[test]
    fn armor_ac_modifier() {
        let chain_mail = armor_modifier(6);
        assert_eq!(chain_mail.value, 6);
        assert_eq!(chain_mail.modifier_type, ModifierType::Item);
        assert_eq!(chain_mail.source, "armor");

        let leather = armor_modifier(1);
        assert_eq!(leather.value, 1);
        assert_eq!(leather.modifier_type, ModifierType::Item);
    }
}
