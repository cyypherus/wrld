use crate::entity::item::{Item, ItemBox};
use crate::world::world::ItemStack;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

pub(crate) struct WorldGenerator;

impl WorldGenerator {
    pub fn generate(
        &mut self,
        elevation_grid: &Vec<Vec<u8>>,
        width: usize,
        height: usize,
    ) -> Vec<Vec<ItemStack>> {
        let mut grid = vec![vec![ItemStack::new(); width]; height];
        self.generate_features(&mut grid, &elevation_grid, width, height);
        grid
    }

    fn generate_features(
        &mut self,
        grid: &mut Vec<Vec<ItemStack>>,
        elevation_grid: &Vec<Vec<u8>>,
        width: usize,
        height: usize,
    ) {
        const DEEP_WATER: u8 = 20;
        const WATER: u8 = 45;
        const SAND: u8 = 50;
        const GRASS: u8 = 60;
        const FOREST: u8 = 100;
        const MOUNTAIN: u8 = 130;

        for y in 0..height {
            for x in 0..width {
                let elevation = elevation_grid[y][x];
                match elevation {
                    ..DEEP_WATER => grid[y][x].items.push(ItemBox::new(Item::DeepWater)),
                    DEEP_WATER..WATER => grid[y][x].items.push(ItemBox::new(Item::Water)),
                    WATER..SAND => grid[y][x].items.push(ItemBox::new(Item::Sand)),
                    SAND..GRASS => grid[y][x].items.push(ItemBox::new(Item::Grass)),
                    GRASS..FOREST => grid[y][x].items.push(ItemBox::new(Item::Forest)),
                    FOREST..MOUNTAIN => grid[y][x].items.push(ItemBox::new(Item::Mountain)),
                    MOUNTAIN.. => grid[y][x].items.push(ItemBox::new(Item::Snow)),
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct FluidSim {
    sources: Vec<(usize, usize)>,
    pub(crate) pressure: Vec<Vec<f32>>,
    velocity_x: Vec<Vec<f32>>,
    velocity_y: Vec<Vec<f32>>,
    ambient_angle: f32,
    ambient_magnitude: f32,
    width: usize,
    height: usize,
}

impl FluidSim {
    pub(crate) fn new(width: usize, height: usize, rng: &mut ChaCha8Rng) -> Self {
        let pressure = vec![vec![0.0; width]; height];
        let velocity_x = vec![vec![0.0; width]; height];
        let velocity_y = vec![vec![0.0; width]; height];

        let source_count = 5 + (rng.random::<u8>() % 8);
        let mut sources = Vec::new();
        for _ in 0..source_count {
            sources.push((
                rng.random::<u8>() as usize % width,
                rng.random::<u8>() as usize % height,
            ));
        }

        let ambient_magnitude = rng.random::<f32>() * 0.04;
        let ambient_angle = rng.random::<f32>() * 2.0 * std::f32::consts::PI;

        FluidSim {
            sources,
            pressure,
            velocity_x,
            velocity_y,
            ambient_angle,
            ambient_magnitude,
            width,
            height,
        }
    }

    pub(crate) fn to_elevation(&self, elevation_grid: &mut Vec<Vec<u8>>) {
        for y in 0..self.height {
            for x in 0..self.width {
                elevation_grid[y][x] = (self.pressure[y][x]).min(255.0) as u8;
            }
        }
    }

    pub(crate) fn tick(&mut self, finishing: bool) {
        let ambient_x = self.ambient_angle.cos() * self.ambient_magnitude;
        let ambient_y = self.ambient_angle.sin() * self.ambient_magnitude;
        self.ambient_angle += 0.03; // Slow rotation

        if !finishing {
            for &(sx, sy) in &self.sources {
                self.pressure[sy][sx] = 255.0;
            }
        }

        let mut new_pressure = self.pressure.clone();
        let mut new_vx = self.velocity_x.clone();
        let mut new_vy = self.velocity_y.clone();

        // Fluid simulation constants
        // const GRADIENT_SCALE: f32 = 0.5;
        // const PRESSURE_FORCE: f32 = 0.002;
        // const VELOCITY_DAMPING: f32 = 0.99;
        // const PRESSURE_DECAY: f32 = 0.99;
        // const DIVERGENCE_SCALE: f32 = 0.5;
        // const DIVERGENCE_PRESSURE_FACTOR: f32 = 0.5;

        const GRADIENT_SCALE: f32 = 0.5;
        const PRESSURE_FORCE: f32 = 0.003;
        const VELOCITY_DAMPING: f32 = 0.995;
        const PRESSURE_DECAY: f32 = 0.99;
        const DIVERGENCE_SCALE: f32 = 0.8;
        const DIVERGENCE_PRESSURE_FACTOR: f32 = 0.5;

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.sources.contains(&(x, y)) && !finishing {
                    continue;
                }

                let grad_x = (self.pressure[y][x + 1] - self.pressure[y][x - 1]) * GRADIENT_SCALE;
                let grad_y = (self.pressure[y + 1][x] - self.pressure[y - 1][x]) * GRADIENT_SCALE;

                new_vx[y][x] = (self.velocity_x[y][x] - grad_x * PRESSURE_FORCE + ambient_x)
                    * VELOCITY_DAMPING;
                new_vy[y][x] = (self.velocity_y[y][x] - grad_y * PRESSURE_FORCE + ambient_y)
                    * VELOCITY_DAMPING;

                // Advect pressure using velocity
                let vx = self.velocity_x[y][x];
                let vy = self.velocity_y[y][x];

                // Simple backward Euler advection
                let src_x = x as f32 - vx;
                let src_y = y as f32 - vy;

                let src_x_i = src_x.floor() as i32;
                let src_y_i = src_y.floor() as i32;

                if src_x_i >= 0
                    && src_x_i < (self.width - 1) as i32
                    && src_y_i >= 0
                    && src_y_i < (self.height - 1) as i32
                {
                    let fx = src_x - src_x_i as f32;
                    let fy = src_y - src_y_i as f32;

                    let ux = src_x_i as usize;
                    let uy = src_y_i as usize;

                    // Bilinear interpolation
                    let p00 = self.pressure[uy][ux];
                    let p10 = self.pressure[uy][ux + 1];
                    let p01 = self.pressure[uy + 1][ux];
                    let p11 = self.pressure[uy + 1][ux + 1];

                    let p0 = p00 * (1.0 - fx) + p10 * fx;
                    let p1 = p01 * (1.0 - fx) + p11 * fx;
                    let advected_pressure = p0 * (1.0 - fy) + p1 * fy;

                    new_pressure[y][x] = advected_pressure * PRESSURE_DECAY;
                }

                let div = (self.velocity_x[y][x + 1] - self.velocity_x[y][x - 1]
                    + self.velocity_y[y + 1][x]
                    - self.velocity_y[y - 1][x])
                    * DIVERGENCE_SCALE;
                new_pressure[y][x] =
                    (new_pressure[y][x] - div * DIVERGENCE_PRESSURE_FACTOR).max(0.0);
            }
        }

        self.pressure = new_pressure;
        self.velocity_x = new_vx;
        self.velocity_y = new_vy;
    }
}
