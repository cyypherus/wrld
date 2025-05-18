use std::fmt;

use fastrand::digit;

use super::item::{Direction, ItemBox};

// Forward declaration to avoid circular references
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    // Basic items
    Food(FoodType),
    Tool(ToolType),
    Material(MaterialType),
    Weapon(WeaponType),
    Clothing(ClothingType),
    Container(ContainerType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoodType {
    Bread,
    Fruit,
    Vegetable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolType {
    Axe,
    Pickaxe,
    Shovel,
    Hammer,
    Saw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialType {
    Wood,
    Stone,
    Metal,
    Cloth,
    Leather,
    Gem,
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum WeaponType {
    Sword,
    Bow,
    Axe,
    Dagger,
    Staff,
    Spear,
    Shield,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClothingType {
    Shirt,
    Pants,
    Boots,
    Gloves,
    Hat,
    Cloak,
    Armor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerType {
    Bag,
    Chest,
    Bottle,
    Pouch,
    Barrel,
    Crate,
}

/// Effect that can be applied to entities and items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    /// Type of effect
    pub kind: EffectType,
    /// Severity level (generally 0-100)
    pub intensity: u32,
    /// Duration of the effect, None means permanent until removed
    pub duration: Option<usize>,
    /// Time remaining for the effect
    pub time_remaining: Option<usize>,
}

/// All possible effect types in the world
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    // Health effects
    Healthy,
    Injured,
    Dead,

    // Status effects
    Hungry,
    Thirsty,

    Skilled(Skill),

    // Inventory effects
    Holding(ItemBox), // Holding an item

    PreferredDirection(Direction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skill {
    Swimming,
}

impl Effect {
    /// Create a new effect with specified parameters
    pub fn new(kind: EffectType, intensity: u32, duration: Option<usize>) -> Self {
        Effect {
            kind,
            intensity,
            duration,
            time_remaining: duration,
        }
    }

    /// Create a permanent effect
    pub fn permanent(kind: EffectType, intensity: u32) -> Self {
        Self::new(kind, intensity, None)
    }

    /// Create a temporary effect with specified duration in ticks
    pub fn temporary(kind: EffectType, intensity: u32, ticks: usize) -> Self {
        Self::new(kind, intensity, Some(ticks))
    }

    /// Create a holding effect (carrying an item)
    pub fn holding(item_type: ItemBox) -> Self {
        Self::permanent(EffectType::Holding(item_type.clone()), 100)
    }

    /// Update the effect duration, returns true if the effect is still active
    pub fn update(&mut self) -> bool {
        if let Some(remaining) = &mut self.time_remaining {
            if *remaining == 0 {
                // Effect has expired
                return false;
            }

            // Reduce the remaining time
            *remaining -= 1;
        }

        // Effect is still active
        true
    }

    /// Check if the effect is expired
    pub fn is_expired(&self) -> bool {
        if let Some(remaining) = self.time_remaining {
            remaining == 0
        } else {
            // Permanent effects never expire
            false
        }
    }
}

// Helper function to convert ItemType to a string representation
fn item_type_to_string(item_type: &Object) -> String {
    match item_type {
        Object::Food(food_type) => match food_type {
            FoodType::Bread => "Bread",
            FoodType::Fruit => "Fruit",
            FoodType::Vegetable => "Vegetable",
        },
        Object::Tool(tool_type) => match tool_type {
            ToolType::Axe => "Axe",
            ToolType::Pickaxe => "Pickaxe",
            ToolType::Shovel => "Shovel",
            ToolType::Hammer => "Hammer",
            ToolType::Saw => "Saw",
        },
        Object::Material(material_type) => match material_type {
            MaterialType::Wood => "Wood",
            MaterialType::Stone => "Stone",
            MaterialType::Metal => "Metal",
            MaterialType::Cloth => "Cloth",
            MaterialType::Leather => "Leather",
            MaterialType::Gem => "Gem",
        },
        Object::Weapon(weapon_type) => match weapon_type {
            WeaponType::Sword => "Sword",
            WeaponType::Bow => "Bow",
            WeaponType::Axe => "Battle Axe",
            WeaponType::Dagger => "Dagger",
            WeaponType::Staff => "Staff",
            WeaponType::Spear => "Spear",
            WeaponType::Shield => "Shield",
        },
        Object::Clothing(clothing_type) => match clothing_type {
            ClothingType::Shirt => "Shirt",
            ClothingType::Pants => "Pants",
            ClothingType::Boots => "Boots",
            ClothingType::Gloves => "Gloves",
            ClothingType::Hat => "Hat",
            ClothingType::Cloak => "Cloak",
            ClothingType::Armor => "Armor",
        },
        Object::Container(container_type) => match container_type {
            ContainerType::Bag => "Bag",
            ContainerType::Chest => "Chest",
            ContainerType::Bottle => "Bottle",
            ContainerType::Pouch => "Pouch",
            ContainerType::Barrel => "Barrel",
            ContainerType::Crate => "Crate",
        },
    }
    .to_string()
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", item_type_to_string(self))
    }
}

impl fmt::Display for EffectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectType::Healthy => write!(f, "Healthy"),
            EffectType::Injured => write!(f, "Injured"),
            EffectType::Dead => write!(f, "Dead"),
            EffectType::Hungry => write!(f, "Hungry"),
            EffectType::Thirsty => write!(f, "Thirsty"),
            EffectType::Skilled(skill) => match skill {
                Skill::Swimming => write!(f, "Skilled in Swimming"),
            },
            EffectType::Holding(item) => write!(f, "Holding {}", item),
            EffectType::PreferredDirection(direction) => {
                write!(f, "Preferred Direction {}", direction)
            }
        }
    }
}
