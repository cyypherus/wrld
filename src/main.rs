mod entity;
mod render;
mod world;

use crate::world::World;
use entity::effect::{
    ClothingType, ContainerType, Effect, FoodType, MaterialType, Object, ToolType, WeaponType,
};
use entity::item::Item;

use pixels::Error;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use render::{Renderer, TextRenderer};
use std::time::{Duration, Instant};
use world::fluid_sim::FluidSim;

use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod handle_input;

// Size of each grid cell in pixels
const CELL_SIZE: usize = 8;
// World dimensions in cells
const WORLD_WIDTH: usize = 200;
const WORLD_HEIGHT: usize = 120;
// Window dimensions in pixels
const WINDOW_WIDTH: u32 = (WORLD_WIDTH * CELL_SIZE) as u32;
const WINDOW_HEIGHT: u32 = (WORLD_HEIGHT * CELL_SIZE) as u32;
// Target FPS
const TARGET_FPS: u64 = 30;
// Base simulation speed
const BASE_SIMULATION_SPEED: u64 = 100; // 100% speed

// View modes

const DEFAULT_HEALTH: u32 = 100;
const FOOD_SPAWN_RATE: u8 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    All,
    Actors,
    Food,
    Items,
    Elevation,
}

struct Application {
    input: WinitInputHelper,
    current_speed: u64,
    paused: bool,
    renderer: Renderer,
    current_view_mode: ViewMode,
    dpi_factor: f64,
    mouse_position: Option<(f32, f32)>,
    world: World,
    show_info: bool,
    window_width: u32,
    window_height: u32,
    text_renderer: TextRenderer,
}

fn main() -> Result<(), Error> {
    // Set up the window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("World Simulation")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let seed = fastrand::u64(..);
    println!("World Seed: {}", seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut world = World::new(WORLD_WIDTH, WORLD_HEIGHT, seed);
    let mut fluid_sim = FluidSim::new(WORLD_WIDTH, WORLD_HEIGHT, 10, &mut rng);

    for _ in 0..300 {
        fluid_sim.tick(false, &mut rng);
    }
    for _ in 0..10 {
        fluid_sim.tick(true, &mut rng);
    }

    // Generate terrain
    println!("Generating world...");
    world.generate(&mut fluid_sim);
    add_npcs(&mut world, 50);
    add_items(&mut world, 50);

    let mut last_update = Instant::now();

    let mut application = Application {
        input: WinitInputHelper::new(),
        current_speed: BASE_SIMULATION_SPEED,
        paused: false,
        renderer: Renderer::new(&window, WINDOW_WIDTH, WINDOW_HEIGHT, CELL_SIZE)?,
        current_view_mode: ViewMode::All,
        dpi_factor: window.scale_factor(),
        mouse_position: None,
        world,
        show_info: false,
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        text_renderer: TextRenderer::new(),
    };

    event_loop.run(move |event, _, control_flow| {
        let should_exit = application.handle_input(&window, &event, control_flow);
        if should_exit {
            return;
        }
        // Main game loop
        match event {
            Event::RedrawRequested(_) => {
                application.redraw(control_flow);
            }
            Event::MainEventsCleared => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_update);

                // Calculate frame time based on simulation speed
                let speed_percent = application.current_speed;
                let target_frame_time =
                    Duration::from_millis(1000 / TARGET_FPS * 100 / speed_percent);

                // Only update at target FPS, adjusted by simulation speed
                if elapsed >= target_frame_time {
                    last_update = now;

                    // Update world state only if not paused
                    if !application.paused {
                        // For speeds > 100%, perform multiple ticks per frame
                        let ticks =
                            speed_percent / 100 + if speed_percent % 100 > 0 { 1 } else { 0 };
                        for _ in 0..ticks {
                            application.world.tick();
                            // fluid_sim.tick(false, &mut rng);
                            // fluid_sim.to_elevation(&mut world.elevation_grid);
                        }
                    }

                    // Request a redraw
                    window.request_redraw();
                }

                // Schedule the next update
                *control_flow = ControlFlow::Poll;
            }
            _ => {}
        }
    });
}

// Add NPCs to the world
fn add_npcs(world: &mut World, count: usize) {
    // Add travelers to land areas only
    let mut attempts = 0;
    let mut traveler_count = 0;

    while traveler_count < count / 2 && attempts < count * 10 {
        let x = fastrand::usize(0..world.width());
        let y = fastrand::usize(0..world.height());

        // Check if this location is land (not water)
        if let Some(stack) = world.get(x, y) {
            let is_water = stack
                .items()
                .iter()
                .any(|item_box| matches!(item_box.item, Item::Water | Item::DeepWater));

            if !is_water {
                let name = generate_name();
                let traveler = Item::Traveler { name };
                world.add_item_with_effects(x, y, traveler, Effect::default_traveler_effects());
                traveler_count += 1;
            }
        }

        attempts += 1;
    }
}

// Add items to the world
fn add_items(world: &mut World, count: usize) {
    // Add food items and other objects
    let mut attempts = 0;
    let mut item_count = 0;

    while item_count < count && attempts < count * 10 {
        let x = fastrand::usize(0..world.width());
        let y = fastrand::usize(0..world.height());

        // Don't place items in water
        if let Some(stack) = world.get(x, y) {
            let is_water = stack
                .items()
                .iter()
                .any(|item_box| matches!(item_box.item, Item::Water | Item::DeepWater));

            let is_grass = stack
                .items()
                .iter()
                .any(|item_box| matches!(item_box.item, Item::Grass));

            if !is_water {
                // Create a random item
                let item_type = match fastrand::usize(0..10) {
                    // Food items (more common)
                    0 => Item::Object(Object::Food(FoodType::Bread)),
                    1 => Item::Object(Object::Food(FoodType::Vegetable)),
                    2 => Item::Object(Object::Food(FoodType::Fruit)),
                    // Tools (less common)
                    3 => Item::Object(Object::Tool(ToolType::Axe)),
                    4 => Item::Object(Object::Tool(ToolType::Hammer)),
                    // Weapons
                    5 => Item::Object(Object::Weapon(WeaponType::Sword)),
                    6 => Item::Object(Object::Weapon(WeaponType::Bow)),
                    // Clothing
                    7 => Item::Object(Object::Clothing(ClothingType::Shirt)),
                    // Container
                    8 => Item::Object(Object::Container(ContainerType::Bag)),
                    // Materials
                    _ => Item::Object(Object::Material(MaterialType::Wood)),
                };

                // Only place items on grass sometimes
                if !is_grass || fastrand::bool() {
                    world.add_item(x, y, item_type);
                    item_count += 1;
                }
            }
        }

        attempts += 1;
    }

    println!("Added {} items to the world", item_count);
}

// Generate a random name for travelers
fn generate_name() -> String {
    let first_parts = [
        "Ar", "Bel", "Cor", "Dan", "El", "Fal", "Gal", "Han", "Ir", "Jor", "Kal", "Lor", "Mal",
        "Nar", "Ober", "Pra", "Quin", "Rys", "Sul", "Tyr",
    ];
    let second_parts = [
        "thor", "mir", "dor", "iel", "lian", "wyn", "dred", "vic", "dar", "son", "grim", "mund",
        "wind", "rath", "kon", "thal", "wen", "zar", "kith", "varn",
    ];

    let first = first_parts[fastrand::usize(0..first_parts.len())];
    let second = second_parts[fastrand::usize(0..second_parts.len())];

    format!("{}{}", first, second)
}
