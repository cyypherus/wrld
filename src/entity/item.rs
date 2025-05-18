use crate::{
    entity::effect::{
        self, ClothingType, ContainerType, Effect, EffectType, FoodType, MaterialType, Object,
        ToolType, WeaponType,
    },
    generate_name,
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

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::East => write!(f, "East"),
            Direction::South => write!(f, "South"),
            Direction::West => write!(f, "West"),
            Direction::NorthEast => write!(f, "Northeast"),
            Direction::SouthEast => write!(f, "Southeast"),
            Direction::SouthWest => write!(f, "Southwest"),
            Direction::NorthWest => write!(f, "Northwest"),
            Direction::None => write!(f, "None"),
        }
    }
}

/// Action that an item can perform
#[derive(Debug, Clone)]
pub enum Action {
    Move(Direction),
    Interact(Option<(isize, isize, usize)>, Option<usize>, Interaction), // Coordinates to interact with
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    PickUp,
    Drop,
    Consume,
    Mate,
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

    pub fn health(&self) -> u32 {
        if let Some(health) = self
            .effects
            .iter()
            .find(|e| matches!(e.kind, EffectType::Healthy))
        {
            return health.intensity;
        }
        0
    }
    pub fn heal(&mut self, health_boost: i32) {
        if let Some(healing_effect) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Healthy))
        {
            if health_boost.is_positive() {
                healing_effect.intensity =
                    healing_effect.intensity.saturating_add(health_boost as u32);
            } else {
                healing_effect.intensity = healing_effect
                    .intensity
                    .saturating_sub(health_boost.unsigned_abs());
            }
        }
    }
    pub fn eat(&mut self, amount: i32) {
        if let Some(hunger) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Hungry))
        {
            if amount.is_positive() {
                hunger.intensity = hunger.intensity.saturating_sub(amount as u32);
            } else {
                hunger.intensity = hunger.intensity.saturating_add(amount.unsigned_abs());
            }
        }
    }
    pub fn hunger(&self) -> u32 {
        if let Some(hunger) = self
            .effects
            .iter()
            .find(|e| matches!(e.kind, EffectType::Hungry))
        {
            return hunger.intensity;
        }
        0
    }
    pub fn thirst(&self) -> u32 {
        if let Some(thirst) = self
            .effects
            .iter()
            .find(|e| matches!(e.kind, EffectType::Thirsty))
        {
            return thirst.intensity;
        }
        0
    }
    pub fn tired(&self) -> bool {
        self.effects
            .iter()
            .any(|e| matches!(e.kind, EffectType::Tired))
    }
    pub fn make_tired(&mut self) {
        self.effects
            .push(Effect::new(EffectType::Tired, 100, Some(100)));
    }
    pub fn drink(&mut self, amount: i32) {
        if let Some(thirst) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Thirsty))
        {
            if amount.is_positive() {
                thirst.intensity = thirst.intensity.saturating_sub(amount as u32);
            } else {
                thirst.intensity = thirst.intensity.saturating_add(amount.unsigned_abs());
            }
        }
    }
    pub fn think(&mut self, thought: String) {
        if let Some(Effect {
            kind: EffectType::Thinking(existing_thought),
            ..
        }) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Thinking(_)))
        {
            *existing_thought = thought;
        } else {
            self.effects
                .push(Effect::new(EffectType::Thinking(thought), 10, Some(10)));
        }
    }
    pub fn quench(&mut self) {
        if let Some(thirst) = self
            .effects
            .iter_mut()
            .find(|e| matches!(e.kind, EffectType::Thirsty))
        {
            thirst.intensity = 0;
        }
    }
    pub fn young(&self) -> bool {
        self.effects
            .iter()
            .any(|e| matches!(e.kind, EffectType::Young))
    }
    /// health, hunger, thirst
    pub fn nutrition(&self) -> (i32, i32, i32) {
        match &self.item {
            Item::Dirt | Item::Grass | Item::Sand | Item::Log => (-1, 1, -1),
            Item::Water | Item::DeepWater => (0, 0, 100),
            Item::Snow => (-5, 0, 20),
            Item::Traveler { .. } => (10, 40, 10),
            Item::Corpse { .. } => (2, 40, 5),
            Item::Object(Object::Food(FoodType::Bread)) => (40, 60, 10),
            Item::Object(Object::Food(FoodType::Fruit)) => (30, 40, 20),
            Item::Object(Object::Food(FoodType::Vegetable)) => (30, 40, 20),
            _ => (0, 0, 0),
        }
    }
    pub fn preferred_direction(&self) -> Option<Direction> {
        if let Some(Effect {
            kind: EffectType::PreferredDirection(d),
            ..
        }) = self
            .effects
            .iter()
            .find(|e| matches!(e.kind, EffectType::PreferredDirection(_)))
        {
            return Some(*d);
        }
        None
    }
    pub fn prefer_direction(&mut self, direction: Option<Direction>) {
        if let Some(direction) = direction {
            self.effects.push(Effect::permanent(
                EffectType::PreferredDirection(direction),
                100,
            ));
        } else {
            self.effects
                .retain(|e| !matches!(e.kind, EffectType::PreferredDirection(_)));
        }
    }
    pub fn primary_need(&self) -> (Priority, Need) {
        let mut _health_level = 100;
        let mut hunger_level = 0;
        let mut thirst_level = 0;

        // Process all effects to determine the entity's state
        for effect in &self.effects {
            match &effect.kind {
                EffectType::Healthy => {
                    _health_level = effect.intensity;
                }
                EffectType::Hungry => {
                    hunger_level = effect.intensity;
                }
                EffectType::Thirsty => {
                    thirst_level = effect.intensity;
                }
                _ => {}
            }
        }

        // Determine the primary need and its priority
        let primary_need: (Priority, Need) = if thirst_level > 80 {
            (Priority::Critical, Need::Water)
        } else if hunger_level > 80 {
            (Priority::Critical, Need::Food)
        } else if thirst_level > 50 {
            (Priority::Urgent, Need::Water)
        } else if hunger_level > 50 {
            (Priority::Urgent, Need::Food)
        } else if hunger_level > 30 {
            (Priority::Normal, Need::Food)
        } else if fastrand::bool() {
            (Priority::Low, Need::Items)
        } else {
            (Priority::Low, Need::Exploration)
        };
        primary_need
    }
}
// Define priority levels for decision making
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    None,
    Liesure,
    Low,      // General collection/exploration
    Normal,   // Thirst/Hunger > 30%
    Urgent,   // Thirst/Hunger > 50%
    Critical, // Health < 20%, Thirst/Hunger > 80%
}
// Define the entity's possible needs
#[derive(Debug)]
pub enum Need {
    Water,
    Food,
    Items,
    Exploration,
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
        target: Option<(&mut ItemStack, usize)>,
        held_target: Option<usize>,
        interaction: Interaction,
    ) -> (bool, String) {
        // Process the interaction based on type
        match interaction {
            Interaction::Consume => {
                let consumed_item: (i32, i32, i32);
                let name: String;
                if let Some((target, z)) = target {
                    let item = target.items()[z].clone();
                    consumed_item = item.nutrition();
                    name = item.get_name();
                    if !matches!(item.item, Item::Water | Item::DeepWater) {
                        target.items.remove(z);
                    }
                } else if let Some(held) = held_target {
                    if let Effect {
                        kind: EffectType::Holding(dropped),
                        intensity: _,
                        duration: _,
                    } = actor.effects.remove(held)
                    {
                        consumed_item = dropped.nutrition();
                        name = dropped.get_name();
                    } else {
                        consumed_item = (0, 0, 0);
                        name = "Nothing".to_string();
                    }
                } else {
                    consumed_item = (0, 0, 0);
                    name = "Nothing".to_string();
                };
                if consumed_item.2 > 0 {
                    actor.heal(consumed_item.0);
                    actor.eat(consumed_item.1);
                    actor.drink(consumed_item.2);

                    return (
                        true,
                        format!(
                            "Consumed {} and gained {}% health, {}% hunger, {}% thirst",
                            name, consumed_item.0, consumed_item.1, consumed_item.2
                        ),
                    );
                }

                // Default consumption result
                (false, "Cannot consume this item".to_string())
            }
            Interaction::Drop => {
                if let Some(held) = held_target {
                    if let EffectType::Holding(dropped) = actor.effects.remove(held).kind {
                        let name = dropped.get_name();
                        if let Some((target, z)) = target {
                            target.push(dropped);
                        }
                        return (true, format!("Dropped {}", name));
                    }
                }
                (false, "No item to drop".to_string())
            }
            Interaction::PickUp => {
                // Handle picking up an item
                if let Some((target, z)) = target {
                    let item = target.items()[z].clone();
                    match &item.item {
                        Item::Object(_) => {
                            // Add holding effect to actor
                            actor
                                .effects
                                .push(Effect::permanent(EffectType::Holding(item.clone()), 100));

                            target.pop();
                            return (true, format!("Picked up {}", item.get_name()));
                        }
                        _ => {
                            return (false, "No item to pick up".to_string());
                        }
                    }
                }
                (false, "No item to pick up".to_string())
            }
            Interaction::Mate => {
                if let Some((target, z)) = target {
                    let item = target.items()[z].clone();

                    match &item.item {
                        Item::Traveler { name: mate_name } => {
                            let Item::Traveler { name: actor_name } = &item.item else {
                                return (false, "No item to mate".to_string());
                            };
                            if actor.tired() || item.tired() {
                                return (false, "Actor is too tired to mate".to_string());
                            }
                            let baby_name = format!("{} {}", generate_name(), mate_name);

                            // Create the baby
                            let baby = Item::Traveler {
                                name: baby_name.clone(),
                            };
                            let baby_effects = vec![
                                Effect::permanent(EffectType::Healthy, 100),
                                Effect::permanent(EffectType::Hungry, 30),
                                Effect::permanent(EffectType::Thirsty, 30),
                                Effect::temporary(EffectType::Young, 100, 50),
                            ];
                            actor.effects.push(Effect::permanent(
                                EffectType::Holding(ItemBox {
                                    item: baby,
                                    effects: baby_effects,
                                }),
                                100,
                            ));
                            actor.make_tired();
                            println!(
                                "A new traveler {} was born to {} and {}",
                                baby_name, actor_name, mate_name
                            );
                            return (
                                true,
                                format!("Successfully created a baby traveler named {}", baby_name),
                            );
                        }
                        _ => {
                            return (false, "Unable to mate with that".to_string());
                        }
                    }
                }
                (false, "Unable to mate with that".to_string())
            }
        }
    }
}
