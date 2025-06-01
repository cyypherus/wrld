use crate::entity::effect::Effect;
use crate::entity::item::{Item, ItemBox};
use crate::world::world::ItemStack;

pub(crate) struct WorldGenerator;

impl WorldGenerator {
    pub fn generate(
        &mut self,
        elevation_grid: &Vec<Vec<u8>>,
        width: usize,
        height: usize,
    ) -> Vec<Vec<ItemStack>> {
        let mut grid = vec![vec![ItemStack::new(); width]; height];
        self.generate_features(&mut grid, elevation_grid, width, height);
        grid
    }

    fn generate_features(
        &mut self,
        grid: &mut Vec<Vec<ItemStack>>,
        elevation_grid: &Vec<Vec<u8>>,
        width: usize,
        height: usize,
    ) {
        // Collect all elevation values and sort them
        let mut all_elevations = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                all_elevations.push(elevation_grid[y][x]);
            }
        }

        all_elevations.sort_unstable();

        // Calculate thresholds based on percentiles
        let total_cells = all_elevations.len();

        let deep_water_threshold = all_elevations[(0.1 * total_cells as f64) as usize];
        let water_threshold = all_elevations[(0.4 * total_cells as f64) as usize];
        let sand_threshold = all_elevations[(0.45 * total_cells as f64) as usize];
        let grass_threshold = all_elevations[(0.5 * total_cells as f64) as usize];
        let forest_threshold = all_elevations[(0.9 * total_cells as f64) as usize];
        let mountain_threshold = all_elevations[(0.99 * total_cells as f64) as usize];

        // Assign biomes based on actual percentile thresholds
        for y in 0..height {
            for x in 0..width {
                let elevation = elevation_grid[y][x];

                if elevation <= deep_water_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::DeepWater,
                        Effect::default_terrain_effects(),
                    ));
                } else if elevation <= water_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Water,
                        Effect::default_terrain_effects(),
                    ));
                } else if elevation <= sand_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Sand,
                        Effect::default_terrain_effects(),
                    ));
                } else if elevation <= grass_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Grass,
                        Effect::default_terrain_effects(),
                    ));
                } else if elevation <= forest_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Forest,
                        Effect::default_terrain_effects(),
                    ));
                } else if elevation <= mountain_threshold {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Mountain,
                        Effect::default_terrain_effects(),
                    ));
                } else {
                    grid[y][x].items.push(ItemBox::with_effects(
                        Item::Snow,
                        Effect::default_terrain_effects(),
                    ));
                }
            }
        }
    }
}
