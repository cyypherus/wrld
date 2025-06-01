use std::fmt;

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
    Hammer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialType {
    Wood,
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum WeaponType {
    Sword,
    Bow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClothingType {
    Shirt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerType {
    Bag,
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
}

/// All possible effect types in the world
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    // Health effects
    Healthy,

    // Status effects
    Hungry,
    Thirsty,

    // Inventory effects
    Holding(ItemBox), // Holding an item

    PreferredDirection(Direction),

    Thinking(String),

    Tired,
    Young,

    Lonely,

    Wear,

    PathPlanned(Vec<(usize, usize)>),
}

impl Effect {
    /// Create a new effect with specified parameters
    pub fn new(kind: EffectType, intensity: u32, duration: Option<usize>) -> Self {
        Effect {
            kind,
            intensity,
            duration,
        }
    }

    pub fn default_traveler_effects() -> Vec<Effect> {
        vec![
            Effect::permanent(EffectType::Healthy, 100),
            Effect::permanent(EffectType::Hungry, 30),
            Effect::permanent(EffectType::Thirsty, 30),
            Effect::permanent(EffectType::Lonely, 0),
            Effect::temporary(EffectType::Young, 100, 50),
        ]
    }

    pub fn default_terrain_effects() -> Vec<Effect> {
        vec![Effect::permanent(EffectType::Wear, 50)]
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
        if let Some(remaining) = &mut self.duration {
            if *remaining == 0 {
                return false;
            }
            *remaining -= 1;
        }
        true
    }
    pub fn is_expired(&self) -> bool {
        if let Some(remaining) = self.duration {
            remaining == 0
        } else {
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

            ToolType::Hammer => "Hammer",
        },
        Object::Material(material_type) => match material_type {
            MaterialType::Wood => "Wood",
        },
        Object::Weapon(weapon_type) => match weapon_type {
            WeaponType::Sword => "Sword",
            WeaponType::Bow => "Bow",
        },
        Object::Clothing(clothing_type) => match clothing_type {
            ClothingType::Shirt => "Shirt",
        },
        Object::Container(container_type) => match container_type {
            ContainerType::Bag => "Bag",
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
            EffectType::Hungry => write!(f, "Hungry"),
            EffectType::Thirsty => write!(f, "Thirsty"),

            EffectType::Holding(item) => write!(f, "Holding {}", item),
            EffectType::PreferredDirection(_) => {
                write!(f, "Preferred Direction")
            }
            EffectType::Thinking(_) => {
                write!(f, "Thinking")
            }
            EffectType::PathPlanned(_) => {
                write!(f, "Path Planned")
            }
            EffectType::Tired => {
                write!(f, "Tired")
            }
            EffectType::Young => {
                write!(f, "Young")
            }
            EffectType::Lonely => {
                write!(f, "Lonely")
            }
            EffectType::Wear => {
                write!(f, "Wear")
            }
        }
    }
}
