use crate::entity::effect::{Effect, EffectType, FoodType, Object};
use crate::entity::item::{
    self, Action, CorpseType, Direction, Interaction, Item, ItemBox, Need, Priority,
};
use crate::world::town_gen::TownGenerator;
use crate::world::world_gen::{WorldGenParams, WorldGenerator};
use crate::{DEFAULT_HEALTH, FOOD_SPAWN_RATE};

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;

// Node for A* pathfinding
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    position: (usize, usize),
    f_score: usize, // Total estimated cost (g_score + heuristic)
    g_score: usize, // Cost from start to this node
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order so lowest score has highest priority
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A stack of items in a single grid cell
#[derive(Clone)]
pub struct ItemStack {
    pub items: Vec<ItemBox>,
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

    /// Generate a procedural world using advanced terrain generation
    pub fn generate(&mut self) {
        // Configure parameters for the world generator
        let params = WorldGenParams {
            seed: self.seed,
            elevation_scale: 0.06,    // Lower scale for larger features
            moisture_scale: 0.05,     // Lower scale for moisture patterns
            vegetation_scale: 0.12,   // Lower scale for vegetation patterns
            sea_level: 0.32,          // Higher sea level for more islands
            beach_level: 0.36,        // Adjusted beach level
            elevation_amplitude: 1.3, // Higher amplitude for more dramatic terrain

            river_count: 20,  // More rivers
            river_width: 0.5, // Slightly thinner rivers
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

    // Find a path using A* algorithm, returning a vector of positions or None if no path found
    /// Find a path from start position to a target using A* pathfinding
    ///
    /// * `start` - Starting position (x, y)
    /// * `is_target` - Function that returns true if the item is a target
    /// * `max_distance` - Maximum search distance
    pub fn find_path(
        &self,
        start: (usize, usize),
        is_target: impl Fn(&ItemBox) -> bool,
        max_distance: usize,
    ) -> Option<Vec<(usize, usize)>> {
        // Create open and closed sets
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_scores = HashMap::new();

        // Make sure start position is valid
        if start.0 >= self.width || start.1 >= self.height {
            println!("Invalid start position for pathfinding: {:?}", start);
            return None;
        }

        // Initialize with start node
        open_set.push(PathNode {
            position: start,
            f_score: 0,
            g_score: 0,
        });
        g_scores.insert(start, 0);

        // Add iteration limit to prevent infinite loops
        let mut iterations = 0;
        let max_iterations = 2000; // Increased limit for larger search distances

        while let Some(current) = open_set.pop() {
            let (current_x, current_y) = current.position;

            // Increment iteration counter and check limit
            iterations += 1;
            if iterations > max_iterations {
                return None;
            }

            // Check if we've reached the target
            if let Some(stack) = self.get(current_x, current_y) {
                if let Some((item_box, _)) = stack.visible_item() {
                    if is_target(item_box) {
                        // Reconstruct and return the path
                        let mut path = vec![current.position];
                        let mut current_pos = current.position;

                        while let Some(prev) = came_from.get(&current_pos) {
                            path.push(*prev);
                            current_pos = *prev;
                        }

                        path.reverse();
                        // println!(
                        //     "Path found from {:?} to {:?} with {} steps (iterations: {})",
                        //     start,
                        //     current.position,
                        //     path.len(),
                        //     iterations
                        // );
                        return Some(path);
                    }
                }
            }

            // If we've exceeded max distance, stop exploring this path
            if current.g_score > max_distance {
                continue;
            }

            // Mark current node as visited
            closed_set.insert(current.position);

            // Check all neighbors (8 directions for smoother pathfinding)
            let neighbors = [
                (current_x.saturating_add(1), current_y), // East
                (current_x, current_y.saturating_add(1)), // South
                (current_x.saturating_sub(1), current_y), // West
                (current_x, current_y.saturating_sub(1)), // North
                (current_x.saturating_add(1), current_y.saturating_sub(1)), // Northeast
                (current_x.saturating_add(1), current_y.saturating_add(1)), // Southeast
                (current_x.saturating_sub(1), current_y.saturating_add(1)), // Southwest
                (current_x.saturating_sub(1), current_y.saturating_sub(1)), // Northwest
            ];

            for &neighbor_pos in &neighbors {
                let (nx, ny) = neighbor_pos;

                // Skip if out of bounds
                if nx >= self.width || ny >= self.height {
                    continue;
                }

                // Skip if already visited
                if closed_set.contains(&neighbor_pos) {
                    continue;
                }

                // Skip if not passable
                // Allow the target to be found even if it's not normally passable
                let is_target_item = self
                    .get(nx, ny)
                    .and_then(|stack| stack.visible_item())
                    .is_none_or(|(box_item, _)| is_target(box_item));

                // Skip impassable terrain (unless it's the target)
                if !self.is_passable(nx, ny) && !is_target_item {
                    // Allow pathfinding to go through high-cost terrain if it's the only way
                    // But it will strongly prefer lower-cost paths when available
                    continue;
                }

                // Calculate the terrain cost for this position
                let terrain_cost = self.get_terrain_cost(nx, ny);

                // Add diagonal movement penalty (√2 ≈ 1.414 times more expensive)
                let move_cost = if current_x != nx && current_y != ny {
                    // Diagonal movement
                    (terrain_cost as f32 * 1.414) as usize
                } else {
                    // Cardinal movement
                    terrain_cost
                };

                // Calculate new g_score with terrain cost
                let tentative_g_score = current.g_score + move_cost;

                // If this path to neighbor is better than any previous one
                if !g_scores.contains_key(&neighbor_pos)
                    || tentative_g_score < *g_scores.get(&neighbor_pos).unwrap()
                {
                    // Update path and scores
                    came_from.insert(neighbor_pos, current.position);
                    g_scores.insert(neighbor_pos, tentative_g_score);

                    // Calculate f_score as g_score only - no heuristic needed since
                    // we're doing a radial search for any matching target rather than
                    // a directed search to a specific destination
                    let f_score = tentative_g_score;

                    // Check if this node is already in the open set
                    let in_open_set = open_set.iter().any(|node| node.position == neighbor_pos);

                    // If it's not in the open set, or if we found a better path, update it
                    if !in_open_set {
                        open_set.push(PathNode {
                            position: neighbor_pos,
                            f_score,
                            g_score: tentative_g_score,
                        });
                    } else if tentative_g_score < g_scores[&neighbor_pos] {
                        // Since BinaryHeap doesn't support updating elements in-place,
                        // we'll just push the new node with better score
                        // (the old one will be ignored when it's encountered)
                        open_set.push(PathNode {
                            position: neighbor_pos,
                            f_score,
                            g_score: tentative_g_score,
                        });
                    }
                }
            }
        }
        None
    }

    // Check if a position is passable
    fn is_passable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        // Get terrain cost - anything with a cost of 50+ is considered impassable
        let terrain_cost = self.get_terrain_cost(x, y);
        terrain_cost < 50
    }

    // Get the terrain cost for pathfinding (lower is better)
    fn get_terrain_cost(&self, x: usize, y: usize) -> usize {
        if let Some(stack) = self.get(x, y) {
            if let Some((item_box, _)) = stack.visible_item() {
                match &item_box.item {
                    // Road is best for travel
                    Item::Road { .. } => 1,
                    Item::Bridge => 1,

                    // Land is preferred for walking
                    Item::Dirt => 10,
                    Item::Grass => 10,
                    Item::Sand => 10,

                    // Obstacles are much harder to traverse
                    Item::Log => 13,
                    Item::Rock => 15,
                    Item::Mountain => 30,

                    // Water is difficult - travelers prefer to avoid it
                    Item::Water => 20,
                    Item::DeepWater => 65, // Almost impassable

                    // Snow is slow but easier than water
                    Item::Snow => 12,

                    // Buildings are traversable but not preferred paths
                    Item::House { .. } => 10,
                    Item::Shop { .. } => 10,
                    Item::Tavern { .. } => 10,
                    Item::Temple { .. } => 10,

                    // Objects should be easy to navigate around or pick up
                    Item::Object(_) => 7,

                    // Default cost for other terrain types
                    _ => 10,
                }
            } else {
                10 // Default cost
            }
        } else {
            99 // Very high cost for invalid positions
        }
    }

    // Get direction to move from current position to next position in path
    fn get_direction_to(&self, from: (usize, usize), to: (usize, usize)) -> Direction {
        let (from_x, from_y) = from;
        let (to_x, to_y) = to;

        // Handle potential integer underflow with usize by explicitly checking
        let dx = match to_x.cmp(&from_x) {
            Ordering::Greater => 1,
            Ordering::Less => -1,
            Ordering::Equal => 0,
        };

        let dy = match to_y.cmp(&from_y) {
            Ordering::Greater => 1,
            Ordering::Less => -1,
            Ordering::Equal => 0,
        };

        match (dx, dy) {
            (0, -1) => Direction::North,
            (1, 0) => Direction::East,
            (0, 1) => Direction::South,
            (-1, 0) => Direction::West,
            (1, -1) => Direction::NorthEast,
            (1, 1) => Direction::SouthEast,
            (-1, 1) => Direction::SouthWest,
            (-1, -1) => Direction::NorthWest,
            _ => Direction::None,
        }
    }

    /// Process one tick of the world, allowing all actors to take their actions
    pub fn tick(&mut self) {
        // Periodically spawn food in the world (adjusted for balance)
        if fastrand::u8(0..100) < FOOD_SPAWN_RATE {
            self.spawn_food();
        }

        // Process actors first, then apply effects
        // This ensures that any thirst/hunger reductions apply before increasing them again

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

            // Then get a mutable reference and determine the action
            let width = self.width();
            let height = self.height();
            if let Some(mut stack) = stack {
                if let Some(actor) = stack.find_actor() {
                    // Skip actors that can't act
                    if !actor.can_act() {
                        continue;
                    }
                    // Clone the actor to avoid borrow issues
                    let actor_clone = actor.clone();
                    let effects = actor_clone.effects.clone();

                    // Get action based on surroundings
                    let action = match &actor_clone.item {
                        Item::Traveler { .. } => {
                            self.decide_action(actor, &effects, &surroundings, x, y)
                        }
                        _ => Action::Wait,
                    };
                    self.update_actor(x, y, actor.clone());

                    // Execute the action
                    match action {
                        Action::Move(direction) => {
                            self.move_item(x, y, direction);
                        }
                        Action::Interact(targ_coords, held_index, interaction) => {
                            // Process the interaction
                            // Create a mutable copy of the actor to work with

                            // Process the interaction with direct modification
                            let (success, message) = ItemBox::process_interaction(
                                actor,
                                targ_coords.and_then(|c| {
                                    let tx = (x as isize + c.0) as usize;
                                    let ty = (y as isize + c.1) as usize;
                                    self.get_mut(tx, ty).map(|s| {
                                    println!(
                                        "Actor at ({}, {}) interacting with ({}, {}), item: {:?}",
                                        x,
                                        y,
                                        tx,
                                        ty,
                                        s.visible_item().map(|(item, _)| &item.item)
                                    );
                                        (s, c.2)
                                    })
                                }),
                                held_index,
                                interaction,
                            );

                            // Update the actor with the modified version
                            if success {
                                println!("Interaction succeeded: {}", message);

                                // Replace the actor with the modified version that has updated effects - need to update actor at original position
                            } else {
                                println!("Interaction failed: {}", message);
                            }
                            self.update_actor(x, y, actor.clone());
                        }
                        Action::Wait => {
                            // Do nothing
                        }
                    }
                }
            }
        }

        // Apply effects over time to all items after processing actions
        self.apply_effects();
    }

    fn update_actor(&mut self, x: usize, y: usize, actor: ItemBox) {
        if let Some(i) = self.grid[y][x]
            .items
            .iter_mut()
            .position(|item_box| item_box.can_act())
        {
            self.grid[y][x].items[i] = actor.clone();
        }
    }

    /// Spawns a random food item somewhere in the world
    fn spawn_food(&mut self) {
        let max_attempts = 50;

        for _ in 0..max_attempts {
            // Pick a random position
            let x = fastrand::usize(0..self.width);
            let y = fastrand::usize(0..self.height);

            // Check if the position has grass (food should grow on grass)
            if let Some(stack) = self.get(x, y) {
                let has_grass = stack
                    .items()
                    .iter()
                    .any(|item| matches!(item.item, Item::Grass));
                let has_traveler = stack
                    .items()
                    .iter()
                    .any(|item| matches!(item.item, Item::Traveler { .. }));
                let has_food = stack
                    .items()
                    .iter()
                    .any(|item| matches!(item.item, Item::Object(Object::Food(_))));

                // Only spawn food on grass where there isn't already food or a traveler
                if has_grass && !has_traveler && !has_food {
                    // Create a random food item
                    let food_type = match fastrand::usize(0..3) {
                        0 => FoodType::Bread,
                        1 => FoodType::Fruit,
                        _ => FoodType::Vegetable,
                    };

                    let food_item = Item::Object(Object::Food(food_type));

                    // Add the food to the world
                    self.add_item(x, y, food_item);
                    println!("Spawned new food item at ({}, {})", x, y);
                    return;
                }
            }
        }
    }
    /// Decide what action an item should take
    fn decide_action(
        &self,
        item_box: &mut ItemBox,
        effects: &[Effect],
        surroundings: &[Vec<Option<(&ItemBox, usize)>>],
        x: usize,
        y: usize,
    ) -> Action {
        match item_box.item.clone() {
            Item::Traveler { name } => {
                let primary_need = item_box.primary_need();
                let preferred_direction = item_box.preferred_direction();
                item_box.prefer_direction(None);

                // ---- 2. Scan surroundings once and act based on priority ----

                // Track the best action found while scanning
                let mut best_action = None;
                let mut best_priority = Priority::None;

                // Scan all surrounding cells in a single loop
                for (y_idx, row) in surroundings.iter().enumerate() {
                    for (x_idx, cell) in row.iter().enumerate() {
                        if x_idx == 1 && y_idx == 1 {
                            continue;
                        } // Skip center (where NPC is)

                        // Convert to relative coordinates (-1, 0, 1)
                        let rx = x_idx as isize - 1;
                        let ry = y_idx as isize - 1;

                        // Check the item at this position
                        if let Some((target_item, idx)) = cell {
                            // Helper to update best action if it has higher priority or same with lower distance
                            let mut update_if_better = |action, priority| {
                                if priority > best_priority {
                                    best_action = Some(action);
                                    best_priority = priority;
                                    true
                                } else {
                                    false
                                }
                            };

                            match &target_item.item {
                                // Food items
                                Item::Object(Object::Food(_)) => {
                                    if matches!(primary_need.1, Need::Food) {
                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Consume,
                                        );
                                        if update_if_better(action, primary_need.0) {
                                            // println!(
                                            //     "Found food item for hungry traveler {}",
                                            //     name
                                            // );
                                            item_box.think("I'm going to find food".to_string());
                                        }
                                    } else if matches!(primary_need.1, Need::Items) {
                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::PickUp,
                                        );
                                        update_if_better(action, Priority::Low);
                                        item_box.think("I'm going to pick up food".to_string());
                                    }
                                }

                                // Water
                                Item::Water | Item::DeepWater => {
                                    if matches!(primary_need.1, Need::Water) {
                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Consume,
                                        );
                                        if update_if_better(action, primary_need.0) {
                                            // println!("Found water for thirsty traveler {}", name);
                                            item_box
                                                .think("I'm going to look for water".to_string());
                                        }
                                    }
                                }

                                // Corpses (can be eaten when very hungry)
                                Item::Corpse { .. } => {
                                    if matches!(primary_need.1, Need::Food) {
                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Consume,
                                        );
                                        if update_if_better(action, primary_need.0) {
                                            // println!("Found corpse for hungry traveler {}", name);
                                            item_box
                                                .think("I'm going to eat this corpse".to_string());
                                        }
                                    }
                                }

                                // Collectible items
                                Item::Object(_) => {
                                    if matches!(primary_need.1, Need::Items) {
                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::PickUp,
                                        );
                                        update_if_better(action, Priority::Low);
                                        item_box.think("I want to pick this up".to_string());
                                    }
                                }

                                // // Interesting exploration targets
                                // Item::Grass | Item::Rock | Item::Log => {
                                //     if matches!(primary_need.1, Need::Exploration) {
                                //         let direction = match (rx, ry) {
                                //             (0, -1) => Direction::North,
                                //             (1, 0) => Direction::East,
                                //             (0, 1) => Direction::South,
                                //             (-1, 0) => Direction::West,
                                //             (1, -1) => Direction::NorthEast,
                                //             (1, 1) => Direction::SouthEast,
                                //             (-1, 1) => Direction::SouthWest,
                                //             (-1, -1) => Direction::NorthWest,
                                //             _ => Direction::None,
                                //         };

                                //         if direction != Direction::None {
                                //             let action = Action::Move(direction);
                                //             update_if_better(action, Priority::Low);
                                //             item_box.think("I'm going to explore".to_string());
                                //         }
                                //     }
                                // }
                                Item::Traveler { name: other_name } => {
                                    // 30% chance of mating when conditions are right
                                    if !target_item.young() && fastrand::f32() < 0.30 {
                                        println!(
                                            "Traveler {} attempting to mate with {}",
                                            name, other_name
                                        );
                                        item_box.think("I'm going to mate!".to_string());

                                        let action = Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Mate,
                                        );
                                        update_if_better(action, Priority::Liesure);
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

                // Drop babies
                if !item_box.tired() {
                    if let Some(pos) = item_box.effects.iter().position(|e| {
                        matches!(
                            e.kind,
                            EffectType::Holding(ItemBox {
                                item: Item::Traveler { .. },
                                effects: _,
                            })
                        )
                    }) {
                        return Action::Interact(Some((0, 0, 0)), Some(pos), Interaction::Drop);
                    }
                }

                // Eat held food
                if matches!(primary_need.1, Need::Food) {
                    if let Some(pos) = item_box.effects.iter().position(|e| {
                        matches!(
                            e.kind,
                            EffectType::Holding(ItemBox {
                                item: Item::Object(Object::Food(_)),
                                effects: _,
                            })
                        )
                    }) {
                        return Action::Interact(None, Some(pos), Interaction::Consume);
                    }
                }

                // If we didn't find an action in immediate surroundings, try A* pathfinding
                // Try to find water if that's the primary need
                if matches!(primary_need.1, Need::Water) && primary_need.0 >= Priority::Urgent {
                    // Get the current position of the actor
                    let current_pos = (x, y);

                    let path = self.find_path(
                        current_pos,
                        |item_box| matches!(item_box.item, Item::Water | Item::DeepWater),
                        20,
                    );

                    if let Some(path) = path {
                        if path.len() > 1 {
                            let next_step = path[1]; // First step after current position
                            let direction = self.get_direction_to(current_pos, next_step);

                            if direction != Direction::None {
                                // Calculate terrain type for the next step
                                let terrain_type =
                                    if let Some(stack) = self.get(next_step.0, next_step.1) {
                                        if let Some((item_box, _)) = stack.visible_item() {
                                            format!("{:?}", item_box.item)
                                        } else {
                                            "unknown".to_string()
                                        }
                                    } else {
                                        "unknown".to_string()
                                    };

                                // println!(
                                //     "Traveler {} found path to water, moving {} (via {})",
                                //     name,
                                //     match direction {
                                //         Direction::North => "north",
                                //         Direction::East => "east",
                                //         Direction::South => "south",
                                //         Direction::West => "west",
                                //         Direction::NorthEast => "northeast",
                                //         Direction::SouthEast => "southeast",
                                //         Direction::SouthWest => "southwest",
                                //         Direction::NorthWest => "northwest",
                                //         Direction::None => "nowhere",
                                //     },
                                //     terrain_type
                                // );
                                item_box.think("I know where to go to find water".to_string());
                                return Action::Move(direction);
                            }
                        }
                    }
                }

                // Try to find food if that's the primary need
                if matches!(primary_need.1, Need::Food) && primary_need.0 >= Priority::Urgent {
                    // Get the current position of the actor
                    let current_pos = (x, y);

                    let path = self.find_path(
                        current_pos,
                        |item_box| matches!(item_box.item, Item::Object(Object::Food(_))),
                        20,
                    );

                    if let Some(path) = path {
                        if path.len() > 1 {
                            let next_step = path[1]; // First step after current position
                            let direction = self.get_direction_to(current_pos, next_step);

                            if direction != Direction::None {
                                // Calculate terrain type for the next step
                                let terrain_type =
                                    if let Some(stack) = self.get(next_step.0, next_step.1) {
                                        if let Some((item_box, _)) = stack.visible_item() {
                                            format!("{:?}", item_box.item)
                                        } else {
                                            "unknown".to_string()
                                        }
                                    } else {
                                        "unknown".to_string()
                                    };

                                // println!(
                                //     "Traveler {} found path to food, moving {} (via {})",
                                //     name,
                                //     match direction {
                                //         Direction::North => "north",
                                //         Direction::East => "east",
                                //         Direction::South => "south",
                                //         Direction::West => "west",
                                //         Direction::NorthEast => "northeast",
                                //         Direction::SouthEast => "southeast",
                                //         Direction::SouthWest => "southwest",
                                //         Direction::NorthWest => "northwest",
                                //         Direction::None => "nowhere",
                                //     },
                                //     terrain_type
                                // );
                                item_box.think("I know where to go to find food".to_string());
                                return Action::Move(direction);
                            }
                        }
                    }
                }

                // If no specific action was determined, move with directional persistence
                // Get the current persistent direction for this entity
                let direction = if let Some(dir) = preferred_direction {
                    // Check if we hit a barrier in that direction
                    let (nx, ny) = match dir {
                        Direction::North => (x, y.saturating_sub(1)),
                        Direction::East => (x + 1, y),
                        Direction::South => (x, y + 1),
                        Direction::West => (x.saturating_sub(1), y),
                        _ => (x, y),
                    };

                    // Check if the new position is passable
                    if nx < self.width && ny < self.height && self.is_passable(nx, ny) {
                        dir
                    } else {
                        // Hit a barrier, pick a new random direction
                        let directions = [
                            Direction::North,
                            Direction::East,
                            Direction::South,
                            Direction::West,
                        ];

                        let new_dir = directions[fastrand::usize(0..directions.len())];
                        println!(
                            "Traveler {} hit barrier, new direction: {:?}",
                            name, new_dir
                        );

                        // Set the new direction for persistence
                        // Can't set direction here because self is immutable
                        // Direction will be set on next tick
                        new_dir
                    }
                } else {
                    // Pick a new random direction and record it
                    let directions = [
                        Direction::North,
                        Direction::East,
                        Direction::South,
                        Direction::West,
                    ];

                    // Select a random direction
                    let new_dir = directions[fastrand::usize(0..directions.len())];

                    item_box.prefer_direction(Some(new_dir));
                    new_dir
                };
                if primary_need.0 > Priority::Normal {
                    item_box.think("I don't think I can find what I need here".to_string());
                } else {
                    item_box.think("I'm going to explore".to_string());
                }

                // No need to check validity - movement gets validated elsewhere
                Action::Move(direction)
            }
            _ => Action::Wait,
        }
    }

    /// Move an item from one position to another
    // Apply effects over time to all items in the world
    fn apply_effects(&mut self) {
        // Update all items with effects
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(stack) = self.get_mut(x, y) {
                    for item_box in stack.items.iter_mut() {
                        // Update effects that have durations
                        item_box.effects.retain_mut(|effect| effect.update());

                        // Apply natural effects based on item type
                        let mut replace_item = Option::<ItemBox>::None;
                        if let Item::Traveler { name } = &item_box.item {
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

                                        if matches!(effect.kind, EffectType::Thirsty) {
                                            // Increase thirst over time (faster than hunger)
                                            effect.intensity = (effect.intensity + 1).min(100);
                                        }

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
                                            effect.intensity = effect.intensity.saturating_sub(1);

                                            // if prev_health != effect.intensity {
                                            //     println!(
                                            //         "{}'s health decreased to {}% (hunger: {}%, thirst: {}%)",
                                            //         name,
                                            //         effect.intensity,
                                            //         hunger_level,
                                            //         thirst_level
                                            //     );
                                            // }

                                            // If health reaches low level, add injured effect
                                            if effect.intensity < 20
                                                && !effects
                                                    .iter()
                                                    .any(|e| matches!(e.kind, EffectType::Injured))
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
                                                new_effects
                                                    .push(Effect::permanent(EffectType::Dead, 100));
                                                println!(
                                                    "{} has died due to health reaching zero",
                                                    name
                                                );

                                                // Mark for conversion to corpse
                                                replace_item =
                                                    Some(ItemBox::new(match &item_box.item {
                                                        Item::Traveler { name } => Item::Corpse {
                                                            name: name.clone(),
                                                            item_type: CorpseType::Traveler,
                                                        },
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
                                item_box
                                    .effects
                                    .push(Effect::permanent(EffectType::Hungry, 10));
                            }

                            // Add thirst effect if not present
                            if !has_thirst {
                                item_box
                                    .effects
                                    .push(Effect::permanent(EffectType::Thirsty, 15));
                            }

                            // Add health effect if not present
                            if health_level == DEFAULT_HEALTH
                                && !item_box
                                    .effects
                                    .iter()
                                    .any(|e| matches!(e.kind, EffectType::Healthy))
                            {
                                item_box
                                    .effects
                                    .push(Effect::permanent(EffectType::Healthy, DEFAULT_HEALTH));
                            }
                        }

                        if let Some(replace_item) = replace_item {
                            *item_box = replace_item;
                        }
                    }
                }
            }
        }
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

        // Find and remove the actor from the source stack
        let actor = self.grid[y][x].find_actor().map(|a| a.clone());

        if let Some(actor_box) = actor {
            // Remove the original actor
            if let Some(index) = self.grid[y][x]
                .items()
                .iter()
                .position(|item| matches!(item.item, Item::Traveler { .. }))
            {
                self.grid[y][x].items.remove(index);

                // Add the actor to the destination stack
                self.grid[new_y][new_x].push(actor_box);
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
