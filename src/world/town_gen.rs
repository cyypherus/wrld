// use rand::Rng;
// use rand::SeedableRng;
// use rand::seq::IndexedRandom;
// use rand::seq::SliceRandom;
// use rand_chacha::ChaCha8Rng;
// use std::collections::{HashSet, VecDeque};

// use crate::entity::item::{Item, ItemBox};
// use crate::world::world::ItemStack;

// /// Represents a settlement size in the world
// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub enum SettlementSize {
//     Hamlet,  // 3-5 buildings
//     Village, // 6-10 buildings
//     Town,    // 11-18 buildings
//     City,    // 19+ buildings
// }

// impl SettlementSize {
//     /// Get the number of buildings for this settlement size
//     pub fn building_count(&self) -> (usize, usize) {
//         match self {
//             SettlementSize::Hamlet => (3, 5),
//             SettlementSize::Village => (6, 10),
//             SettlementSize::Town => (11, 18),
//             SettlementSize::City => (19, 30),
//         }
//     }

//     /// Get the radius of influence for this settlement size
//     pub fn radius(&self) -> usize {
//         match self {
//             SettlementSize::Hamlet => 3,
//             SettlementSize::Village => 4,
//             SettlementSize::Town => 6,
//             SettlementSize::City => 8,
//         }
//     }
// }

// /// Generates towns and road networks
// pub struct TownGenerator {
//     rng: ChaCha8Rng,
//     town_names: Vec<String>,
//     shop_types: Vec<String>,
//     tavern_name_prefixes: Vec<String>,
//     tavern_name_suffixes: Vec<String>,
//     deities: Vec<String>,
// }

// impl TownGenerator {
//     /// Create a new town generator with the given seed
//     pub fn new(seed: u64) -> Self {
//         let mut rng = ChaCha8Rng::seed_from_u64(seed);

//         // Town name components
//         let prefixes = vec![
//             "North", "South", "East", "West", "New", "Old", "Upper", "Lower", "Great", "Little",
//             "High", "Low", "Far", "Mid", "Fort", "Port",
//         ];

//         let roots = vec![
//             "wood", "field", "lake", "river", "bridge", "ford", "haven", "hill", "vale", "dale",
//             "shire", "town", "bury", "borough", "port", "wick", "caster", "chester", "mouth",
//             "stead", "wick", "ton", "ham", "by",
//         ];

//         // Generate random town names from combinations
//         let mut town_names = Vec::new();
//         for _ in 0..50 {
//             if rng.random_bool(0.6) {
//                 // Use prefix + root
//                 let prefix = prefixes.choose(&mut rng).unwrap();
//                 let root = roots.choose(&mut rng).unwrap();
//                 town_names.push(format!("{}{}", prefix, root));
//             } else {
//                 // Use standalone name
//                 let root = roots.choose(&mut rng).unwrap();
//                 let suffix = match rng.random_range(0..4) {
//                     0 => "ton",
//                     1 => "ville",
//                     2 => "ford",
//                     _ => "berg",
//                 };
//                 town_names.push(format!("{}{}", root, suffix));
//             }
//         }

//         // Shop types
//         let shop_types = [
//             "Blacksmith",
//             "Tailor",
//             "Baker",
//             "Butcher",
//             "Alchemist",
//             "Herbalist",
//             "Carpenter",
//             "Jeweler",
//             "Armorer",
//             "Fletcher",
//             "General",
//             "Pottery",
//         ];

//         // Tavern name components
//         let tavern_name_prefixes = vec![
//             "The",
//             "Ye Olde",
//             "Golden",
//             "Silver",
//             "Copper",
//             "Rusty",
//             "Broken",
//             "Jolly",
//             "Merry",
//             "Dancing",
//             "Laughing",
//             "Prancing",
//             "Wandering",
//         ];

//         let tavern_name_suffixes = vec![
//             "Dragon", "Wyvern", "Phoenix", "Lion", "Eagle", "Wolf", "Bear", "Stag", "Hound",
//             "Goose", "Swan", "Hammer", "Sword", "Shield", "Tankard", "Goblet", "Barrel", "Mug",
//             "Flagon", "Lantern", "Hearth",
//         ];

//         // Deity names
//         let deities = [
//             "Solaris", "Lunari", "Terranus", "Aquarius", "Ventus", "Ignis", "Fortuna", "Chronos",
//             "Astra", "Vitalis", "Mortis", "Bellum",
//         ];

//         TownGenerator {
//             rng,
//             town_names,
//             shop_types: shop_types.iter().map(|s| s.to_string()).collect(),
//             tavern_name_prefixes: tavern_name_prefixes.iter().map(|s| s.to_string()).collect(),
//             tavern_name_suffixes: tavern_name_suffixes.iter().map(|s| s.to_string()).collect(),
//             deities: deities.iter().map(|s| s.to_string()).collect(),
//         }
//     }

//     /// Generate towns and road network for a world
//     pub fn generate(
//         &mut self,
//         grid: &mut [Vec<ItemStack>],
//         width: usize,
//         height: usize,
//         min_towns: usize,
//         max_towns: usize,
//     ) {
//         // Find suitable locations for towns based on terrain
//         let town_locations = self.find_town_locations(grid, width, height, min_towns, max_towns);

//         // Generate town centers and associated information
//         let town_centers = self.create_settlement_centers(town_locations);

//         // Build towns at the selected locations
//         for (idx, &(x, y)) in town_centers.iter().enumerate() {
//             // Create the town at this location
//             self.build_settlement(grid, x, y, idx, town_centers.len());
//         }

//         // Create road network connecting towns
//         self.create_road_network(grid, &town_centers, width, height);
//     }

//     /// Create town center information based on locations
//     fn create_settlement_centers(&mut self, locations: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
//         locations
//     }

//     /// Find suitable locations for towns
//     fn find_town_locations(
//         &mut self,
//         grid: &[Vec<ItemStack>],
//         width: usize,
//         height: usize,
//         min_towns: usize,
//         max_towns: usize,
//     ) -> Vec<(usize, usize)> {
//         let mut suitability_map = vec![vec![0.0; width]; height];
//         let mut candidate_locations = Vec::new();

//         // Calculate suitability scores for each location
//         for y in 3..height - 3 {
//             for x in 3..width - 3 {
//                 // Skip water and mountains
//                 let mut has_water = false;
//                 let mut has_mountain = false;
//                 let stack = &grid[y][x];

//                 for item in stack.items() {
//                     match item.item {
//                         Item::Water | Item::DeepWater => has_water = true,
//                         Item::Mountain | Item::Rock => has_mountain = true,
//                         _ => {}
//                     }
//                 }

//                 if has_water || has_mountain {
//                     continue;
//                 }

//                 // Check surrounding area for good conditions (flat land, some water nearby but not too much)
//                 let mut flat_land_count = 0;
//                 let mut water_nearby = false;
//                 let mut mountain_nearby = false;

//                 let search_radius = 5;
//                 for dy in -search_radius..=search_radius {
//                     for dx in -search_radius..=search_radius {
//                         let nx = x as isize + dx;
//                         let ny = y as isize + dy;

//                         if nx < 0 || nx >= width as isize || ny < 0 || ny >= height as isize {
//                             continue;
//                         }

//                         let nx = nx as usize;
//                         let ny = ny as usize;

//                         for item in grid[ny][nx].items() {
//                             match item.item {
//                                 Item::Water | Item::DeepWater => {
//                                     // Water within 2-5 tiles is good
//                                     let dist = dx.abs() + dy.abs();
//                                     if dist <= search_radius && dist >= 2 {
//                                         water_nearby = true;
//                                     }
//                                 }
//                                 Item::Mountain => {
//                                     // Mountains within 3-5 tiles is good
//                                     let dist = dx.abs() + dy.abs();
//                                     if dist <= search_radius && dist >= 3 {
//                                         mountain_nearby = true;
//                                     }
//                                 }
//                                 Item::Grass | Item::Dirt | Item::Forest => {
//                                     // Count flat land for building
//                                     if dx.abs() <= 3 && dy.abs() <= 3 {
//                                         flat_land_count += 1;
//                                     }
//                                 }
//                                 _ => {}
//                             }
//                         }
//                     }
//                 }

//                 // Calculate suitability score
//                 let mut score = 0.0;

//                 // Favor locations with flat land for building
//                 score += (flat_land_count as f64) * 0.2;

//                 // Bonus for water access
//                 if water_nearby {
//                     score += 20.0;
//                 }

//                 // Bonus for scenic mountain views
//                 if mountain_nearby {
//                     score += 15.0;
//                 }

//                 // Slightly randomize to break ties
//                 score += self.rng.random_range(-5.0..5.0);

//                 suitability_map[y][x] = score;

//                 // Only consider locations with decent scores
//                 if score > 30.0 {
//                     candidate_locations.push((x, y, score));
//                 }
//             }
//         }

//         // Sort by decreasing suitability
//         candidate_locations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

//         // Select the best locations, ensuring minimum distance between towns
//         let mut selected_locations = Vec::new();
//         let min_distance = 15; // Minimum distance between town centers

//         for &(x, y, _) in &candidate_locations {
//             // Check if this location is too close to already selected towns
//             let mut too_close = false;
//             for &(sx, sy) in &selected_locations {
//                 let distance =
//                     ((x as isize - sx as isize).pow(2) + (y as isize - sy as isize).pow(2)) as f64;
//                 if distance < (min_distance as f64).powi(2) {
//                     too_close = true;
//                     break;
//                 }
//             }

//             if !too_close {
//                 selected_locations.push((x, y));

//                 // Stop once we have enough towns
//                 if selected_locations.len() >= max_towns {
//                     break;
//                 }
//             }
//         }

//         // If we don't have enough towns, lower our standards
//         if selected_locations.len() < min_towns {
//             for &(x, y, _) in &candidate_locations {
//                 if !selected_locations.contains(&(x, y)) {
//                     selected_locations.push((x, y));

//                     if selected_locations.len() >= min_towns {
//                         break;
//                     }
//                 }
//             }
//         }

//         selected_locations
//     }

//     /// Build a settlement at the specified location
//     fn build_settlement(
//         &mut self,
//         grid: &mut [Vec<ItemStack>],
//         x: usize,
//         y: usize,
//         index: usize,
//         _total_towns: usize,
//     ) {
//         // Determine settlement size - first few locations are larger
//         let size = if index == 0 {
//             SettlementSize::City
//         } else if index <= 1 {
//             SettlementSize::Town
//         } else if index <= 3 {
//             SettlementSize::Village
//         } else {
//             SettlementSize::Hamlet
//         };

//         // Generate town name
//         let town_name = if !self.town_names.is_empty() {
//             let idx = self.rng.random_range(0..self.town_names.len());
//             self.town_names.remove(idx)
//         } else {
//             format!("Town {}", index + 1)
//         };

//         let radius = size.radius();

//         // Determine number of buildings based on town size
//         let (min_buildings, max_buildings) = size.building_count();
//         let num_buildings = self.rng.random_range(min_buildings..=max_buildings);

//         // Create a town center with guaranteed buildings
//         let mut used_positions = HashSet::new();
//         used_positions.insert((x, y));

//         // Place a town square/central road at the town center
//         if let Some(stack) = &mut grid.get_mut(y).and_then(|row| row.get_mut(x)) {
//             // Clear existing items except the base
//             while stack.items().len() > 1 {
//                 stack.pop();
//             }

//             // Add road as town center
//             stack.push(ItemBox::new(Item::Road { connected: true }));
//         }

//         // Track all building positions for later road connection
//         let mut building_positions = Vec::new();

//         // First, place key buildings: temple for larger towns, tavern for all
//         if size == SettlementSize::City || size == SettlementSize::Town {
//             // Place a temple near the center
//             let positions = self.find_building_positions(grid, x, y, 2, 1, &used_positions);

//             if let Some(pos) = positions.first() {
//                 let deity = self
//                     .deities
//                     .choose(&mut self.rng)
//                     .cloned()
//                     .unwrap_or_else(|| "Unknown".to_string());

//                 let temple = Item::Temple {
//                     deity: deity.clone(),
//                 };
//                 self.place_building(grid, pos.0, pos.1, temple);
//                 building_positions.push(*pos);
//                 used_positions.insert(*pos);
//             }
//         }

//         // Place a tavern in every town (with the town's name)
//         let positions = self.find_building_positions(grid, x, y, 2, 1, &used_positions);

//         if let Some(pos) = positions.first() {
//             let prefix = self
//                 .tavern_name_prefixes
//                 .choose(&mut self.rng)
//                 .cloned()
//                 .unwrap_or_else(|| "The".to_string());
//             let suffix = self
//                 .tavern_name_suffixes
//                 .choose(&mut self.rng)
//                 .cloned()
//                 .unwrap_or_else(|| "Inn".to_string());

//             // Use the tavern name to indicate the town name
//             let tavern_name = format!("{} {} ({})", prefix, suffix, town_name);
//             let tavern = Item::Tavern { name: tavern_name };
//             self.place_building(grid, pos.0, pos.1, tavern);
//             building_positions.push(*pos);
//             used_positions.insert(*pos);
//         }

//         // Place shops according to town size
//         let shop_count = match size {
//             SettlementSize::Hamlet => 0,
//             SettlementSize::Village => 1,
//             SettlementSize::Town => 2,
//             SettlementSize::City => 3,
//         };

//         for _ in 0..shop_count {
//             let positions = self.find_building_positions(grid, x, y, radius, 2, &used_positions);

//             if let Some(pos) = positions.first() {
//                 let shop_type = self
//                     .shop_types
//                     .choose(&mut self.rng)
//                     .cloned()
//                     .unwrap_or_else(|| "General".to_string());

//                 let shop = Item::Shop {
//                     shop_type: shop_type.clone(),
//                 };
//                 self.place_building(grid, pos.0, pos.1, shop);
//                 building_positions.push(*pos);
//                 used_positions.insert(*pos);
//             }
//         }

//         // Fill the remaining slots with houses
//         let remaining = num_buildings - building_positions.len();

//         for _ in 0..remaining {
//             let positions = self.find_building_positions(grid, x, y, radius, 3, &used_positions);

//             if let Some(pos) = positions.first() {
//                 // 30% chance of having a named owner
//                 let owner = if self.rng.random_bool(0.3) {
//                     Some(self.generate_random_name())
//                 } else {
//                     None
//                 };

//                 let house = Item::House {
//                     owner: owner.clone(),
//                 };
//                 self.place_building(grid, pos.0, pos.1, house);
//                 building_positions.push(*pos);
//                 used_positions.insert(*pos);
//             }
//         }

//         // Add roads between buildings within the town
//         self.create_town_roads(grid, x, y, &building_positions);
//     }

//     /// Find suitable positions for placing buildings
//     fn find_building_positions(
//         &mut self,
//         grid: &[Vec<ItemStack>],
//         center_x: usize,
//         center_y: usize,
//         max_radius: usize,
//         count: usize,
//         used_positions: &HashSet<(usize, usize)>,
//     ) -> Vec<(usize, usize)> {
//         let width = grid[0].len();
//         let height = grid.len();
//         let mut candidates = Vec::new();

//         for r in 1..=max_radius {
//             let r_isize = r as isize;
//             for dx in -r_isize..=r_isize {
//                 for dy in -r_isize..=r_isize {
//                     // Only consider positions on the edge of each radius
//                     if dx.abs() != r_isize && dy.abs() != r_isize {
//                         continue;
//                     }

//                     let x = center_x as isize + dx;
//                     let y = center_y as isize + dy;

//                     if x < 0 || x >= width as isize || y < 0 || y >= height as isize {
//                         continue;
//                     }

//                     let x = x as usize;
//                     let y = y as usize;

//                     // Skip if position is already used
//                     if used_positions.contains(&(x, y)) {
//                         continue;
//                     }

//                     // Check if this position is suitable for a building
//                     if self.is_suitable_for_building(grid, x, y) {
//                         candidates.push((x, y));
//                     }
//                 }
//             }
//         }

//         // Shuffle candidates and take the required number
//         candidates.shuffle(&mut self.rng);
//         candidates.truncate(count);

//         candidates
//     }

//     /// Check if a position is suitable for placing a building
//     fn is_suitable_for_building(&self, grid: &[Vec<ItemStack>], x: usize, y: usize) -> bool {
//         let width = grid[0].len();
//         let height = grid.len();

//         // Check if out of bounds
//         if x >= width || y >= height {
//             return false;
//         }

//         // Check the item stack at this position
//         let stack = &grid[y][x];

//         // Must have a solid foundation (dirt or sand)
//         let mut has_foundation = false;

//         // Must not have water, rock, mountain, or existing structures
//         let mut has_obstacle = false;

//         for item in stack.items() {
//             match item.item {
//                 Item::Dirt => has_foundation = true,
//                 Item::Sand => has_foundation = true,
//                 Item::Water | Item::DeepWater => has_obstacle = true,
//                 Item::Rock | Item::Mountain => has_obstacle = true,
//                 Item::House { .. }
//                 | Item::Shop { .. }
//                 | Item::Tavern { .. }
//                 | Item::Temple { .. } => has_obstacle = true,
//                 _ => {}
//             }
//         }

//         has_foundation && !has_obstacle
//     }

//     /// Place a building at the specified location
//     fn place_building(&self, grid: &mut [Vec<ItemStack>], x: usize, y: usize, building: Item) {
//         if let Some(stack) = &mut grid.get_mut(y).and_then(|row| row.get_mut(x)) {
//             // Remove items except the base (dirt or sand)
//             while stack.items().len() > 1 {
//                 stack.pop();
//             }

//             // Add the building and air on top
//             stack.push(ItemBox::new(building));
//         }
//     }

//     /// Create roads within a town connecting buildings
//     fn create_town_roads(
//         &mut self,
//         grid: &mut [Vec<ItemStack>],
//         center_x: usize,
//         center_y: usize,
//         building_positions: &[(usize, usize)],
//     ) {
//         let width = grid[0].len();
//         let height = grid.len();

//         // Create roads from town center to each building
//         for &(bx, by) in building_positions {
//             self.create_road_path(grid, (center_x, center_y), (bx, by), width, height, true);
//         }
//     }

//     /// Create a road network connecting towns
//     fn create_road_network(
//         &mut self,
//         grid: &mut [Vec<ItemStack>],
//         town_centers: &[(usize, usize)],
//         width: usize,
//         height: usize,
//     ) {
//         if town_centers.len() <= 1 {
//             return;
//         }

//         // Create a minimum spanning tree to connect all towns efficiently
//         let mut connected = HashSet::new();
//         let mut edges = Vec::new();

//         // Start with the largest town
//         connected.insert(0);

//         // Keep connecting towns until all are connected
//         while connected.len() < town_centers.len() {
//             let mut best_edge = None;
//             let mut best_distance = f64::MAX;

//             // Calculate the closest unconnected town to any connected town
//             for &connected_idx in &connected {
//                 let (cx, cy) = town_centers[connected_idx];

//                 for (i, center) in town_centers.iter().enumerate() {
//                     if connected.contains(&i) {
//                         continue;
//                     }

//                     let (tx, ty) = *center;
//                     let distance = ((cx as isize - tx as isize).pow(2)
//                         + (cy as isize - ty as isize).pow(2))
//                         as f64;

//                     if distance < best_distance {
//                         best_distance = distance;
//                         best_edge = Some((connected_idx, i));
//                     }
//                 }
//             }

//             if let Some((from, to)) = best_edge {
//                 edges.push((from, to));
//                 connected.insert(to);
//             } else {
//                 break; // Should not happen
//             }
//         }

//         // Add some additional roads for redundancy and realism
//         // Connect some towns that are close to each other but not already connected
//         for i in 0..town_centers.len() {
//             for j in i + 1..town_centers.len() {
//                 // Skip if already connected directly
//                 if edges.contains(&(i, j)) || edges.contains(&(j, i)) {
//                     continue;
//                 }

//                 let (x1, y1) = town_centers[i];
//                 let (x2, y2) = town_centers[j];

//                 let distance = ((x1 as isize - x2 as isize).pow(2)
//                     + (y1 as isize - y2 as isize).pow(2)) as f64;

//                 // Connect close towns with some probability
//                 if distance < 1000.0 && self.rng.random_bool(0.3) {
//                     edges.push((i, j));
//                 }
//             }
//         }

//         // Create roads for each edge
//         for &(from, to) in &edges {
//             let (x1, y1) = town_centers[from];
//             let (x2, y2) = town_centers[to];

//             self.create_road_path(grid, (x1, y1), (x2, y2), width, height, false);
//         }
//     }

//     /// Create a road path between two points using A* pathfinding
//     fn create_road_path(
//         &mut self,
//         grid: &mut [Vec<ItemStack>],
//         p1: (usize, usize),
//         p2: (usize, usize),
//         width: usize,
//         height: usize,
//         _is_town_road: bool,
//     ) {
//         let (x1, y1) = p1;
//         let (x2, y2) = p2;

//         // A* pathfinding
//         #[derive(Clone, Eq, PartialEq, Debug)]
//         struct Node {
//             x: usize,
//             y: usize,
//             g_score: usize, // Cost from start to this node
//             f_score: usize, // g_score + heuristic
//         }

//         impl std::cmp::Ord for Node {
//             fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//                 // Reverse order so priority queue becomes min-heap
//                 other.f_score.cmp(&self.f_score)
//             }
//         }

//         impl std::cmp::PartialOrd for Node {
//             fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//                 Some(self.cmp(other))
//             }
//         }

//         // Heuristic function (Manhattan distance)
//         let h = |x: usize, y: usize| -> usize {
//             ((x as isize - x2 as isize).abs() + (y as isize - y2 as isize).abs()) as usize
//         };

//         // Cost function - how difficult it is to build a road here
//         let cost = |x: usize, y: usize| -> usize {
//             if x >= width || y >= height {
//                 return usize::MAX;
//             }

//             let stack = &grid[y][x];

//             for item in stack.items() {
//                 match item.item {
//                     Item::Water => return 10, // Higher cost to cross water (will need bridges)
//                     Item::DeepWater => return 20, // Much higher cost for deep water
//                     Item::Mountain => return usize::MAX, // Cannot cross mountains
//                     Item::Rock => return 8,   // Difficult terrain
//                     Item::House { .. }
//                     | Item::Shop { .. }
//                     | Item::Tavern { .. }
//                     | Item::Temple { .. } => {
//                         return usize::MAX; // Cannot build roads through buildings
//                     }
//                     Item::Road { .. } | Item::Bridge => return 1, // Very easy to follow existing roads
//                     _ => {}
//                 }
//             }

//             // Default cost
//             3
//         };

//         // Calculate initial values
//         let start_h = h(x1, y1);
//         let start_node = Node {
//             x: x1,
//             y: y1,
//             g_score: 0,
//             f_score: start_h,
//         };

//         // Priority queue for open set
//         let mut open_set = VecDeque::new();
//         open_set.push_back(start_node);

//         // Track visited nodes and the path
//         let mut came_from = std::collections::HashMap::new();
//         let mut g_scores = std::collections::HashMap::new();
//         g_scores.insert((x1, y1), 0);

//         while !open_set.is_empty() {
//             // Get the node with lowest f_score
//             let current = open_set.pop_front().unwrap();

//             // Check if we've reached the goal
//             if current.x == x2 && current.y == y2 {
//                 // Reconstruct and build the path
//                 let mut path = Vec::new();
//                 let mut current_pos = (x2, y2);

//                 while current_pos != (x1, y1) {
//                     path.push(current_pos);
//                     current_pos = *came_from.get(&current_pos).unwrap();
//                 }
//                 path.push((x1, y1));
//                 path.reverse();

//                 // Build the road along the path
//                 for &(x, y) in &path {
//                     // Check for water to place bridges instead of roads
//                     let mut has_water = false;
//                     if let Some(stack) = &grid.get(y).and_then(|row| row.get(x)) {
//                         for item in stack.items() {
//                             if let Item::Water | Item::DeepWater = item.item {
//                                 has_water = true;
//                                 break;
//                             }
//                         }
//                     }

//                     if let Some(stack) = &mut grid.get_mut(y).and_then(|row| row.get_mut(x)) {
//                         // Skip if there's a building here
//                         let mut has_building = false;
//                         for item in stack.items() {
//                             match item.item {
//                                 Item::House { .. }
//                                 | Item::Shop { .. }
//                                 | Item::Tavern { .. }
//                                 | Item::Temple { .. } => {
//                                     has_building = true;
//                                     break;
//                                 }
//                                 _ => {}
//                             }
//                         }

//                         if has_building {
//                             continue;
//                         }

//                         // Check if already a road or bridge
//                         let mut already_road = false;
//                         for item in stack.items() {
//                             match item.item {
//                                 Item::Road { .. } | Item::Bridge => {
//                                     already_road = true;
//                                     break;
//                                 }
//                                 _ => {}
//                             }
//                         }

//                         if !already_road {
//                             // Remove everything except the base
//                             while stack.items().len() > 1 {
//                                 stack.pop();
//                             }

//                             // Add road or bridge
//                             if has_water {
//                                 stack.push(ItemBox::new(Item::Bridge));
//                             } else {
//                                 stack.push(ItemBox::new(Item::Road { connected: true }));
//                             }
//                         }
//                     }
//                 }

//                 return;
//             }

//             // Check all neighbors
//             for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
//                 let nx = current.x as isize + dx;
//                 let ny = current.y as isize + dy;

//                 if nx < 0 || nx >= width as isize || ny < 0 || ny >= height as isize {
//                     continue;
//                 }

//                 let nx = nx as usize;
//                 let ny = ny as usize;

//                 // Calculate cost to this neighbor
//                 let move_cost = cost(nx, ny);
//                 if move_cost == usize::MAX {
//                     continue; // Impassable
//                 }

//                 let tentative_g = current.g_score + move_cost;
//                 let neighbor_key = (nx, ny);

//                 if !g_scores.contains_key(&neighbor_key)
//                     || tentative_g < *g_scores.get(&neighbor_key).unwrap()
//                 {
//                     // This path to neighbor is better than any previous one
//                     came_from.insert(neighbor_key, (current.x, current.y));
//                     g_scores.insert(neighbor_key, tentative_g);

//                     let f_score = tentative_g + h(nx, ny);

//                     // Add to open set with proper ordering
//                     let neighbor_node = Node {
//                         x: nx,
//                         y: ny,
//                         g_score: tentative_g,
//                         f_score,
//                     };

//                     // Find the proper position to insert (maintaining sort order)
//                     let mut inserted = false;
//                     for i in 0..open_set.len() {
//                         if open_set[i].f_score > f_score {
//                             open_set.insert(i, neighbor_node.clone());
//                             inserted = true;
//                             break;
//                         }
//                     }

//                     if !inserted {
//                         open_set.push_back(neighbor_node.clone());
//                     }
//                 }
//             }
//         }
//     }

//     /// Generate a random name for house owners
//     fn generate_random_name(&mut self) -> String {
//         let first_names = [
//             "John",
//             "Mary",
//             "William",
//             "Emma",
//             "James",
//             "Sarah",
//             "Robert",
//             "Elizabeth",
//             "Thomas",
//             "Margaret",
//             "Charles",
//             "Anna",
//             "Joseph",
//             "Catherine",
//             "Henry",
//             "Jane",
//         ];

//         let last_names = [
//             "Smith", "Johnson", "Williams", "Brown", "Jones", "Miller", "Davis", "Wilson",
//             "Taylor", "Clark", "White", "Harris", "Martin", "Thompson", "Wood", "Lewis",
//         ];

//         let first = first_names[self.rng.random_range(0..first_names.len())];
//         let last = last_names[self.rng.random_range(0..last_names.len())];

//         format!("{} {}", first, last)
//     }
// }
