use crate::entity::effect::{Effect, EffectType, Farm, FoodType, Object, Profession};
use crate::entity::item::{Action, Direction, Interaction, Item, ItemBox, Need, Priority};

use crate::{FOOD_SPAWN_RATE, FluidSim};

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::f32::consts::PI;
use std::fmt;

use super::world_gen::WorldGenerator;

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
        ItemStack { items: vec![] }
    }

    pub fn with_base(base: Item) -> Self {
        ItemStack {
            items: vec![ItemBox::new(base)],
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
    pub fn replace_top(&mut self, item: ItemBox) {
        self.items.pop();
        self.items.push(item);
    }

    /// Get all items in the stack
    pub fn items(&self) -> &[ItemBox] {
        &self.items
    }

    /// Find the first item that can act in this stack
    pub fn find_actor(&mut self) -> Option<&mut ItemBox> {
        self.items.iter_mut().find(|item_box| item_box.can_act())
    }

    pub fn visible_item(&self) -> Option<(usize, &ItemBox)> {
        self.items.last().map(|item| (self.items.len() - 1, item))
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
    pub(crate) grid: Vec<Vec<ItemStack>>,
    pub(crate) elevation_grid: Vec<Vec<u8>>,
    seed: u64,
    time: u64,
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
            elevation_grid: vec![vec![0; width]; height],
            seed,
            time: 0,
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
    pub fn generate(&mut self, fluid_sim: &mut FluidSim) {
        let mut generator = WorldGenerator;

        let mut elevation_grid = vec![vec![0; self.width]; self.height];
        fluid_sim.to_elevation(&mut elevation_grid);
        let grid = generator.generate(&elevation_grid, self.width, self.height);
        self.grid = grid;
        self.elevation_grid = elevation_grid;
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

    /// Get immediate surroundings (3x3 grid) around a position
    pub fn get_surroundings(&self, x: usize, y: usize) -> Vec<Vec<Option<(usize, &ItemBox)>>> {
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
        target_coord: Option<(usize, usize)>,
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
            if let Some(target) = target_coord {
                // Check if we've reached the specific target coordinate
                if (current_x, current_y) == target {
                    // Reconstruct and return the path
                    let mut path = vec![current.position];
                    let mut current_pos = current.position;

                    while let Some(prev) = came_from.get(&current_pos) {
                        path.push(*prev);
                        current_pos = *prev;
                    }

                    path.reverse();
                    return Some(path);
                }
            } else if let Some(stack) = self.get(current_x, current_y) {
                if let Some((_, item_box)) = stack.visible_item() {
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
                let is_target_item = if let Some(target) = target_coord {
                    // If we have a specific target, check if this is it
                    (nx, ny) == target
                } else {
                    // Otherwise use the target predicate
                    self.get(nx, ny)
                        .and_then(|stack| stack.visible_item())
                        .is_none_or(|(_, box_item)| is_target(box_item))
                };

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

                    // Calculate f_score based on g_score and heuristic
                    // If we have a specific target, use Manhattan distance as heuristic
                    // Otherwise, use g_score only for radial search
                    let f_score = if let Some(target) = target_coord {
                        // A* with Manhattan distance heuristic
                        let h_score = (nx.abs_diff(target.0) + ny.abs_diff(target.1)) as usize;
                        tentative_g_score + h_score
                    } else {
                        // Radial search with no heuristic
                        tentative_g_score
                    };

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
            if let Some((_, item_box)) = stack.visible_item() {
                match &item_box.item {
                    // Road is best for travel
                    Item::Road { .. } => 1,
                    Item::Bridge => 1,

                    // Land is preferred for walking
                    Item::Dirt => 2,
                    Item::Grass => 3,
                    Item::Forest => 7,
                    Item::Sand => 10,

                    // Obstacles are much harder to traverse
                    Item::Log => 13,
                    Item::Rock => 15,
                    Item::Mountain => 15,

                    // Water is difficult - travelers prefer to avoid it
                    Item::Water => 13,
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
        self.time += 1;
        if self.time > 100 {
            self.time = 0;
        }
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

        for (x, y) in actor_positions {
            let stack = self.get_mut(x, y).cloned();
            let surroundings = self.get_surroundings(x, y);

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
                                        s.visible_item().map(|(_, item)| &item.item)
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
                if stack.items().iter().any(|item| {
                    matches!(item.item, Item::Grass | Item::Forest)
                        && !matches!(item.item, Item::Object(Object::Food(_)))
                }) {
                    // Check if there's already food nearby (in a 5x5 area)
                    let mut food_nearby = false;
                    for check_y in y.saturating_sub(1)..=y.saturating_add(1) {
                        for check_x in x.saturating_sub(1)..=x.saturating_add(1) {
                            if check_x < self.width && check_y < self.height {
                                if let Some(nearby_stack) = self.get(check_x, check_y) {
                                    if nearby_stack.items().iter().any(|item| {
                                        matches!(item.item, Item::Object(Object::Food(_)))
                                    }) {
                                        food_nearby = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if food_nearby {
                            break;
                        }
                    }

                    // Only spawn food if there's no food nearby
                    if !food_nearby {
                        let food_type = match fastrand::usize(0..3) {
                            0 => FoodType::Bread,
                            1 => FoodType::Fruit,
                            _ => FoodType::Vegetable,
                        };

                        let food_item = Item::Object(Object::Food(food_type));

                        // Add the food to the world
                        self.add_item(x, y, food_item);
                        // println!("Spawned new food item at ({}, {})", x, y);
                        return;
                    }
                }
            }
        }
    }
    /// Decide what action an item should take
    fn decide_action(
        &self,
        item_box: &mut ItemBox,
        effects: &[Effect],
        surroundings: &[Vec<Option<(usize, &ItemBox)>>],
        x: usize,
        y: usize,
    ) -> Action {
        let pathfind_limit = 80;
        match item_box.item.clone() {
            Item::Traveler { name } => {
                let primary_need = item_box.primary_need();
                let mut best_action = None;

                if primary_need.0 > Priority::Low {
                    for (y_idx, row) in surroundings.iter().enumerate() {
                        for (x_idx, cell) in row.iter().enumerate() {
                            if x_idx == 1 && y_idx == 1 {
                                continue;
                            } // Skip center (where NPC is)

                            // Convert to relative coordinates (-1, 0, 1)
                            let rx = x_idx as isize - 1;
                            let ry = y_idx as isize - 1;

                            if let Some((idx, target_item)) = cell {
                                best_action = match (&primary_need.1, &target_item.item) {
                                    (Need::Water, Item::Water | Item::DeepWater) => {
                                        item_box.think("I'm going to drink this".to_string());
                                        Some(Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Consume,
                                        ))
                                    }
                                    (Need::Food, Item::Object(Object::Food(_))) => {
                                        item_box.think("I'm going to eat this".to_string());
                                        Some(Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::Consume,
                                        ))
                                    }
                                    (Need::Items, Item::Object(_)) => {
                                        item_box
                                            .think("I'm going to pick this thing up".to_string());
                                        Some(Action::Interact(
                                            Some((rx, ry, *idx)),
                                            None,
                                            Interaction::PickUp,
                                        ))
                                    }
                                    (Need::Social, Item::Traveler { name }) => {
                                        if !target_item.young() {
                                            item_box.think(format!("I want to mate with {}", name));
                                            Some(Action::Interact(
                                                Some((rx, ry, *idx)),
                                                None,
                                                Interaction::Mate,
                                            ))
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                }
                                .or(best_action);
                            }
                        }
                    }
                }

                // If we found a suitable action, return it
                if let Some(action) = best_action {
                    item_box.forget_path();
                    return action;
                }

                if let Some(path) = item_box.planned_path_mut() {
                    if !path.is_empty() {
                        if let Some(dir) = path.iter().find_map(|step| {
                            let dir = self.get_direction_to((x, y), *step);
                            if dir != Direction::None {
                                Some(dir)
                            } else {
                                None
                            }
                        }) {
                            path.remove(0);
                            return Action::Move(dir);
                        }
                    } else {
                        item_box.forget_path();
                    }
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

                if primary_need.0 >= Priority::Urgent {
                    match primary_need.1 {
                        Need::Food => {
                            if let Some(path) = self.find_path(
                                (x, y),
                                |item_box| matches!(item_box.item, Item::Object(Object::Food(_))),
                                pathfind_limit,
                                None,
                            ) {
                                item_box.think("I know where to go to find food".to_string());
                                item_box.forget_path();
                                item_box.plan_path(path);
                                return Action::Wait;
                            }
                        }
                        Need::Water => {
                            if let Some(path) = self.find_path(
                                (x, y),
                                |item_box| matches!(item_box.item, Item::Water | Item::DeepWater),
                                pathfind_limit,
                                None,
                            ) {
                                item_box.think("I know where to go to find water".to_string());
                                item_box.forget_path();
                                item_box.plan_path(path);
                                return Action::Wait;
                            }
                        }
                        Need::Social => {
                            if let Some(path) = self.find_path(
                                (x, y),
                                |item_box| matches!(item_box.item, Item::Traveler { .. }),
                                pathfind_limit,
                                None,
                            ) {
                                item_box.think("I know where to go to find a friend".to_string());
                                item_box.forget_path();
                                item_box.plan_path(path);
                                return Action::Wait;
                            }
                        }
                        _ => {}
                    }
                } else if fastrand::f32() > 0.95
                    && self
                        .find_path(
                            (x, y),
                            |item_box| matches!(item_box.item, Item::Water | Item::DeepWater),
                            10,
                            None,
                        )
                        .is_some()
                {
                    item_box.effects.push(Effect::permanent(
                        EffectType::Profession(Profession::Farmer(Farm { location: (x, y) })),
                        100,
                    ));
                    return Action::Wait;
                }

                Action::Wait
            }
            _ => Action::Wait,
        }
    }

    /// Move an item from one position to another
    // Apply effects over time to all items in the world
    fn apply_effects(&mut self) {
        let time = self.time;
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(stack) = self.get_mut(x, y) {
                    for npc in stack.items.iter_mut() {
                        npc.effects.retain_mut(|effect| effect.update());
                        match npc.item {
                            Item::Traveler { ref name } => {
                                if npc.health() == 0 {
                                    npc.die(name.clone());
                                    return;
                                };
                                if npc.hunger() > 100 {
                                    npc.heal(-1);
                                }
                                if npc.thirst() > 100 {
                                    npc.heal(-1);
                                }
                                if time % 2 == 0 {
                                    npc.eat(-1);
                                    npc.drink(-1);
                                }
                                if time % 20 == 0 && npc.loneliness() < 45 {
                                    npc.feel_lonely(1);
                                }
                            }
                            Item::Grass => {
                                if npc.wear() >= 53 {
                                    *npc = ItemBox::with_effects(
                                        Item::Dirt,
                                        Effect::default_terrain_effects(),
                                    )
                                }
                                if time % 50 == 0 && npc.wear() > 47 {
                                    npc.wear_down(-1);
                                }
                            }
                            Item::Snow | Item::Mountain | Item::Road { .. } => {
                                if npc.wear() >= 53 {
                                    *npc = ItemBox::with_effects(
                                        Item::Dirt,
                                        Effect::default_terrain_effects(),
                                    )
                                }
                                if time % 50 == 0 && npc.wear() > 47 {
                                    npc.wear_down(-1);
                                }
                            }
                            Item::Forest => {
                                if npc.wear() >= 53 {
                                    *npc = ItemBox::with_effects(
                                        Item::Grass,
                                        Effect::default_terrain_effects(),
                                    );
                                }
                                if time % 50 == 0 && npc.wear() > 47 {
                                    npc.wear_down(-1);
                                }
                            }
                            Item::Dirt => {
                                if npc.wear() <= 47 {
                                    *npc = ItemBox::with_effects(
                                        Item::Grass,
                                        Effect::default_terrain_effects(),
                                    )
                                }
                                if time % 50 == 0 && npc.wear() > 47 {
                                    npc.wear_down(-1);
                                }
                            }
                            Item::Water
                            | Item::DeepWater
                            | Item::Sand
                            | Item::Log
                            | Item::Rock
                            | Item::Corpse { .. }
                            | Item::House { .. }
                            | Item::Shop { .. }
                            | Item::Tavern { .. }
                            | Item::Temple { .. }
                            | Item::Object(_)
                            | Item::Bridge => (),
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
            .map(|i| !i.1.is_traversable())
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

                self.grid[new_y][new_x].top_mut().wear_down(5);
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
                    stack.visible_item().map(|i| i.1.get_char()).unwrap_or(' ')
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
