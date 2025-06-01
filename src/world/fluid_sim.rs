use rand::Rng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug)]
pub(crate) struct FluidSim {
    sources: Vec<(usize, usize)>,
    sinks: Vec<(usize, usize)>,
    pub(crate) pressure: Vec<Vec<f32>>,
    velocity_x: Vec<Vec<f32>>,
    velocity_y: Vec<Vec<f32>>,
    ambient_angle: f32,
    ambient_magnitude: f32,
    width: usize,
    height: usize,
    padding: usize,
}

impl FluidSim {
    pub(crate) fn new(width: usize, height: usize, padding: usize, rng: &mut ChaCha8Rng) -> Self {
        let pressure = vec![vec![0.0; width + padding * 2]; height + padding * 2];
        let velocity_x = vec![vec![0.0; width + padding * 2]; height + padding * 2];
        let velocity_y = vec![vec![0.0; width + padding * 2]; height + padding * 2];

        let source_count = 11;
        let mut sources = Vec::new();
        for _ in 0..source_count {
            sources.push((
                rng.random::<u8>() as usize % width,
                rng.random::<u8>() as usize % height,
            ));
        }
        let mut sinks = Vec::new();
        for _ in 0..8 {
            sinks.push((
                rng.random::<u8>() as usize % width,
                rng.random::<u8>() as usize % height,
            ));
        }

        let ambient_magnitude = rng.random::<f32>() * 0.04;
        let ambient_angle = rng.random::<f32>() * 2.0 * std::f32::consts::PI;

        FluidSim {
            sources,
            sinks,
            pressure,
            velocity_x,
            velocity_y,
            ambient_angle,
            ambient_magnitude,
            width,
            height,
            padding,
        }
    }

    pub(crate) fn to_elevation(&self, elevation_grid: &mut [Vec<u8>]) {
        for (y, row) in elevation_grid.iter_mut().enumerate() {
            for (x, column) in row.iter_mut().enumerate() {
                *column = (self.pressure[y + self.padding][x + self.padding]).min(255.0) as u8;
            }
        }
    }

    pub(crate) fn tick(&mut self, finishing: bool, rng: &mut ChaCha8Rng) {
        self.ambient_angle += rng.random_range::<f32, _>(-0.02..0.02);
        if self.ambient_angle < 0.0 {
            self.ambient_angle += 2.0 * std::f32::consts::PI;
        } else if self.ambient_angle > 2.0 * std::f32::consts::PI {
            self.ambient_angle -= 2.0 * std::f32::consts::PI;
        }

        self.ambient_magnitude += rng.random_range(-0.02..0.02);
        self.ambient_magnitude = self.ambient_magnitude.clamp(-0.03, 0.03);

        let ambient_x = self.ambient_angle.cos() * self.ambient_magnitude;
        let ambient_y = self.ambient_angle.sin() * self.ambient_magnitude;

        if !finishing {
            for &(sx, sy) in &self.sources {
                self.pressure[sy][sx] = 255.0;
            }
            for &(sx, sy) in &self.sinks {
                self.pressure[sy][sx] = 0.0;
            }
        }

        let mut new_pressure = self.pressure.clone();
        let mut new_vx = self.velocity_x.clone();
        let mut new_vy = self.velocity_y.clone();

        // Increased constants for more turbulence
        const GRADIENT_SCALE: f32 = 0.7;
        const PRESSURE_FORCE: f32 = 0.005;
        const VELOCITY_DAMPING: f32 = 0.99; // Less damping for more persistent movement
        const PRESSURE_DECAY: f32 = 0.985; // Slower decay
        const DIVERGENCE_SCALE: f32 = 1.2; // Increased divergence
        const DIVERGENCE_PRESSURE_FACTOR: f32 = 0.7;

        // Process interior cells
        let sim_height = self.height + (self.padding * 2);
        let sim_width = self.width + (self.padding * 2);
        for y in 1..sim_height - 1 {
            for x in 1..sim_width - 1 {
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
                    && src_x_i < (sim_width - 1) as i32
                    && src_y_i >= 0
                    && src_y_i < (sim_height - 1) as i32
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
