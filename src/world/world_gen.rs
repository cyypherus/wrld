use crate::entity::item::{Item, ItemBox};
use crate::world::world::ItemStack;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Contains all parameters for world generation
pub struct WorldGenParams {
    /// Seed for the world generation
    pub seed: u64,
    /// Controls the frequency of the noise function for elevation
    pub elevation_scale: f64,
    /// Controls the frequency of the noise function for moisture
    pub moisture_scale: f64,
    /// Controls the frequency of the noise function for vegetation
    pub vegetation_scale: f64,
    /// Sea level threshold (values below this are water)
    pub sea_level: f64,
    /// Beach level threshold (values just above sea level are beaches)
    pub beach_level: f64,
    /// Controls how much the elevation changes
    pub elevation_amplitude: f64,
    /// Controls how many rivers are attempted to be placed
    pub river_count: usize,
    /// Controls how wide rivers are
    pub river_width: f64,
}

impl Default for WorldGenParams {
    fn default() -> Self {
        WorldGenParams {
            seed: 0,
            elevation_scale: 0.08,
            moisture_scale: 0.06,
            vegetation_scale: 0.15,
            sea_level: 0.38,
            beach_level: 0.42,
            elevation_amplitude: 1.2,
            river_count: 4,
            river_width: 1.3,
        }
    }
}

/// A complete world generator that uses multiple noise functions to create realistic terrain
pub struct WorldGenerator {
    params: WorldGenParams,
    elevation_noise: Perlin,
    moisture_noise: Perlin,
    vegetation_noise: Perlin,
    feature_noise: Perlin,
    rng: ChaCha8Rng,
}

impl WorldGenerator {
    /// Create a new world generator with the given seed
    pub fn new(seed: u64) -> Self {
        WorldGenerator {
            params: WorldGenParams {
                seed,
                ..WorldGenParams::default()
            },
            elevation_noise: Perlin::new(seed as u32),
            moisture_noise: Perlin::new((seed.wrapping_add(1)) as u32),
            vegetation_noise: Perlin::new((seed.wrapping_add(2)) as u32),
            feature_noise: Perlin::new((seed.wrapping_add(3)) as u32),
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Create a new world generator with custom parameters
    pub fn with_params(params: WorldGenParams) -> Self {
        let seed = params.seed;
        WorldGenerator {
            params,
            elevation_noise: Perlin::new(seed as u32),
            moisture_noise: Perlin::new((seed.wrapping_add(1)) as u32),
            vegetation_noise: Perlin::new((seed.wrapping_add(2)) as u32),
            feature_noise: Perlin::new((seed.wrapping_add(3)) as u32),
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Generate terrain for a world grid
    pub fn generate(&mut self, width: usize, height: usize) -> Vec<Vec<ItemStack>> {
        let mut grid = vec![vec![ItemStack::new(); width]; height];

        // First pass: generate base terrain (elevation and water)
        self.generate_base_terrain(&mut grid, width, height);

        // Second pass: add rivers
        self.generate_rivers(&mut grid, width, height);

        // Third pass: add vegetation and features based on moisture and elevation
        self.generate_features(&mut grid, width, height);

        grid
    }

    /// Generate base terrain including elevation and water
    fn generate_base_terrain(
        &mut self,
        grid: &mut Vec<Vec<ItemStack>>,
        width: usize,
        height: usize,
    ) {
        // Calculate the center of the map for island-centric generation
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let max_distance = (center_x.powi(2) + center_y.powi(2)).sqrt();

        for y in 0..height {
            for x in 0..width {
                // Calculate distance from center (normalized 0-1)
                let dx = x as f64 - center_x;
                let dy = y as f64 - center_y;
                let distance = (dx.powi(2) + dy.powi(2)).sqrt() / max_distance;

                // Generate various noise values
                let nx = x as f64 * self.params.elevation_scale;
                let ny = y as f64 * self.params.elevation_scale;

                // Create base elevation using Perlin noise
                let mut elevation = self.elevation_noise.get([nx, ny]) * 0.5 + 0.5;

                // Add a second octave of noise for more varied terrain
                let detail_elevation = self.elevation_noise.get([nx * 2.0, ny * 2.0]) * 0.25 + 0.25;
                elevation = (elevation + detail_elevation) / 1.25;

                // Apply island falloff to make land centered with water around edges
                let falloff = (1.0 - distance.powf(2.2)).max(0.0);
                elevation = (elevation * falloff * self.params.elevation_amplitude).min(1.0);

                // Generate moisture level
                let moisture = self.moisture_noise.get([
                    x as f64 * self.params.moisture_scale,
                    y as f64 * self.params.moisture_scale,
                ]) * 0.5
                    + 0.5;

                // Determine the base item based on elevation
                let base_item = if elevation < self.params.sea_level - 0.15 {
                    // Deep water areas
                    Item::Sand // Seabed
                } else if elevation < self.params.sea_level {
                    // Shallow water areas
                    Item::Sand // Seabed
                } else if elevation < self.params.beach_level {
                    // Beach areas
                    Item::Sand
                } else {
                    // Land areas
                    Item::Dirt
                };

                // Create the stack with the base item
                let mut stack = ItemStack::with_base(base_item);

                // Add the top item based on elevation and moisture
                if elevation < self.params.sea_level - 0.15 {
                    // Deep water areas
                    stack.push(ItemBox::new(Item::DeepWater));
                } else if elevation < self.params.sea_level {
                    // Shallow water areas
                    stack.push(ItemBox::new(Item::Water));
                } else if elevation < self.params.beach_level {
                    // Beach areas, occasionally add rocks
                    if self.feature_noise.get([nx * 3.0, ny * 3.0]) > 0.85 {
                        stack.push(ItemBox::new(Item::Rock));
                    }
                } else if elevation > 0.85 {
                    // Mountain peaks with snow
                    stack.push(ItemBox::new(Item::Mountain));
                    stack.push(ItemBox::new(Item::Snow));
                } else if elevation > 0.75 {
                    // Mountain areas
                    stack.push(ItemBox::new(Item::Mountain));
                } else if elevation > 0.65 {
                    // Rocky elevated areas
                    stack.push(ItemBox::new(Item::Rock));
                } else {
                    // Regular land, add vegetation based on moisture
                    if moisture > 0.7 {
                        // Very moist areas get grass and occasional logs (forest)
                        stack.push(ItemBox::new(Item::Grass));
                        if self.feature_noise.get([nx * 5.0, ny * 5.0]) > 0.8 {
                            stack.push(ItemBox::new(Item::Log));
                        }
                    } else if moisture > 0.4 {
                        // Moderately moist areas get grass
                        stack.push(ItemBox::new(Item::Grass));
                    } else if moisture > 0.3 {
                        // Dry areas sometimes get grass
                        if self.feature_noise.get([nx * 2.0, ny * 2.0]) > 0.5 {
                            stack.push(ItemBox::new(Item::Grass));
                        }
                    }
                    // Arid areas are just dirt
                }

                // Always top with air
                stack.push(ItemBox::new(Item::Air));

                // Set the stack in the grid
                grid[y][x] = stack;
            }
        }
    }

    /// Generate rivers using path finding from mountains to the sea
    fn generate_rivers(&mut self, grid: &mut Vec<Vec<ItemStack>>, width: usize, height: usize) {
        // Find the highest points (potential river sources)
        let mut potential_sources = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * self.params.elevation_scale;
                let ny = y as f64 * self.params.elevation_scale;
                let elevation = self.elevation_noise.get([nx, ny]) * 0.5 + 0.5;

                if elevation > 0.75 {
                    potential_sources.push((x, y, elevation));
                }
            }
        }

        // Sort by elevation descending
        potential_sources.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Limit to the desired number of rivers
        let river_count = self.params.river_count.min(potential_sources.len());

        for i in 0..river_count {
            if let Some(&(x, y, _)) = potential_sources.get(i) {
                self.create_river(grid, width, height, x, y);
            }
        }
    }

    /// Create a river starting from a source point, flowing downhill to the sea
    fn create_river(
        &mut self,
        grid: &mut Vec<Vec<ItemStack>>,
        width: usize,
        height: usize,
        start_x: usize,
        start_y: usize,
    ) {
        let mut river_points = Vec::new();
        let mut current_x = start_x;
        let mut current_y = start_y;
        let mut reached_water = false;

        // Follow the steepest downhill path until reaching water or the edge
        while !reached_water
            && current_x > 0
            && current_x < width - 1
            && current_y > 0
            && current_y < height - 1
        {
            river_points.push((current_x, current_y));

            // Look at all neighbors to find the lowest elevation
            let mut lowest_elevation = 2.0; // Higher than possible noise value
            let mut next_x = current_x;
            let mut next_y = current_y;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue; // Skip current position
                    }

                    let nx = (current_x as isize + dx) as usize;
                    let ny = (current_y as isize + dy) as usize;

                    if nx < width && ny < height {
                        let stack = &grid[ny][nx];

                        // Check if this is water (destination)
                        for item in stack.items() {
                            if let Item::Water | Item::DeepWater = item.item {
                                reached_water = true;
                                next_x = nx;
                                next_y = ny;
                                break;
                            }
                        }

                        if reached_water {
                            break;
                        }

                        // Otherwise, evaluate elevation for next step
                        let elevation = self.elevation_noise.get([
                            nx as f64 * self.params.elevation_scale,
                            ny as f64 * self.params.elevation_scale,
                        ]) * 0.5
                            + 0.5;

                        if elevation < lowest_elevation {
                            lowest_elevation = elevation;
                            next_x = nx;
                            next_y = ny;
                        }
                    }
                }

                if reached_water {
                    break;
                }
            }

            // Move to the lowest neighbor
            current_x = next_x;
            current_y = next_y;

            // Avoid infinite loops if we can't find a downhill path
            if river_points.contains(&(current_x, current_y)) {
                break;
            }
        }

        // Add the final water point if we reached water
        if reached_water {
            river_points.push((current_x, current_y));
        }

        // Create the river by adding water to all points along the path
        for &(x, y) in &river_points {
            // Use a wider river path based on river_width
            let river_width = self.params.river_width as isize;

            for dy in -river_width..=river_width {
                for dx in -river_width..=river_width {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;

                    // Check if in bounds
                    if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                        let nx = nx as usize;
                        let ny = ny as usize;

                        // Calculate distance from river center
                        let distance = ((dx * dx + dy * dy) as f64).sqrt();

                        // Only modify if within river width (with some randomness for natural look)
                        if distance < self.params.river_width + self.rng.gen_range(-0.3..0.3) {
                            let stack = &mut grid[ny][nx];

                            // Replace the stack with river terrain
                            *stack = ItemStack::with_base(Item::Sand);
                            stack.push(ItemBox::new(Item::Water));
                            stack.push(ItemBox::new(Item::Air));
                        }
                    }
                }
            }
        }
    }

    /// Generate features like vegetation, rocks, and logs
    fn generate_features(&mut self, grid: &mut Vec<Vec<ItemStack>>, width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                let stack = &mut grid[y][x];

                // Skip water and mountain tiles
                let mut skip_tile = false;
                for item in stack.items() {
                    match item.item {
                        Item::Water | Item::DeepWater | Item::Mountain | Item::Snow => {
                            skip_tile = true;
                            break;
                        }
                        _ => {}
                    }
                }

                if skip_tile {
                    continue;
                }

                // Get noise values for this position
                let nx = x as f64 * self.params.vegetation_scale;
                let ny = y as f64 * self.params.vegetation_scale;
                let vegetation = self.vegetation_noise.get([nx, ny]) * 0.5 + 0.5;
                let feature = self.feature_noise.get([nx * 2.0, ny * 2.0]) * 0.5 + 0.5;
                let detail_feature = self.feature_noise.get([nx * 7.0, ny * 7.0]) * 0.5 + 0.5;

                // Check for existing items
                let mut has_grass = false;
                let mut has_rock = false;
                let mut has_log = false;

                for item in stack.items() {
                    match item.item {
                        Item::Grass => has_grass = true,
                        Item::Rock => has_rock = true,
                        Item::Log => has_log = true,
                        _ => {}
                    }
                }

                // See if we're on a beach
                let mut is_beach = false;
                for item in stack.items() {
                    if let Item::Sand = item.item {
                        is_beach = true;
                        break;
                    }
                }

                // Add features based on the environment
                if is_beach {
                    // Beaches sometimes get rocks
                    if !has_rock && detail_feature > 0.88 {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Rock)); // Add rock
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }
                } else {
                    // Regular land

                    // Add grass if there isn't any already and the vegetation level is right
                    if !has_grass && vegetation > 0.4 && feature < 0.8 {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Grass)); // Add grass
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }

                    // Create forest clusters by adding logs
                    if vegetation > 0.7 && detail_feature > 0.85 && !has_log {
                        if has_grass {
                            // Add logs on top of grass
                            stack.pop(); // Remove air
                            stack.push(ItemBox::new(Item::Log)); // Add log
                            stack.push(ItemBox::new(Item::Air)); // Add air back on top
                        } else {
                            // Add logs on dirt with no grass
                            stack.pop(); // Remove air
                            stack.push(ItemBox::new(Item::Log)); // Add log
                            stack.push(ItemBox::new(Item::Air)); // Add air back on top
                        }
                    }

                    // Add scattered rocks in some areas, especially at higher elevations
                    if feature > 0.7 && detail_feature > 0.9 && !has_rock && !has_log {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Rock)); // Add rock
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }
                }
            }
        }

        // Second pass - create coherent features by checking neighboring tiles
        self.enhance_features(grid, width, height);
    }

    /// Enhance features by looking at neighboring tiles to create more coherent patterns
    fn enhance_features(&mut self, grid: &mut Vec<Vec<ItemStack>>, width: usize, height: usize) {
        // Create a copy of the grid to reference while making changes
        let grid_copy = grid.clone();

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                // Count neighboring features
                let mut log_count = 0;
                let mut grass_count = 0;
                let mut rock_count = 0;

                // Check all 8 neighboring cells
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue; // Skip the current cell
                        }

                        let nx = (x as isize + dx) as usize;
                        let ny = (y as isize + dy) as usize;

                        if nx < width && ny < height {
                            for item in grid_copy[ny][nx].items() {
                                match item.item {
                                    Item::Log => log_count += 1,
                                    Item::Grass => grass_count += 1,
                                    Item::Rock => rock_count += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                let stack = &mut grid[y][x];

                // Skip water and mountain tiles
                let mut skip_tile = false;
                for item in stack.items() {
                    match item.item {
                        Item::Water | Item::DeepWater | Item::Mountain | Item::Snow => {
                            skip_tile = true;
                            break;
                        }
                        _ => {}
                    }
                }

                if skip_tile {
                    continue;
                }

                // Enhance forests - add logs near other logs
                if log_count >= 3 {
                    // Check if we already have a log
                    let mut has_log = false;
                    for item in stack.items() {
                        if let Item::Log = item.item {
                            has_log = true;
                            break;
                        }
                    }

                    if !has_log && self.rng.gen_range(0.0..1.0) < 0.6 {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Log)); // Add log
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }
                }

                // Enhance grass - add grass near other grass
                if grass_count >= 5 && log_count < 2 {
                    // Check if we already have grass
                    let mut has_grass = false;
                    for item in stack.items() {
                        if let Item::Grass = item.item {
                            has_grass = true;
                            break;
                        }
                    }

                    if !has_grass && self.rng.gen_range(0.0..1.0) < 0.7 {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Grass)); // Add grass
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }
                }

                // Enhance rocky areas - add rocks near other rocks
                if rock_count >= 2 {
                    // Check if we already have a rock
                    let mut has_rock = false;
                    for item in stack.items() {
                        if let Item::Rock = item.item {
                            has_rock = true;
                            break;
                        }
                    }

                    if !has_rock && self.rng.gen_range(0.0..1.0) < 0.4 {
                        stack.pop(); // Remove air
                        stack.push(ItemBox::new(Item::Rock)); // Add rock
                        stack.push(ItemBox::new(Item::Air)); // Add air back on top
                    }
                }
            }
        }
    }
}
