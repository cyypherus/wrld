use crate::entity::effect::{Effect, EffectType, FoodType, Object};
use crate::entity::item::{
    Action, CorpseType, Direction, Interaction, InteractionResult, Item, ItemBox,
};
use crate::world::town_gen::TownGenerator;
use crate::world::world_gen::{WorldGenParams, WorldGenerator};
use std::fmt;

/// A stack of items in a single grid cell
#[derive(Clone)]
pub struct ItemStack {
    items: Vec<ItemBox>,
}

impl ItemStack {
    pub fn new() -> Self {
        ItemStack {
            items: vec![ItemBox::new(Item::Air)], // Default to having just air
        }
    }

    /// Create a stack with a base item and air on top
    pub fn with_base(base: Item) -> Self {
        ItemStack {
            items: vec![ItemBox::new(base), ItemBox::new(Item::Air)],
        }
    }

    /// Add an item to the top of the stack
    pub fn push(&mut self, item: ItemBox) {
        self.items.push(item);
    }

    /// Remove the top item from the stack
    pub fn pop(&mut self) -> Option<ItemBox> {
        if self.items.len() > 1 {
            // Always keep at least one item (usually air)
            self.items.pop()
        } else {
            None
        }
    }

    /// Get the top item of the stack
    pub fn top(&self) -> Option<&ItemBox> {
        self.items.last()
    }

    /// Get a mutable reference to the top item
    pub fn top_mut(&mut self) -> &mut ItemBox {
        self.items.last_mut().unwrap()
    }

    /// Replace the top item with a new one
    pub fn replace_top(&mut self, item: ItemBox) -> ItemBox {
        let old = self.items.pop().unwrap_or(ItemBox::new(Item::Air));
        self.items.push(item);
        old
    }

    /// Get all items in the stack
    pub fn items(&self) -> &[ItemBox] {
        &self.items
    }

    /// Find the first item that can act in this stack
    pub fn find_actor(&mut self) -> Option<&mut ItemBox> {
        self.items.iter_mut().find(|item_box| item_box.can_act())
    }

    /// Get the most visible non-air item in the stack
    pub fn visible_item(&self) -> Option<(&ItemBox, usize)> {
        // Look through the stack from top to bottom to find a non-air item
        for (index, item_box) in self.items.iter().enumerate().rev() {
            match item_box.item {
                Item::Air => continue,               // Skip air
                _ => return Some((item_box, index)), // Return the first non-air item
            }
        }
        None
    }
}

impl fmt::Debug for ItemStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stack[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", item)?;
        }
        write!(f, "]")
    }
}

/// The world grid containing stacks of items
pub struct World {
    width: usize,
    height: usize,
    grid: Vec<Vec<ItemStack>>,
    seed: u64,
}

impl World {
    /// Create a new empty world
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut grid = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(ItemStack::new());
            }
            grid.push(row);
        }

        World {
            width,
            height,
            grid,
            seed,
        }
    }

    /// Get the width of the world
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the world
    pub fn height(&self) -> usize {
        self.height
    }

    /// Find towns in the world and return information about them
    pub fn get_towns(&self) -> Vec<(String, (usize, usize))> {
        let mut towns = Vec::new();

        // Scan the world for town structures
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(stack) = self.get(x, y) {
                    for item in stack.items() {
                        if let Item::Tavern { name } = &item.item {
                            // Use taverns as town centers for simplicity
                            towns.push((name.clone(), (x, y)));
                            break;
                        }
                    }
                }
            }
        }

        towns
    }

    /// Get a reference to the item stack at (x, y)
    pub fn get(&self, x: usize, y: usize) -> Option<&ItemStack> {
        if x < self.width && y < self.height {
            Some(&self.grid[y][x])
        } else {
            None
        }
    }

    /// Get a mutable reference to the item stack at (x, y)
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut ItemStack> {
        if x < self.width && y < self.height {
            Some(&mut self.grid[y][x])
        } else {
            None
        }
    }

    /// Add an item at position (x, y)
    pub fn add_item(&mut self, x: usize, y: usize, item: Item) -> bool {
        self.add_item_with_effects(x, y, item, Vec::new())
    }

    pub fn add_item_with_effects(
        &mut self,
        x: usize,
        y: usize,
        item: Item,
        effects: Vec<Effect>,
    ) -> bool {
        if let Some(stack) = self.get_mut(x, y) {
            stack.push(ItemBox::with_effects(item, effects));
            true
        } else {
            false
        }
    }

    /// Add an item box directly
    pub fn add_item_box(&mut self, x: usize, y: usize, item_box: ItemBox) -> bool {
        if let Some(stack) = self.get_mut(x, y) {
            stack.push(item_box);
            true
        } else {
            false
        }
    }

    /// Generate a procedural world using advanced terrain generation
    pub fn generate(&mut self) {
        // Configure parameters for the world generator
        let params = WorldGenParams {
            seed: self.seed,
            elevation_scale: 0.06,    // Lower scale for larger features
            moisture_scale: 0.05,     // Lower scale for moisture patterns
            vegetation_scale: 0.12,   // Lower scale for vegetation patterns
            sea_level: 0.42,          // Higher sea level for more islands
            beach_level: 0.46,        // Adjusted beach level
            elevation_amplitude: 1.3, // Higher amplitude for more dramatic terrain
            mountain_frequency: 0.7,  // More mountains
            river_count: 5,           // More rivers
            river_width: 1.2,         // Slightly thinner rivers
        };

        // Create generator with custom parameters
        let mut generator = WorldGenerator::with_params(params);

        // Generate the world grid
        self.grid = generator.generate(self.width, self.height);

        // Generate towns and roads
        self.generate_settlements();

        // Add food items (fruits & vegetables) to the world
        self.add_food_items();
    }

    fn add_food_items(&mut self) {
        let num_food_items = (self.width * self.height) / 200; // Roughly 0.5% of cells get food
        let mut food_placed = 0;
        let mut attempts = 0;

        while food_placed < num_food_items && attempts < num_food_items * 10 {
            let x = fastrand::usize(0..self.width);
            let y = fastrand::usize(0..self.height);

            // Check if this location is suitable for food
            if let Some(stack) = self.get(x, y) {
                let is_grass = stack
                    .items()
                    .iter()
                    .any(|item_box| matches!(item_box.item, Item::Grass));

                let is_water = stack
                    .items()
                    .iter()
                    .any(|item_box| matches!(item_box.item, Item::Water | Item::DeepWater));

                // Only place food on grass, not in water
                if is_grass && !is_water {
                    // Randomly choose between fruit and vegetable
                    let food_type = if fastrand::bool() {
                        Item::Object(Object::Food(FoodType::Fruit))
                    } else {
                        Item::Object(Object::Food(FoodType::Vegetable))
                    };

                    self.add_item(x, y, food_type);
                    food_placed += 1;
                }
            }

            attempts += 1;
        }

        println!("Added {} food items to the world", food_placed);
    }

    /// Generate settlements (towns, roads) directly as items in the world
    fn generate_settlements(&mut self) {
        // Create town generator with the same seed
        let mut town_generator = TownGenerator::new(self.seed);

        // Determine how many towns to generate based on world size
        let min_towns = (self.width * self.height) / 2000;
        let max_towns = (self.width * self.height) / 1000;

        // Ensure we have at least 3 towns
        let min_towns = min_towns.max(3);
        let max_towns = max_towns.max(5);

        // Generate towns directly as items in the grid
        town_generator.generate(
            &mut self.grid,
            self.width,
            self.height,
            min_towns,
            max_towns,
        );
    }

    /// Get immediate surroundings (3x3 grid) around a position
    pub fn get_surroundings(&self, x: usize, y: usize) -> Vec<Vec<Option<(&ItemBox, usize)>>> {
        let mut surroundings = Vec::with_capacity(3);

        for dy in -1..=1 {
            let mut row = Vec::with_capacity(3);
            for dx in -1..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    if let Some((item, i)) = self
                        .get(nx as usize, ny as usize)
                        .and_then(|s| s.visible_item())
                    {
                        row.push(Some((item, i)));
                    } else {
                        row.push(None);
                    }
                } else {
                    row.push(None);
                }
            }
            surroundings.push(row);
        }

        surroundings
    }

    /// Get a sparse selection of extended surroundings (points at various distances)
    pub fn get_extended_view(&self, x: usize, y: usize) -> Vec<Vec<Option<(&ItemBox, usize)>>> {
        let mut extended = Vec::new();
        let ranges = [2, 3, 5, 8]; // Distances to sample

        for &range in &ranges {
            let mut row = Vec::new();

            // Check the four cardinal directions at this distance
            for &(dx, dy) in &[(range, 0), (0, range), (-range, 0), (0, -range)] {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    if let Some((item, i)) = self
                        .get(nx as usize, ny as usize)
                        .and_then(|s| s.visible_item())
                    {
                        row.push(Some((item, i)));
                    } else {
                        row.push(None);
                    }
                } else {
                    row.push(None);
                }
            }

            extended.push(row);
        }

        extended
    }

    /// Process one tick of the world, allowing all actors to take their actions
    pub fn tick(&mut self) {
        // Apply effects over time to all items
        self.apply_effects();

        // Collect all positions with actors
        let mut actor_positions = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(actor) = self.grid[y][x].find_actor() {
                    if actor.can_act() {
                        actor_positions.push((x, y));
                    }
                }
            }
        }

        // Process actions for each actor
        for (x, y) in actor_positions {
            // First, gather information about surroundings
            let stack = self.get_mut(x, y).cloned();
            let surroundings = self.get_surroundings(x, y);
            let extended = self.get_extended_view(x, y);

            // Then get a mutable reference and determine the action
            let width = self.width();
            let height = self.height();
            if let Some(mut stack) = stack {
                if let Some(actor) = stack.find_actor() {
                    // Clone the actor to avoid borrow issues
                    let actor_clone = actor.clone();
                    let effects = actor_clone.effects.clone();

                    // Print current status if hunger or thirst are high
                    if let Item::Traveler { name } | Item::Animal { species: name } =
                        &actor_clone.item
                    {
                        for effect in &effects {
                            match &effect.kind {
                                EffectType::Hungry if effect.intensity > 50 => {
                                    println!("{} is hungry ({}%)", name, effect.intensity);
                                }
                                EffectType::Thirsty if effect.intensity > 50 => {
                                    println!("{} is thirsty ({}%)", name, effect.intensity);
                                }
                                _ => {}
                            }
                        }
                    }

                    // Get action based on surroundings
                    let action = match &actor_clone.item {
                        Item::Traveler { .. } | Item::Animal { .. } => {
                            self.decide_action(&actor_clone, &effects, &surroundings, &extended)
                        }
                        _ => Action::Wait,
                    };

                    dbg!(&action);

                    // Execute the action
                    match action {
                        Action::Move(direction) => {
                            self.move_item(x, y, direction);
                        }
                        Action::Interact((tx, ty, tz), interaction) => {
                            // Get the target coordinates relative to the actor's position
                            let tx = (x as isize + tx) as usize;
                            let ty = (y as isize + ty) as usize;
                            if tx < width && ty < height {
                                // Get the target item to interact with
                                if let Some(target_stack) = self.get(tx, ty) {
                                    dbg!(tx, ty, tz, target_stack);
                                    if let Some(target_item) = target_stack.items().get(tz) {
                                        // Process the interaction
                                        // Create a mutable copy of the actor to work with
                                        let mut actor_mut = actor_clone.clone();

                                        // Process the interaction with direct modification
                                        let (success, message) = ItemBox::process_interaction(
                                            &mut actor_mut,
                                            target_item,
                                            interaction,
                                        );

                                        // Update the actor with the modified version
                                        if success {
                                            println!("Interaction succeeded: {}", message);

                                            // Replace the actor with the modified version that has updated effects
                                            if let Some(actor) =
                                                self.get_mut(tx, ty).and_then(|s| s.find_actor())
                                            {
                                                *actor = actor_mut;
                                            }

                                            // Remove consumed items if needed
                                            if interaction == Interaction::Consume
                                                || interaction == Interaction::PickUp
                                            {
                                                // Remove the target item from its stack
                                                if let Some(target_stack) = self.get_mut(tx, ty) {
                                                    // Remove the first visible non-air item
                                                    if let Some((_, item_idx)) =
                                                        target_stack.visible_item()
                                                    {
                                                        target_stack.items.remove(item_idx);
                                                    }
                                                }
                                            }
                                        } else {
                                            println!("Interaction failed: {}", message);
                                        }
                                    }
                                }
                            }
                        }
                        Action::Wait => {
                            // Do nothing
                        }
                    }
                }
            }
        }
    }

    /// Decide what action an item should take
    fn decide_action(
        &self,
        item_box: &ItemBox,
        effects: &[Effect],
        surroundings: &[Vec<Option<(&ItemBox, usize)>>],
        extended: &[Vec<Option<(&ItemBox, usize)>>],
    ) -> Action {
        use crate::entity::effect::EffectType;

        // Define priority levels for decision making
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        enum Priority {
            Critical, // Health < 20%, Thirst/Hunger > 80%
            Urgent,   // Thirst/Hunger > 50%
            Normal,   // Thirst/Hunger > 30%
            Low,      // General collection/exploration
        }

        // Helper function to compare priorities (higher is better)
        fn priority_level_better(a: Priority, b: Priority) -> bool {
            match (a, b) {
                (Priority::Critical, _) => b != Priority::Critical,
                (Priority::Urgent, Priority::Critical) => false,
                (Priority::Urgent, _) => true,
                (Priority::Normal, Priority::Critical | Priority::Urgent) => false,
                (Priority::Normal, _) => true,
                (Priority::Low, _) => false,
            }
        }

        match &item_box.item {
            Item::Traveler { name } => {
                // Define the entity's possible needs
                #[derive(Debug)]
                enum Need {
                    Water,
                    Food,
                    Items,
                    Exploration,
                }

                let mut health_level = 100;
                let mut hunger_level = 0;
                let mut thirst_level = 0;
                let mut has_critical_health = false;

                // Process all effects to determine the entity's state
                for effect in effects {
                    match &effect.kind {
                        EffectType::Healthy => {
                            health_level = effect.intensity;
                        }
                        EffectType::Hungry => {
                            hunger_level = effect.intensity;
                        }
                        EffectType::Thirsty => {
                            thirst_level = effect.intensity;
                        }
                        EffectType::Injured | EffectType::Sick | EffectType::Poisoned => {
                            if effect.intensity > 50 {
                                has_critical_health = true;
                            }
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

                println!(
                    "Traveler {} primary need: {:?} with priority {:?} (Health: {}, Hunger: {}, Thirst: {})",
                    name, primary_need.1, primary_need.0, health_level, hunger_level, thirst_level
                );

                // ---- 2. Scan surroundings once and act based on priority ----

                // Track the best action found while scanning
                let mut best_action = None;
                let mut best_priority = Priority::Low;
                let mut best_distance = f32::MAX;

                // Scan all surrounding cells in a single loop
                for y in -1isize..1 {
                    for x in -1isize..1 {
                        if x == 1 && y == 1 {
                            continue;
                        } // Skip center (where NPC is)
                        if x < 0 || y < 0 {
                            continue;
                        }

                        // Get the distance from center (Manhattan distance)
                        let distance = ((x - 1).abs() + (y - 1).abs()) as f32;

                        // Check the item at this position
                        if let Some((item_box, idx)) = &surroundings[y as usize][x as usize] {
                            match &item_box.item {
                                // Food items
                                Item::Object(Object::Food(_)) => {
                                    if matches!(primary_need.1, Need::Food) {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::Consume);

                                        // If this is the primary need, it's the best action
                                        if matches!(primary_need.1, Need::Food)
                                            && (priority_level_better(
                                                primary_need.0,
                                                best_priority,
                                            ) || (primary_need.0 == best_priority
                                                && distance < best_distance))
                                        {
                                            best_action = Some(action);
                                            best_priority = primary_need.0;
                                            best_distance = distance;
                                        }
                                    } else if matches!(primary_need.1, Need::Items) {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::PickUp);

                                        if priority_level_better(Priority::Low, best_priority)
                                            || (Priority::Low == best_priority
                                                && distance < best_distance)
                                        {
                                            best_action = Some(action);
                                            best_priority = Priority::Low;
                                            best_distance = distance;
                                        }
                                    }
                                }

                                // Water
                                Item::Water | Item::DeepWater => {
                                    if matches!(primary_need.1, Need::Water) {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::Consume);

                                        if priority_level_better(primary_need.0, best_priority)
                                            || (primary_need.0 == best_priority
                                                && distance < best_distance)
                                        {
                                            best_action = Some(action);
                                            best_priority = primary_need.0;
                                            best_distance = distance;
                                            println!("Found water for thirsty traveler {}", name);
                                        }
                                    }
                                }

                                // Corpses (can be eaten when very hungry)
                                Item::Corpse { .. } => {
                                    if matches!(primary_need.1, Need::Food) && hunger_level > 70 {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::Consume);

                                        if priority_level_better(primary_need.0, best_priority)
                                            || (primary_need.0 == best_priority
                                                && distance < best_distance)
                                        {
                                            best_action = Some(action);
                                            best_priority = primary_need.0;
                                            best_distance = distance;
                                            println!(
                                                "Found food item for hungry traveler {}",
                                                name
                                            );
                                        }
                                    }
                                }

                                // Animals (can be hunted when critically hungry)
                                Item::Animal { .. } => {
                                    if matches!(primary_need.1, Need::Food) && hunger_level > 90 {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::Consume);

                                        if priority_level_better(Priority::Critical, best_priority)
                                            || (Priority::Critical == best_priority
                                                && distance < best_distance)
                                        {
                                            best_action = Some(action);
                                            best_priority = Priority::Critical;
                                            best_distance = distance;
                                        }
                                    }
                                }

                                // Collectible items
                                Item::Object(_) => {
                                    if matches!(primary_need.1, Need::Items) {
                                        let action =
                                            Action::Interact((x, y, *idx), Interaction::PickUp);

                                        if priority_level_better(Priority::Low, best_priority)
                                            || (Priority::Low == best_priority
                                                && distance < best_distance)
                                        {
                                            best_action = Some(action);
                                            best_priority = Priority::Low;
                                            best_distance = distance;
                                        }
                                    }
                                }

                                // Interesting exploration targets
                                Item::Grass | Item::Rock | Item::Log => {
                                    if matches!(primary_need.1, Need::Exploration) {
                                        // Calculate movement direction
                                        let dx = x;
                                        let dy = y;

                                        let direction = match (dx, dy) {
                                            (0, -1) => Direction::North,
                                            (1, 0) => Direction::East,
                                            (0, 1) => Direction::South,
                                            (-1, 0) => Direction::West,
                                            (1, -1) => Direction::NorthEast,
                                            (1, 1) => Direction::SouthEast,
                                            (-1, 1) => Direction::SouthWest,
                                            (-1, -1) => Direction::NorthWest,
                                            _ => Direction::None,
                                        };

                                        if direction != Direction::None {
                                            let action = Action::Move(direction);

                                            if priority_level_better(Priority::Low, best_priority)
                                                || (Priority::Low == best_priority
                                                    && distance < best_distance)
                                            {
                                                best_action = Some(action);
                                                best_priority = Priority::Low;
                                                best_distance = distance;
                                            }
                                        }
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                }

                // If we found a suitable action, return it
                if let Some(action) = best_action {
                    return action;
                }

                // If we didn't find an action in immediate surroundings, check extended view for water
                if matches!(primary_need.1, Need::Water) && primary_need.0 >= Priority::Urgent {
                    for (i, view) in extended.iter().enumerate() {
                        for (j, item) in view.iter().enumerate() {
                            let Some(item) = item else { continue };
                            if matches!(item.0.item, Item::Water | Item::DeepWater) {
                                // Calculate rough direction to the water based on which extended view area it was in
                                let direction = match (i, j) {
                                    (0, 0) => Direction::North,
                                    (0, 1) => Direction::East,
                                    (0, 2) => Direction::South,
                                    (0, 3) => Direction::West,
                                    (1, 0) => Direction::North,
                                    (1, 1) => Direction::East,
                                    (1, 2) => Direction::South,
                                    (1, 3) => Direction::West,
                                    (2, 0) => Direction::North,
                                    (2, 1) => Direction::East,
                                    (2, 2) => Direction::South,
                                    (2, 3) => Direction::West,
                                    (3, 0) => Direction::North,
                                    (3, 1) => Direction::East,
                                    (3, 2) => Direction::South,
                                    (3, 3) => Direction::West,
                                    _ => Direction::None,
                                };

                                if direction != Direction::None {
                                    println!(
                                        "Traveler {} spotted water in distance, moving {}",
                                        name,
                                        match direction {
                                            Direction::North => "north",
                                            Direction::East => "east",
                                            Direction::South => "south",
                                            Direction::West => "west",
                                            Direction::NorthEast => "northeast",
                                            Direction::SouthEast => "southeast",
                                            Direction::SouthWest => "southwest",
                                            Direction::NorthWest => "northwest",
                                            Direction::None => "nowhere",
                                        }
                                    );
                                    return Action::Move(direction);
                                }
                            }
                        }
                    }
                }

                // If no specific action was determined, move randomly
                if fastrand::u8(0..100) < 30 {
                    let directions = [
                        Direction::North,
                        Direction::East,
                        Direction::South,
                        Direction::West,
                    ];

                    // Try random directions until finding a valid move
                    let direction = directions[fastrand::usize(0..directions.len())];

                    // No need to check validity - movement gets validated elsewhere
                    println!("Traveler {} moving randomly: {:?}", name, direction);
                    return Action::Move(direction);
                }

                // Default to waiting
                println!("Traveler {} waiting", name);
                Action::Wait
            }
            Item::Animal { species } => {
                // Animals have simpler behavior patterns but still react to environment

                // Check health condition from effects
                let mut health = 100;
                let mut low_health = false;

                // Find health effects
                for effect in effects {
                    match effect.kind {
                        EffectType::Injured => {
                            health -= effect.intensity as i32;
                            if health < 25 {
                                low_health = true;
                            }
                        }
                        EffectType::Sick => {
                            health -= effect.intensity as i32 / 2;
                            if health < 25 {
                                low_health = true;
                            }
                        }
                        EffectType::Thirsty | EffectType::Hungry => {
                            if effect.intensity > 80 {
                                low_health = true;
                            }
                        }
                        _ => {}
                    }
                }

                // Animals seek water when health is low
                if low_health {
                    for y in 0..3 {
                        for x in 0..3 {
                            let Some((item_box, _)) = &surroundings[y][x] else {
                                continue;
                            };
                            let item = &item_box.item;
                            if let Item::Water = item {
                                let dx = x as isize - 1;
                                let dy = y as isize - 1;

                                if dx == 0 && dy == 0 {
                                    return Action::Wait; // Drink
                                }

                                let direction = match (dx, dy) {
                                    (0, -1) => Direction::North,
                                    (1, 0) => Direction::East,
                                    (0, 1) => Direction::South,
                                    (-1, 0) => Direction::West,
                                    _ => Direction::None,
                                };

                                if direction != Direction::None {
                                    return Action::Move(direction);
                                }
                            }
                        }
                    }
                }

                // Species-specific behavior
                match species.as_str() {
                    "cow" => {
                        // Cows prefer to stay on grass and move slowly
                        // Find grass in surroundings
                        for y in 0..3 {
                            for x in 0..3 {
                                let Some((item_box, _)) = &surroundings[y][x] else {
                                    continue;
                                };
                                let item = &item_box.item;
                                if let Item::Grass = item {
                                    // Found grass, move toward it sometimes
                                    if fastrand::u8(0..100) < 40 {
                                        let dx = x as isize - 1;
                                        let dy = y as isize - 1;

                                        let direction = match (dx, dy) {
                                            (0, -1) => Direction::North,
                                            (1, 0) => Direction::East,
                                            (0, 1) => Direction::South,
                                            (-1, 0) => Direction::West,
                                            _ => Direction::None,
                                        };

                                        if direction != Direction::None {
                                            return Action::Move(direction);
                                        }
                                    }
                                }
                            }
                        }

                        // Cows move randomly but very infrequently
                        if fastrand::u8(0..100) < 10 {
                            let directions = [
                                Direction::North,
                                Direction::East,
                                Direction::South,
                                Direction::West,
                            ];
                            return Action::Move(directions[fastrand::usize(0..4)]);
                        }
                    }
                    "sheep" => {
                        // Sheep tend to flock together
                        // Look for other sheep
                        for y in 0..3 {
                            for x in 0..3 {
                                if x == 1 && y == 1 {
                                    continue; // Skip self
                                }
                                let Some(item) = surroundings[y][x].map(|i| &i.0.item) else {
                                    break;
                                };

                                if let Item::Animal {
                                    species: other_species,
                                    ..
                                } = &item
                                {
                                    if other_species == "sheep" {
                                        // Found another sheep, move toward it sometimes
                                        if fastrand::u8(0..100) < 60 {
                                            let dx = x as isize - 1;
                                            let dy = y as isize - 1;

                                            let direction = match (dx, dy) {
                                                (0, -1) => Direction::North,
                                                (1, 0) => Direction::East,
                                                (0, 1) => Direction::South,
                                                (-1, 0) => Direction::West,
                                                _ => Direction::None,
                                            };

                                            if direction != Direction::None {
                                                return Action::Move(direction);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Sheep move randomly more often than cows
                        if fastrand::u8(0..100) < 25 {
                            let directions = [
                                Direction::North,
                                Direction::East,
                                Direction::South,
                                Direction::West,
                            ];
                            return Action::Move(directions[fastrand::usize(0..4)]);
                        }
                    }
                    "bird" => {
                        // Birds move frequently and prefer logs or rocks to perch on
                        let mut found_perch = false;

                        // Look for logs or rocks
                        for y in 0..3 {
                            for x in 0..3 {
                                let Some(item) = surroundings[y][x].map(|i| &i.0.item) else {
                                    break;
                                };
                                match item {
                                    Item::Log | Item::Rock => {
                                        found_perch = true;
                                        // Move toward perch
                                        if fastrand::u8(0..100) < 70 {
                                            let dx = x as isize - 1;
                                            let dy = y as isize - 1;

                                            let direction = match (dx, dy) {
                                                (0, -1) => Direction::North,
                                                (1, 0) => Direction::East,
                                                (0, 1) => Direction::South,
                                                (-1, 0) => Direction::West,
                                                _ => Direction::None,
                                            };

                                            if direction != Direction::None {
                                                return Action::Move(direction);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Birds move randomly very frequently
                        if !found_perch || fastrand::u8(0..100) < 60 {
                            let directions = [
                                Direction::North,
                                Direction::East,
                                Direction::South,
                                Direction::West,
                                Direction::NorthEast,
                                Direction::SouthEast,
                                Direction::SouthWest,
                                Direction::NorthWest,
                            ];
                            return Action::Move(directions[fastrand::usize(0..8)]);
                        }
                    }
                    _ => {
                        // Generic animal behavior for unknown species
                        if fastrand::u8(0..100) < 30 {
                            let directions = [
                                Direction::North,
                                Direction::East,
                                Direction::South,
                                Direction::West,
                            ];
                            return Action::Move(directions[fastrand::usize(0..4)]);
                        }
                    }
                }

                Action::Wait
            }
            _ => Action::Wait,
        }
    }

    /// Move an item from one position to another
    // Apply effects over time to all items in the world
    fn apply_effects(&mut self) {
        // Use a fixed delta time for simplicity (could be passed as a parameter)
        let delta = std::time::Duration::from_secs(1);

        // Update all items with effects
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(stack) = self.get_mut(x, y) {
                    for item_box in stack.items.iter_mut() {
                        // Update effects that have durations
                        item_box.effects.retain_mut(|effect| effect.update(delta));

                        // Apply natural effects based on item type
                        let mut replace_item = Option::<ItemBox>::None;
                        match &item_box.item {
                            Item::Traveler { name } | Item::Animal { species: name } => {
                                // Gradually increase hunger and thirst
                                let mut has_hunger = false;
                                let mut has_thirst = false;
                                let mut health_level = 100; // Default health level
                                let mut hunger_level = 0;
                                let mut thirst_level = 0;

                                // First pass: gather information about current effects
                                for effect in &item_box.effects {
                                    match effect.kind {
                                        EffectType::Hungry => {
                                            has_hunger = true;
                                            hunger_level = effect.intensity;
                                        }
                                        EffectType::Thirsty => {
                                            has_thirst = true;
                                            thirst_level = effect.intensity;
                                        }
                                        EffectType::Healthy => {
                                            health_level = effect.intensity;
                                        }
                                        _ => {}
                                    }
                                }

                                // Second pass: update effect intensities
                                let effects = item_box.effects.clone();
                                let mut new_effects = Vec::new();
                                for effect in &mut item_box.effects {
                                    match effect.kind {
                                        EffectType::Hungry => {
                                            // Save old intensity for reporting
                                            let old_intensity = effect.intensity;

                                            // Increase hunger over time (slower)
                                            effect.intensity = (effect.intensity + 1).min(100);

                                            // Report significant hunger changes
                                            if effect.intensity > 80 && old_intensity <= 80 {
                                                println!(
                                                    "{} is now severely hungry ({}%)",
                                                    name, effect.intensity
                                                );
                                            }
                                        }
                                        EffectType::Thirsty => {
                                            // Save old intensity for reporting
                                            let old_intensity = effect.intensity;

                                            // Increase thirst over time (faster than hunger)
                                            effect.intensity = (effect.intensity + 2).min(100);

                                            // Report significant thirst changes
                                            if effect.intensity > 80 && old_intensity <= 80 {
                                                println!(
                                                    "{} is now severely thirsty ({}%)",
                                                    name, effect.intensity
                                                );
                                            }
                                        }
                                        EffectType::Healthy => {
                                            // Only reduce health if extremely hungry or thirsty
                                            let mut severe_condition = false;

                                            for other_effect in &effects {
                                                match other_effect.kind {
                                                    EffectType::Hungry | EffectType::Thirsty
                                                        if other_effect.intensity > 80 =>
                                                    {
                                                        severe_condition = true;
                                                        break;
                                                    }
                                                    _ => {}
                                                }
                                            }

                                            if severe_condition && effect.intensity > 0 {
                                                // Reduce health if starving/dehydrated
                                                let prev_health = effect.intensity;
                                                effect.intensity =
                                                    effect.intensity.saturating_sub(1);

                                                if prev_health != effect.intensity {
                                                    println!(
                                                        "{}'s health decreased to {}% due to starvation/dehydration (hunger: {}%, thirst: {}%)",
                                                        name,
                                                        effect.intensity,
                                                        hunger_level,
                                                        thirst_level
                                                    );
                                                }

                                                // If health reaches low level, add injured effect
                                                if effect.intensity < 20
                                                    && !effects.iter().any(|e| {
                                                        matches!(e.kind, EffectType::Injured)
                                                    })
                                                {
                                                    new_effects.push(Effect::permanent(
                                                        EffectType::Injured,
                                                        50,
                                                    ));
                                                    println!(
                                                        "{} became injured due to poor health",
                                                        name
                                                    );
                                                }

                                                // If health is zero, add dead effect and convert to corpse
                                                if effect.intensity == 0
                                                    && !effects
                                                        .iter()
                                                        .any(|e| matches!(e.kind, EffectType::Dead))
                                                {
                                                    new_effects.push(Effect::permanent(
                                                        EffectType::Dead,
                                                        100,
                                                    ));
                                                    println!(
                                                        "{} has died due to health reaching zero",
                                                        name
                                                    );

                                                    // Mark for conversion to corpse
                                                    replace_item =
                                                        Some(ItemBox::new(match &item_box.item {
                                                            Item::Traveler { name } => {
                                                                Item::Corpse {
                                                                    name: name.clone(),
                                                                    item_type: CorpseType::Traveler,
                                                                }
                                                            }
                                                            Item::Animal { species } => {
                                                                Item::Corpse {
                                                                    name: name.clone(),
                                                                    item_type: CorpseType::Animal(
                                                                        species.clone(),
                                                                    ),
                                                                }
                                                            }
                                                            _ => item_box.item.clone(),
                                                        }));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                item_box.effects.extend(new_effects);

                                // Add hunger effect if not present
                                if !has_hunger {
                                    println!("{} gained hunger status", name);
                                    item_box
                                        .effects
                                        .push(Effect::permanent(EffectType::Hungry, 10));
                                }

                                // Add thirst effect if not present
                                if !has_thirst {
                                    println!("{} gained thirst status", name);
                                    item_box
                                        .effects
                                        .push(Effect::permanent(EffectType::Thirsty, 15));
                                }

                                // Add health effect if not present
                                if health_level == 100
                                    && !item_box
                                        .effects
                                        .iter()
                                        .any(|e| matches!(e.kind, EffectType::Healthy))
                                {
                                    item_box
                                        .effects
                                        .push(Effect::permanent(EffectType::Healthy, 100));
                                }
                            }
                            _ => {}
                        }
                        if let Some(replace_item) = replace_item {
                            *item_box = replace_item;
                        }
                    }
                }
            }
        }
    }

    // Helper method to check if a move in a direction is possible
    fn can_move_towards(&self, x: usize, y: usize, direction: Direction) -> Option<(usize, usize)> {
        let (dx, dy) = match direction {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
            Direction::NorthWest => (-1, -1),
            Direction::None => return None, // No movement
        };

        let new_x = x as isize + dx;
        let new_y = y as isize + dy;

        // Check if the new position is within bounds
        if new_x < 0 || new_x >= self.width as isize || new_y < 0 || new_y >= self.height as isize {
            return None;
        }

        let new_x = new_x as usize;
        let new_y = new_y as usize;

        // Check if the destination is traversable
        if let Some(item) = self.grid[new_y][new_x].visible_item() {
            if item.0.is_traversable() {
                return Some((new_x, new_y));
            }
        }

        None
    }

    fn move_item(&mut self, x: usize, y: usize, direction: Direction) -> bool {
        let (dx, dy) = match direction {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
            Direction::NorthWest => (-1, -1),
            Direction::None => return false, // No movement for None direction
        };

        let new_x = x as isize + dx;
        let new_y = y as isize + dy;

        // Check if the new position is within bounds
        if new_x < 0 || new_x >= self.width as isize || new_y < 0 || new_y >= self.height as isize {
            return false;
        }

        let new_x = new_x as usize;
        let new_y = new_y as usize;

        // Check if the destination contains any non-traversable items
        if self.grid[new_y][new_x]
            .visible_item()
            .map(|i| !i.0.is_traversable())
            .unwrap_or(true)
        {
            return false;
        }

        // Check if the visible item at destination is water
        let destination_is_water = self.grid[new_y][new_x]
            .visible_item()
            .map(|item_box| matches!(item_box.0.item, Item::Water | Item::DeepWater))
            .unwrap_or(false);

        if destination_is_water {
            // Get the item that's moving
            if let Some(actor) = self.grid[y][x].find_actor() {
                // Check if the actor can traverse water
                let can_swim = actor.can_traverse_water();

                // If actor can't swim and the destination top item is water, prevent movement
                if !can_swim {
                    return false; // Can't move into water
                }
                // Else: can traverse water, continue with movement
            } else {
                return false; // No actor found, can't move
            }
        }

        // Find and remove the actor from the source stack
        let actor = self.grid[y][x].find_actor().map(|a| a.clone());

        if let Some(actor_box) = actor {
            // Remove the original actor
            if let Some(index) = self.grid[y][x]
                .items()
                .iter()
                .position(|item| matches!(item.item, Item::Traveler { .. } | Item::Animal { .. }))
            {
                self.grid[y][x].items.remove(index);

                // Add the actor to the destination stack
                self.grid[new_y][new_x].push(actor_box);

                println!(
                    "Successfully moved actor from ({},{}) to ({},{})",
                    x, y, new_x, new_y
                );
                true
            } else {
                println!("Failed to find actor in stack at ({},{})", x, y);
                false
            }
        } else {
            println!("No actor found at ({},{})", x, y);
            false
        }
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "World {}x{} (seed: {})",
            self.width, self.height, self.seed
        )?;

        // Print towns
        let towns = self.get_towns();
        writeln!(f, "Towns: {}", towns.len())?;
        for (name, (x, y)) in &towns {
            writeln!(f, "- {} at ({}, {})", name, x, y)?;
        }

        // Print the grid
        for row in &self.grid {
            for stack in row {
                write!(
                    f,
                    "{} ",
                    stack.visible_item().map(|i| i.0.get_char()).unwrap_or(' ')
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
