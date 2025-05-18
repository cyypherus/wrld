use std::fmt;
use std::time::Duration;

// Forward declaration to avoid circular references
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    // Basic items
    Food(FoodType),
    Tool(ToolType),
    Material(MaterialType),
    Valuable(ValuableType),
    Weapon(WeaponType),
    Clothing(ClothingType),
    Container(ContainerType),
    Miscellaneous,
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
    Fishing,
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
pub enum ValuableType {
    Coin,
    Gem,
    Jewelry,
    ArtObject,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    /// Type of effect
    pub kind: EffectType,
    /// Severity level (generally 0-100)
    pub intensity: u32,
    /// Duration of the effect, None means permanent until removed
    pub duration: Option<Duration>,
    /// Time remaining for the effect
    pub time_remaining: Option<Duration>,
}

/// All possible effect types in the world
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    // Environmental effects
    Wet,
    Dry,
    Hot,
    Cold,

    // Health effects
    Healthy,
    Sick,
    Injured,
    Poisoned,
    Dead,

    // Status effects
    Hungry,
    Thirsty,
    Tired,
    Energetic,

    // Psychological effects
    Happy,
    Sad,
    Angry,
    Scared,

    // Economic effects
    Wealthy,
    Poor,

    Skilled(Skill),

    // Inventory effects
    Holding(Object),  // Holding an item
    Equipped(Object), // Equipped with an item
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skill {
    Swimming,
}

impl Effect {
    /// Create a new effect with specified parameters
    pub fn new(kind: EffectType, intensity: u32, duration: Option<Duration>) -> Self {
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

    /// Create a temporary effect with specified duration in seconds
    pub fn temporary(kind: EffectType, intensity: u32, seconds: u64) -> Self {
        Self::new(kind, intensity, Some(Duration::from_secs(seconds)))
    }

    /// Create a holding effect (carrying an item)
    pub fn holding(item_type: Object) -> Self {
        Self::permanent(EffectType::Holding(item_type.clone()), 100)
    }

    /// Create an equipped effect (wearing/using an item)
    pub fn equipped(item_type: Object) -> Self {
        Self::permanent(EffectType::Equipped(item_type.clone()), 100)
    }

    /// Update the effect duration, returns true if the effect is still active
    pub fn update(&mut self, delta: Duration) -> bool {
        if let Some(remaining) = &mut self.time_remaining {
            if *remaining <= delta {
                // Effect has expired
                return false;
            }

            // Reduce the remaining time
            *remaining -= delta;
        }

        // Effect is still active
        true
    }

    /// Check if the effect is expired
    pub fn is_expired(&self) -> bool {
        if let Some(remaining) = self.time_remaining {
            remaining.as_secs() == 0
        } else {
            // Permanent effects never expire
            false
        }
    }

    /// Get a description of the effect
    pub fn description(&self) -> String {
        let effect_name = match &self.kind {
            EffectType::Wet => "Wet".to_string(),
            EffectType::Dry => "Dry".to_string(),
            EffectType::Hot => "Hot".to_string(),
            EffectType::Cold => "Cold".to_string(),
            EffectType::Healthy => "Healthy".to_string(),
            EffectType::Sick => "Sick".to_string(),
            EffectType::Injured => "Injured".to_string(),
            EffectType::Poisoned => "Poisoned".to_string(),
            EffectType::Dead => "Dead".to_string(),
            EffectType::Hungry => "Hungry".to_string(),
            EffectType::Thirsty => "Thirsty".to_string(),
            EffectType::Tired => "Tired".to_string(),
            EffectType::Energetic => "Energetic".to_string(),
            EffectType::Happy => "Happy".to_string(),
            EffectType::Sad => "Sad".to_string(),
            EffectType::Angry => "Angry".to_string(),
            EffectType::Scared => "Scared".to_string(),
            EffectType::Wealthy => "Wealthy".to_string(),
            EffectType::Poor => "Poor".to_string(),
            EffectType::Skilled(skill) => match skill {
                Skill::Swimming => "Skilled in Swimming".to_string(),
            },
            EffectType::Holding(item) => format!("Holding {}", item_type_to_string(item)),
            EffectType::Equipped(item) => format!("Equipped with {}", item_type_to_string(item)),
        };

        let intensity_desc = match self.intensity {
            0..=20 => "Slightly",
            21..=40 => "Moderately",
            41..=60 => "Very",
            61..=80 => "Extremely",
            _ => "Overwhelmingly",
        };

        let duration_desc = if let Some(remaining) = self.time_remaining {
            format!(" ({} seconds remaining)", remaining.as_secs())
        } else {
            "".to_string()
        };

        format!("{} {}{}", intensity_desc, effect_name, duration_desc)
    }

    /// Check if this effect counters another effect
    pub fn counters(&self, other: &EffectType) -> bool {
        matches!(
            (&self.kind, other),
            (EffectType::Wet, EffectType::Dry)
                | (EffectType::Dry, EffectType::Wet)
                | (EffectType::Hot, EffectType::Cold)
                | (EffectType::Cold, EffectType::Hot)
                | (EffectType::Healthy, EffectType::Sick)
                | (EffectType::Healthy, EffectType::Injured)
                | (EffectType::Healthy, EffectType::Poisoned)
                | (EffectType::Energetic, EffectType::Tired)
                | (EffectType::Happy, EffectType::Sad)
                | (EffectType::Wealthy, EffectType::Poor)
                | (EffectType::Poor, EffectType::Wealthy)
        )
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
            ToolType::Fishing => "Fishing Rod",
        },
        Object::Material(material_type) => match material_type {
            MaterialType::Wood => "Wood",
            MaterialType::Stone => "Stone",
            MaterialType::Metal => "Metal",
            MaterialType::Cloth => "Cloth",
            MaterialType::Leather => "Leather",
            MaterialType::Gem => "Gem",
        },
        Object::Valuable(valuable_type) => match valuable_type {
            ValuableType::Coin => "Coins",
            ValuableType::Gem => "Gems",
            ValuableType::Jewelry => "Jewelry",
            ValuableType::ArtObject => "Art Object",
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
        Object::Miscellaneous => "Miscellaneous Item",
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
            EffectType::Wet => write!(f, "Wet"),
            EffectType::Dry => write!(f, "Dry"),
            EffectType::Hot => write!(f, "Hot"),
            EffectType::Cold => write!(f, "Cold"),
            EffectType::Healthy => write!(f, "Healthy"),
            EffectType::Sick => write!(f, "Sick"),
            EffectType::Injured => write!(f, "Injured"),
            EffectType::Poisoned => write!(f, "Poisoned"),
            EffectType::Dead => write!(f, "Dead"),
            EffectType::Hungry => write!(f, "Hungry"),
            EffectType::Thirsty => write!(f, "Thirsty"),
            EffectType::Tired => write!(f, "Tired"),
            EffectType::Energetic => write!(f, "Energetic"),
            EffectType::Happy => write!(f, "Happy"),
            EffectType::Sad => write!(f, "Sad"),
            EffectType::Angry => write!(f, "Angry"),
            EffectType::Scared => write!(f, "Scared"),
            EffectType::Wealthy => write!(f, "Wealthy"),
            EffectType::Poor => write!(f, "Poor"),
            EffectType::Skilled(skill) => match skill {
                Skill::Swimming => write!(f, "Skilled in Swimming"),
            },
            EffectType::Holding(item) => write!(f, "Holding {}", item),
            EffectType::Equipped(item) => write!(f, "Equipped with {}", item),
        }
    }
}
