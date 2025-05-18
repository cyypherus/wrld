mod entity;
mod render;
mod world;

use crate::world::World;
use entity::effect::{
    ClothingType, ContainerType, Effect, EffectType, FoodType, MaterialType, Object, Skill,
    ToolType, WeaponType,
};
use entity::item::Item;
use pixels::Error;
use render::{Renderer, TextRenderer};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

// Size of each grid cell in pixels
const CELL_SIZE: usize = 8;
// World dimensions in cells
const WORLD_WIDTH: usize = 120;
const WORLD_HEIGHT: usize = 90;
// Window dimensions in pixels
const WINDOW_WIDTH: u32 = (WORLD_WIDTH * CELL_SIZE) as u32;
const WINDOW_HEIGHT: u32 = (WORLD_HEIGHT * CELL_SIZE) as u32;
// Target FPS
const TARGET_FPS: u64 = 30;
// Base simulation speed
const BASE_SIMULATION_SPEED: u64 = 25; // 100% speed
static SIMULATION_SPEED: AtomicU64 = AtomicU64::new(BASE_SIMULATION_SPEED);

// View modes
const VIEW_MODE_ALL: u8 = 0;
const VIEW_MODE_ACTORS: u8 = 1;
const VIEW_MODE_FOOD: u8 = 2;
const VIEW_MODE_ITEMS: u8 = 3;
static VIEW_MODE: AtomicU8 = AtomicU8::new(VIEW_MODE_ALL);

const DEFAULT_HEALTH: u32 = 100;
const FOOD_SPAWN_RATE: u8 = 50;

fn main() -> Result<(), Error> {
    // Set up the window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("World Simulation")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    // Set up input handling
    let mut input = WinitInputHelper::new();

    // Create the world with a random seed
    let seed = fastrand::u64(..);
    println!("World Seed: {}", seed);
    let mut world = World::new(WORLD_WIDTH, WORLD_HEIGHT, seed);

    // Generate terrain
    println!("Generating world...");
    world.generate();

    // Print information about generated towns
    println!("\nTowns Generated:");
    for (name, (x, y)) in world.get_towns() {
        println!("- {} at position ({}, {})", name, x, y);
    }

    // Add some NPCs
    add_npcs(&mut world, 50);

    // Add additional items to the world
    add_items(&mut world, 50);

    // Create the renderer
    let mut renderer = Renderer::new(&window, WINDOW_WIDTH, WINDOW_HEIGHT, CELL_SIZE)?;

    // Game loop timing
    let mut last_update = Instant::now();

    // Track mouse position for hover information
    let mut mouse_position = None;
    let mut show_info = false;

    // Simulation state
    let mut paused = false;
    let mut current_speed = BASE_SIMULATION_SPEED;
    let mut current_view_mode = VIEW_MODE_ALL;

    // Create a simple text renderer that writes directly to pixel buffer
    let mut text_renderer = TextRenderer::new();

    event_loop.run(move |event, _, control_flow| {
        // Handle input
        if input.update(&event) {
            // Close the window if escape is pressed
            if input.key_pressed(VirtualKeyCode::Escape) {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Toggle pause with the space bar
            if input.key_pressed(VirtualKeyCode::Space) {
                paused = !paused;
                println!("Simulation {}", if paused { "paused" } else { "resumed" });
                window.request_redraw();
            }

            // Increase speed with Up arrow
            if input.key_pressed(VirtualKeyCode::Up) {
                current_speed = (current_speed + 5).min(300); // Maximum 300% speed
                SIMULATION_SPEED.store(current_speed, Ordering::Relaxed);
                println!("Simulation speed: {}%", current_speed);
                window.request_redraw();
            }

            // Decrease speed with Down arrow
            if input.key_pressed(VirtualKeyCode::Down) {
                current_speed = current_speed.saturating_sub(5).max(5); // Minimum 25% speed
                SIMULATION_SPEED.store(current_speed, Ordering::Relaxed);
                println!("Simulation speed: {}%", current_speed);
                window.request_redraw();
            }

            // Toggle view mode with number keys
            if input.key_pressed(VirtualKeyCode::Key1) {
                current_view_mode = VIEW_MODE_ALL;
                VIEW_MODE.store(current_view_mode, Ordering::Relaxed);
                println!("View mode: All (default)");
                window.request_redraw();
            } else if input.key_pressed(VirtualKeyCode::Key2) {
                current_view_mode = VIEW_MODE_ACTORS;
                VIEW_MODE.store(current_view_mode, Ordering::Relaxed);
                println!("View mode: Actors only");
                window.request_redraw();
            } else if input.key_pressed(VirtualKeyCode::Key3) {
                current_view_mode = VIEW_MODE_FOOD;
                VIEW_MODE.store(current_view_mode, Ordering::Relaxed);
                println!("View mode: Food only");
                window.request_redraw();
            } else if input.key_pressed(VirtualKeyCode::Key4) {
                current_view_mode = VIEW_MODE_ITEMS;
                VIEW_MODE.store(current_view_mode, Ordering::Relaxed);
                println!("View mode: Items only");
                window.request_redraw();
            }

            // Handle window resizing
            if let Some(size) = input.window_resized() {
                if let Err(e) = renderer.resize_surface(size.width, size.height) {
                    eprintln!("Error resizing surface: {}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                // No need to resize our simple text renderer
            }

            // Track mouse position
            if let Some(position) = input.mouse() {
                mouse_position = Some(position);

                // Update hover information
                let physical_position = PhysicalPosition::new(position.0 as f64, position.1 as f64);
                renderer.update_hover(physical_position, &world);

                // Request a redraw immediately when hover changes
                window.request_redraw();
            }

            // Toggle info display with right mouse button
            if input.mouse_pressed(0) || input.mouse_pressed(1) || input.mouse_pressed(2) {
                show_info = !show_info;

                // Request a redraw immediately when toggling hover display
                window.request_redraw();
            }
        }

        // Handle mouse events
        match &event {
            Event::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                ..
            } => {
                renderer.clear_hover();
                show_info = false;

                // Request a redraw when cursor leaves the window
                window.request_redraw();
            }
            _ => {}
        }

        // Main game loop
        match event {
            Event::RedrawRequested(_) => {
                // Render the world with current view mode
                renderer.render(&world, VIEW_MODE.load(Ordering::Relaxed));

                // Draw paused status if needed
                // Draw simulation status (paused/speed) and view mode
                let view_mode_text = match current_view_mode {
                    VIEW_MODE_ALL => "All (1)",
                    VIEW_MODE_ACTORS => "Actors Only (2)",
                    VIEW_MODE_FOOD => "Food Only (3)",
                    VIEW_MODE_ITEMS => "Items Only (4)",
                    _ => "Unknown",
                };

                let status_text = if paused {
                    format!(
                        "PAUSED - Press SPACE to resume - Speed: {}% - View: {}",
                        current_speed, view_mode_text
                    )
                } else {
                    format!(
                        "Speed: {}% - Use UP/DOWN arrows to adjust, SPACE to pause - View: {}",
                        current_speed, view_mode_text
                    )
                };

                text_renderer.draw_text(
                    &status_text,
                    10,
                    10,
                    [255, 255, 50], // yellow
                    [0, 0, 0, 200], // background
                    renderer.pixels_mut(),
                );

                // Render hover information
                if show_info && mouse_position.is_some() {
                    // Get the hover info and render it
                    if let Some(hover_info) = renderer.get_hover_info() {
                        // Calculate text dimensions
                        let lines: Vec<&str> = hover_info.lines().collect();
                        let line_count = lines.len();
                        let longest_line = lines.iter().map(|line| line.len()).max().unwrap_or(0);

                        // Estimate text box dimensions (based on font size in TextRenderer)
                        let char_width = 6;
                        let char_height = 10;
                        let line_spacing = 2;
                        let padding = 0;
                        let text_width = longest_line * char_width + padding * 2;
                        let text_height = line_count * (char_height + line_spacing) + padding * 2;

                        // Get mouse position
                        let mouse_x = mouse_position.unwrap().0 as usize;
                        let mouse_y = mouse_position.unwrap().1 as usize;

                        // Initial position (offset from cursor)
                        let mut x = mouse_x + 15;
                        let mut y = mouse_y + 15;

                        // Adjust if text would go off-screen
                        if x + text_width > WINDOW_WIDTH as usize {
                            x = mouse_x.saturating_sub(text_width + 15);
                        }

                        if y + text_height > WINDOW_HEIGHT as usize {
                            y = mouse_y.saturating_sub(text_height + 15);
                        }

                        // Final safety clamp to ensure we're on screen
                        let safe_x = x.min((WINDOW_WIDTH as usize).saturating_sub(text_width));
                        let safe_y = y.min(WINDOW_HEIGHT as usize).saturating_sub(text_height);

                        text_renderer.draw_text(
                            &hover_info,
                            safe_x,
                            safe_y,
                            [255, 255, 255], // white
                            [0, 0, 0, 200],  // slightly more opaque black background
                            renderer.pixels_mut(),
                        );
                    }
                }

                if let Err(e) = renderer.pixels().render() {
                    eprintln!("Error rendering: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_update);

                // Calculate frame time based on simulation speed
                let speed_percent = SIMULATION_SPEED.load(Ordering::Relaxed);
                let target_frame_time =
                    Duration::from_millis(1000 / TARGET_FPS * 100 / speed_percent);

                // Only update at target FPS, adjusted by simulation speed
                if elapsed >= target_frame_time {
                    last_update = now;

                    // Update world state only if not paused
                    if !paused {
                        // For speeds > 100%, perform multiple ticks per frame
                        let ticks =
                            speed_percent / 100 + if speed_percent % 100 > 0 { 1 } else { 0 };
                        for _ in 0..ticks {
                            world.tick();
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

                let mut effects = vec![Effect::permanent(
                    EffectType::Healthy,
                    DEFAULT_HEALTH, // 100% health
                )];

                // Give a small percentage of travelers the ability to swim
                if fastrand::u8(0..100) < 10 {
                    // 10% chance
                    effects.push(Effect::permanent(
                        EffectType::Skilled(Skill::Swimming), // Swimming ability
                        100,
                    ));
                }

                world.add_item_with_effects(x, y, traveler, effects);
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
