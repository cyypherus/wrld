use crate::{
    entity::effect::{
        ClothingType, ContainerType, Effect, EffectType, FoodType, MaterialType, Object, ToolType,
        WeaponType,
    },
    world::world::ItemStack,
};

use std::fmt::Debug;

/// Color representation using RGBA format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0, 255);
    pub const GREEN: Color = Color::new(0, 255, 0, 255);
    pub const BLUE: Color = Color::new(0, 0, 255, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0, 255);
    pub const BROWN: Color = Color::new(139, 69, 19, 255);
    pub const GRASS_GREEN: Color = Color::new(76, 187, 23, 255);
    pub const WATER_BLUE: Color = Color::new(28, 163, 236, 255);
    pub const DEEP_WATER_BLUE: Color = Color::new(0, 105, 148, 255);
    pub const SAND_COLOR: Color = Color::new(237, 201, 175, 255);
    pub const DIRT_COLOR: Color = Color::new(120, 85, 55, 255);
    pub const ROCK_COLOR: Color = Color::new(128, 132, 135, 255);
    pub const LOG_COLOR: Color = Color::new(207, 164, 105, 255);
    pub const MOUNTAIN_COLOR: Color = Color::new(102, 107, 112, 255);
    pub const SNOW_COLOR: Color = Color::new(230, 230, 250, 255);
    pub const TRAVELER_COLOR: Color = Color::new(255, 69, 0, 255);
    pub const ANIMAL_COLOR: Color = Color::new(255, 215, 0, 255);
    pub const HOUSE_COLOR: Color = Color::new(165, 42, 42, 255);
    pub const SHOP_COLOR: Color = Color::new(210, 105, 30, 255);
    pub const TAVERN_COLOR: Color = Color::new(139, 69, 19, 255);
    pub const TEMPLE_COLOR: Color = Color::new(220, 220, 220, 255);
    pub const ROAD_COLOR: Color = Color::new(150, 150, 150, 255);
    pub const BRIDGE_COLOR: Color = Color::new(133, 94, 66, 255);
}

/// Direction for movement actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
    None,
}

/// Action that an item can perform
#[derive(Debug, Clone)]
pub enum Action {
    Move(Direction),
    Interact((isize, isize, usize), Interaction), // Coordinates to interact with
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    PickUp,
    Drop,
    Consume,
}

/// Container for item data with common properties
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemBox {
    /// The specific item type
    pub item: Item,
    /// Effects applied to this item
    pub effects: Vec<Effect>,
}

impl std::fmt::Display for ItemBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base_name = self.item.to_string();

        if self.effects.is_empty() {
            write!(f, "{}", base_name)
        } else {
            let effects_str = self
                .effects
                .iter()
                .map(|e| e.kind.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            write!(f, "{} [{}]", base_name, effects_str)
        }
    }
}

impl ItemBox {
    /// Create a new ItemBox with the given item and no effects
    pub fn new(item: Item) -> Self {
        Self {
            item,
            effects: Vec::new(),
        }
    }

    /// Create a new ItemBox with the given item and effects
    pub fn with_effects(item: Item, effects: Vec<Effect>) -> Self {
        Self { item, effects }
    }

    pub fn heal(&mut self, health_boost: u32) {
        if let Some(healing_effect) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Healthy))
        {
            healing_effect.intensity += health_boost;
        }
    }
}

/// All possible items in the world
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    // Environment items
    Air,
    Dirt,
    Grass,
    Water,
    DeepWater,
    Sand,
    Log,
    Rock,
    Mountain,
    Snow,

    // Entities
    Traveler { name: String },
    Corpse { name: String, item_type: CorpseType },

    // Town structures
    House { owner: Option<String> },
    Shop { shop_type: String },
    Tavern { name: String },
    Temple { deity: String },

    // Infrastructure
    Road { connected: bool },
    Bridge,
    Object(Object),
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Air => write!(f, "Air"),
            Item::Dirt => write!(f, "Dirt"),
            Item::Grass => write!(f, "Grass"),
            Item::Water => write!(f, "Water"),
            Item::DeepWater => write!(f, "Deep Water"),
            Item::Sand => write!(f, "Sand"),
            Item::Log => write!(f, "Log"),
            Item::Rock => write!(f, "Rock"),
            Item::Mountain => write!(f, "Mountain"),
            Item::Snow => write!(f, "Snow"),
            Item::Traveler { name } => write!(f, "Traveler ({})", name),

            Item::Corpse { name, item_type } => match item_type {
                CorpseType::Traveler => write!(f, "Corpse of {} (Traveler)", name),
                CorpseType::Animal(species) => write!(f, "Corpse of {} ({})", name, species),
            },
            Item::House { owner } => match owner {
                Some(name) => write!(f, "{}'s House", name),
                None => write!(f, "Empty House"),
            },
            Item::Shop { shop_type } => write!(f, "{} Shop", shop_type),
            Item::Tavern { name } => write!(f, "Tavern '{}'", name),
            Item::Temple { deity } => write!(f, "Temple of {}", deity),
            Item::Road { connected } => {
                if *connected {
                    write!(f, "Road (connected)")
                } else {
                    write!(f, "Road (path)")
                }
            }
            Item::Bridge => write!(f, "Bridge"),
            Item::Object(obj) => write!(
                f,
                "{}",
                match obj {
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
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseType {
    Traveler,
    Animal(String), // Species
}

impl ItemBox {
    /// Get the display character for the item
    pub fn get_char(&self) -> char {
        match &self.item {
            Item::Air => ' ',
            Item::Dirt => '.',
            Item::Grass => '"',
            Item::Water => '~',
            Item::DeepWater => '≈',
            Item::Sand => ',',
            Item::Log => 'L',
            Item::Rock => '^',
            Item::Mountain => 'M',
            Item::Snow => '*',
            Item::Traveler { .. } => '@',
            Item::Corpse { .. } => '%',
            Item::House { .. } => 'H',
            Item::Shop { .. } => 'S',
            Item::Tavern { .. } => 'T',
            Item::Temple { .. } => '†',
            Item::Road { .. } => '=',
            Item::Bridge => '#',
            Item::Object(obj) => match obj {
                Object::Food(_) => 'f',
                Object::Tool(_) => 't',
                Object::Material(_) => 'm',

                Object::Weapon(_) => 'w',
                Object::Clothing(_) => 'c',
                Object::Container(_) => 'C',
            },
        }
    }

    /// Get the color for rendering the item
    pub fn get_color(&self) -> Color {
        match &self.item {
            Item::Air => Color::TRANSPARENT,
            Item::Dirt => Color::DIRT_COLOR,
            Item::Grass => Color::GRASS_GREEN,
            Item::Water => Color::WATER_BLUE,
            Item::DeepWater => Color::DEEP_WATER_BLUE,
            Item::Sand => Color::SAND_COLOR,
            Item::Log => Color::LOG_COLOR,
            Item::Rock => Color::ROCK_COLOR,
            Item::Mountain => Color::MOUNTAIN_COLOR,
            Item::Snow => Color::SNOW_COLOR,
            Item::Traveler { .. } => Color::TRAVELER_COLOR,

            Item::Corpse { .. } => Color::new(120, 40, 40, 255), // Dark red
            Item::House { .. } => Color::HOUSE_COLOR,
            Item::Shop { .. } => Color::SHOP_COLOR,
            Item::Tavern { .. } => Color::TAVERN_COLOR,
            Item::Temple { .. } => Color::TEMPLE_COLOR,
            Item::Road { .. } => Color::ROAD_COLOR,
            Item::Bridge => Color::BRIDGE_COLOR,
            Item::Object(obj) => match obj {
                Object::Food(_) => Color::new(210, 105, 30, 255), // Brown
                Object::Tool(_) => Color::new(169, 169, 169, 255), // Silver
                Object::Material(_) => Color::new(139, 69, 19, 255), // Saddle brown
                Object::Weapon(_) => Color::new(192, 192, 192, 255), // Silver
                Object::Clothing(_) => Color::new(240, 248, 255, 255), // Alice blue
                Object::Container(_) => Color::new(160, 82, 45, 255), // Sienna
            },
        }
    }

    /// Get the name of the item
    pub fn get_name(&self) -> String {
        let effects = self.effects.clone();
        match &self.item {
            Item::Air => "Air".to_string(),
            Item::Dirt => "Dirt".to_string(),
            Item::Grass => "Grass".to_string(),
            Item::Water => "Water".to_string(),
            Item::DeepWater => "Deep Water".to_string(),
            Item::Sand => "Sand".to_string(),
            Item::Log => "Log".to_string(),
            Item::Rock => "Rock".to_string(),
            Item::Mountain => "Mountain".to_string(),
            Item::Snow => "Snow".to_string(),
            Item::Traveler { name } => {
                format!("Traveler ({})", name)
            }

            Item::Corpse { name, item_type } => match item_type {
                CorpseType::Traveler => format!("Corpse of {} (Traveler)", name),
                CorpseType::Animal(species) => format!("Corpse of {} ({})", name, species),
            },
            Item::House { owner } => match owner {
                Some(name) => format!("{}'s House", name),
                None => "Empty House".to_string(),
            },
            Item::Shop { shop_type } => format!("{} Shop", shop_type),
            Item::Tavern { name } => format!("Tavern '{}'", name),
            Item::Temple { deity } => format!("Temple of {}", deity),
            Item::Road { connected } => {
                if *connected {
                    "Road (connected)".to_string()
                } else {
                    "Road (path)".to_string()
                }
            }
            Item::Bridge => "Bridge".to_string(),
            Item::Object(obj) => {
                let obj_name = match obj {
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
                .to_string();

                let effect_str = if effects.is_empty() {
                    "".to_string()
                } else {
                    let mut effect_str = " [".to_string();
                    for (i, effect) in effects.iter().enumerate() {
                        if i > 0 {
                            effect_str.push_str(", ");
                        }
                        effect_str.push_str(&effect.kind.to_string());
                    }
                    effect_str.push(']');
                    effect_str
                };

                format!("{}{}", obj_name, effect_str)
            }
        }
    }

    pub fn is_traversable(&self) -> bool {
        match &self.item {
            Item::Air => false,
            Item::Dirt => true,
            Item::Grass => true,
            Item::Water => true,
            Item::DeepWater => false,
            Item::Sand => true,
            Item::Log => true,
            Item::Rock => true,
            Item::Mountain => true,
            Item::Snow => false,
            Item::Traveler { .. } => false,

            Item::Corpse { .. } => true,
            Item::House { .. } => true,
            Item::Shop { .. } => true,
            Item::Tavern { .. } => true,
            Item::Temple { .. } => true,
            Item::Road { .. } => true,
            Item::Bridge => true,
            Item::Object(_) => true, // Objects are generally traversable (can walk over them)
        }
    }

    /// Whether this item can act (NPCs, players, etc)
    pub fn can_act(&self) -> bool {
        match &self.item {
            Item::Traveler { .. } => {
                // Only alive entities can act - check if they have a Dead effect
                !self
                    .effects
                    .iter()
                    .any(|effect| matches!(effect.kind, EffectType::Dead))
            }
            _ => false,
        }
    }

    /// Process interactions between two items
    /// Process interactions between items
    pub fn process_interaction(
        actor: &mut ItemBox,
        target: &mut ItemStack,
        z: usize,
        interaction: Interaction,
    ) -> (bool, String) {
        let item = &target.items()[z].clone();
        // Process the interaction based on type
        match interaction {
            Interaction::Consume => {
                // Handle consuming an item (like eating food)
                // Check if the consumer is attempting to eat a corpse
                if let Item::Corpse { name, item_type } = &item.item {
                    // Consume the corpse for a mix of health, hunger, and thirst benefits
                    let (health_boost, hunger_reduction, thirst_reduction) = match item_type {
                        CorpseType::Traveler => (10, 80, 10), // Consuming a traveler is less healthy but very filling
                        CorpseType::Animal(_) => (30, 60, 20), // Animal corpses provide more nutrition
                    };

                    actor.heal(health_boost);

                    // Get current hunger level
                    let current_hunger = actor
                        .effects
                        .iter()
                        .find(|e| matches!(e.kind, EffectType::Hungry))
                        .map_or(100, |e| e.intensity);

                    // Reduce hunger (bounded at 0)
                    let new_hunger = current_hunger.saturating_sub(hunger_reduction);

                    // Update hunger effect or add it if not present
                    if let Some(idx) = actor
                        .effects
                        .iter()
                        .position(|e| matches!(e.kind, EffectType::Hungry))
                    {
                        actor.effects[idx] = Effect::permanent(EffectType::Hungry, new_hunger);
                    } else {
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Hungry, new_hunger));
                    }

                    // Get current thirst level
                    let current_thirst = actor
                        .effects
                        .iter()
                        .find(|e| matches!(e.kind, EffectType::Thirsty))
                        .map_or(100, |e| e.intensity);

                    // Reduce thirst (bounded at 0)
                    let new_thirst = current_thirst.saturating_sub(thirst_reduction);

                    // Update thirst effect or add it if not present
                    if let Some(idx) = actor
                        .effects
                        .iter()
                        .position(|e| matches!(e.kind, EffectType::Thirsty))
                    {
                        actor.effects[idx] = Effect::permanent(EffectType::Thirsty, new_thirst);
                    } else {
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Thirsty, new_thirst));
                    }

                    println!(
                        "Corpse consumption: Hunger {}->{}%, Thirst {}->{}%",
                        current_hunger, new_hunger, current_thirst, new_thirst
                    );

                    return (
                        true,
                        format!(
                            "Consumed the corpse of {} and gained {}% health",
                            name, health_boost
                        ),
                    );
                } else if let Item::Traveler { name } = &item.item {
                    // We're trying to consume a living traveler! This is only possible if they're dead
                    if item
                        .effects
                        .iter()
                        .any(|effect| matches!(effect.kind, EffectType::Dead))
                    {
                        // They're dead, consume them for nutrition
                        actor.heal(10);

                        // Reduce hunger significantly
                        let current_hunger = actor
                            .effects
                            .iter()
                            .find(|e| matches!(e.kind, EffectType::Hungry))
                            .map_or(100, |e| e.intensity);

                        // Reduce hunger
                        let new_hunger = current_hunger.saturating_sub(80);

                        // Update hunger effect
                        if let Some(idx) = actor
                            .effects
                            .iter()
                            .position(|e| matches!(e.kind, EffectType::Hungry))
                        {
                            actor.effects[idx] = Effect::permanent(EffectType::Hungry, new_hunger);
                        } else {
                            actor
                                .effects
                                .push(Effect::permanent(EffectType::Hungry, new_hunger));
                        }

                        return (true, format!("Consumed {} who was dead", name));
                    } else {
                        return (
                            false,
                            format!("Cannot consume {} - they are still alive!", name),
                        );
                    }
                } else if let Item::Object(Object::Food(food_type)) = &item.item {
                    let (health_boost, hunger_reduction, thirst_reduction) = match food_type {
                        FoodType::Bread => (40, 60, 10),
                        FoodType::Fruit => (30, 40, 20),
                        FoodType::Vegetable => (30, 50, 15),
                    };

                    actor.heal(health_boost);

                    // Get current hunger level
                    let current_hunger = actor
                        .effects
                        .iter()
                        .find(|e| matches!(e.kind, EffectType::Hungry))
                        .map_or(100, |e| e.intensity);

                    // Reduce hunger (bounded at 0)
                    let new_hunger = current_hunger.saturating_sub(hunger_reduction);

                    // Update hunger effect
                    if let Some(idx) = actor
                        .effects
                        .iter()
                        .position(|e| matches!(e.kind, EffectType::Hungry))
                    {
                        actor.effects[idx] = Effect::permanent(EffectType::Hungry, new_hunger);
                    } else {
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Hungry, new_hunger));
                    }

                    // Get current thirst level
                    let current_thirst = actor
                        .effects
                        .iter()
                        .find(|e| matches!(e.kind, EffectType::Thirsty))
                        .map_or(100, |e| e.intensity);

                    // Reduce thirst (bounded at 0)
                    let new_thirst = current_thirst.saturating_sub(thirst_reduction);

                    // Update thirst effect
                    if let Some(idx) = actor
                        .effects
                        .iter()
                        .position(|e| matches!(e.kind, EffectType::Thirsty))
                    {
                        actor.effects[idx] = Effect::permanent(EffectType::Thirsty, new_thirst);
                    } else {
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Thirsty, new_thirst));
                    }

                    println!(
                        "Food consumption: Hunger {}->{}%, Thirst {}->{}%",
                        current_hunger, new_hunger, current_thirst, new_thirst
                    );

                    return (
                        true,
                        format!(
                            "Consumed {} and gained {}% health, reduced hunger by {}%, thirst by {}%",
                            item.get_name(),
                            health_boost,
                            hunger_reduction,
                            thirst_reduction
                        ),
                    );
                }

                // Handle drinking water
                if matches!(item.item, Item::Water | Item::DeepWater) {
                    // Get current thirst level
                    let current_thirst = actor
                        .effects
                        .iter()
                        .find(|e| matches!(e.kind, EffectType::Thirsty))
                        .map_or(100, |e| e.intensity);

                    // Update thirst effect to completely reset thirst
                    if let Some(idx) = actor
                        .effects
                        .iter()
                        .position(|e| matches!(e.kind, EffectType::Thirsty))
                    {
                        actor.effects[idx] = Effect::permanent(EffectType::Thirsty, 0);
                    } else {
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Thirsty, 0));
                    }

                    println!("Water consumption: Thirst {}->0%", current_thirst);

                    return (
                        true,
                        format!("Drank water and quenched thirst ({}->0%)", current_thirst),
                    );
                }

                // Default consumption result
                (false, "Cannot consume this item".to_string())
            }
            Interaction::Drop => {
                let index = actor
                    .effects
                    .iter()
                    .position(|e| matches!(e.kind, EffectType::Holding(_)));
                if let Some(i) = index {
                    if let EffectType::Holding(dropped) = actor.effects.remove(i).kind {
                        let name = dropped.get_name();
                        target.push(dropped);
                        return (true, format!("Dropped {}", name));
                    }
                }
                return (false, "No item to drop".to_string());
            }
            Interaction::PickUp => {
                // Handle picking up an item
                match &item.item {
                    Item::Object(_) => {
                        // Add holding effect to actor
                        actor
                            .effects
                            .push(Effect::permanent(EffectType::Holding(item.clone()), 100));

                        target.pop();
                        (true, format!("Picked up {}", item.get_name()))
                    }
                    Item::Corpse { name, .. } => {
                        // Pick up corpse (we could add a special effect for carrying corpses)
                        (true, format!("Picked up corpse of {}", name))
                    }
                    _ => (false, "No item to pick up".to_string()),
                }
            }
        }
    }
}
