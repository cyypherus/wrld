use crate::entity::item::{Color, Item};
use crate::world::World;
use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

/// Responsible for rendering the world to a pixel buffer
pub struct Renderer {
    pixels: Pixels,
    cell_size: usize,
    hover_position: Option<(usize, usize)>,
    hover_info: Option<String>,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(
        window: &Window,
        width: u32,
        height: u32,
        cell_size: usize,
    ) -> Result<Self, pixels::Error> {
        let surface_texture = SurfaceTexture::new(width, height, window);
        let pixels = Pixels::new(width, height, surface_texture)?;

        Ok(Renderer {
            pixels,
            cell_size,
            hover_position: None,
            hover_info: None,
        })
    }

    /// Update hover position based on mouse coordinates
    pub fn update_hover(&mut self, position: (f32, f32), world: &World) {
        let x = (position.0 / self.cell_size as f32) as usize;
        let y = (position.1 / self.cell_size as f32) as usize;

        // Check if position is within world bounds
        if x < world.width() && y < world.height() {
            self.hover_position = Some((x, y));

            // Generate hover info
            if let Some(stack) = world.get(x, y) {
                let mut info = String::new();

                // Add position info
                info.push_str(&format!("Position: ({}, {})\n", x, y));

                // Add items in stack (most visible first) with their effects
                info.push_str("Items:\n");

                let items = stack.items();

                if items.is_empty() {
                    info.push_str("  <empty>\n");
                } else {
                    for (index, item_box) in items.iter().enumerate() {
                        // Add the item name with index
                        info.push_str(&format!("  {}. {}\n", index + 1, item_box.get_name()));

                        // Add detailed entity information based on type
                        match &item_box.item {
                            Item::Traveler { name } => {
                                info.push_str(&format!("     Type: Traveler ({})\n", name));
                                info.push_str(
                                    "     Role: Autonomous entity that needs food and water\n",
                                );
                            }
                            Item::Corpse { name, item_type } => {
                                info.push_str(&format!("     Type: Corpse of {}\n", name));
                                match item_type {
                                    crate::entity::item::CorpseType::Traveler => {
                                        info.push_str("     Details: Remains of a traveler (can be consumed in emergencies)\n");
                                    }
                                    crate::entity::item::CorpseType::Animal(species) => {
                                        info.push_str(&format!("     Details: Remains of a {} (can be consumed for food)\n", species));
                                    }
                                }
                            }
                            Item::Object(obj) => {
                                info.push_str(&format!("     Type: Object ({})\n", obj));
                                match obj {
                                    crate::entity::effect::Object::Food(_) => {
                                        let (health, hunger, thirst) = item_box.nutrition();
                                        info.push_str(&format!("     Food Stats: +{}% health, -{}% hunger, -{}% thirst\n",
                                            health, hunger, thirst));
                                    }
                                    crate::entity::effect::Object::Weapon(weapon_type) => {
                                        info.push_str(&format!(
                                            "     Weapon Type: {:?}\n",
                                            weapon_type
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            Item::Water => {
                                info.push_str(
                                    "     Type: Water (drinkable, reduces thirst completely)\n",
                                );
                                info.push_str(
                                    "     Terrain: Can be traversed but slows movement\n",
                                );
                            }
                            Item::DeepWater => {
                                info.push_str("     Type: Deep Water (drinkable, reduces thirst completely)\n");
                                info.push_str(
                                    "     Terrain: Difficult to traverse without swimming skill\n",
                                );
                            }
                            Item::Road { connected } => {
                                info.push_str(&format!(
                                    "     Type: Road (connected: {})\n",
                                    connected
                                ));
                                info.push_str("     Terrain: Fastest travel path for entities\n");
                            }
                            _ => {}
                        }

                        // Add effects if there are any
                        if !item_box.effects.is_empty() {
                            info.push_str("     Effects:\n");
                            for effect in &item_box.effects {
                                // Enhanced effect description with intensity and duration
                                let mut effect_desc = "         ".to_string();

                                effect_desc.push_str(&format!("{}", effect.kind));

                                // Add effect description based on type
                                match &effect.kind {
                                    crate::entity::effect::EffectType::Healthy => {
                                        let health_status = match effect.intensity {
                                            0..=20 => "Recovering",
                                            21..=40 => "Stable",
                                            41..=60 => "Good",
                                            61..=80 => "Excellent",
                                            _ => "Peak condition",
                                        };
                                        effect_desc.push_str(&format!(" - {}", health_status));
                                    }
                                    crate::entity::effect::EffectType::Injured => {
                                        let injury_severity = match effect.intensity {
                                            0..=20 => "Minor scratches",
                                            21..=40 => "Moderate wounds",
                                            41..=60 => "Serious injuries",
                                            61..=80 => "Severe trauma",
                                            _ => "Critical condition",
                                        };
                                        effect_desc.push_str(&format!(" - {}", injury_severity));
                                    }
                                    crate::entity::effect::EffectType::Dead => {
                                        effect_desc.push_str(" - Entity is deceased");
                                    }
                                    crate::entity::effect::EffectType::Hungry => {
                                        let hunger_status = match effect.intensity {
                                            0..=20 => "Well fed",
                                            21..=40 => "Satisfied",
                                            41..=60 => "Hungry",
                                            61..=80 => "Very hungry",
                                            _ => "Starving",
                                        };
                                        effect_desc.push_str(&format!(" - {}", hunger_status));
                                    }
                                    crate::entity::effect::EffectType::Thirsty => {
                                        let thirst_status = match effect.intensity {
                                            0..=20 => "Hydrated",
                                            21..=40 => "Content",
                                            41..=60 => "Thirsty",
                                            61..=80 => "Very thirsty",
                                            _ => "Dehydrated",
                                        };
                                        effect_desc.push_str(&format!(" - {}", thirst_status));
                                    }
                                    crate::entity::effect::EffectType::Skilled(skill) => {
                                        match skill {
                                            crate::entity::effect::Skill::Swimming => {
                                                let skill_level = match effect.intensity {
                                                    0..=20 => "Novice swimmer",
                                                    21..=40 => "Decent swimmer",
                                                    41..=60 => "Competent swimmer",
                                                    61..=80 => "Strong swimmer",
                                                    _ => "Expert swimmer",
                                                };
                                                effect_desc
                                                    .push_str(&format!(" - {}", skill_level));
                                            }
                                        }
                                    }
                                    crate::entity::effect::EffectType::Holding(item) => {
                                        effect_desc.push_str(&format!(
                                            " - Currently holding {}",
                                            item.get_name()
                                        ));
                                    }
                                    crate::entity::effect::EffectType::PreferredDirection(
                                        direction,
                                    ) => {
                                        effect_desc
                                            .push_str(&format!(" - Prefers moving {}", direction));
                                    }
                                    crate::entity::effect::EffectType::Thinking(thought) => {
                                        effect_desc.push_str(&format!(" - Thinking {}", thought));
                                    }
                                    crate::entity::effect::EffectType::Tired => {
                                        effect_desc.push_str(" - Tired");
                                    }
                                    crate::entity::effect::EffectType::Young => {
                                        effect_desc.push_str(" - Young");
                                    }
                                }

                                // Add duration information if available
                                if let Some(duration) = &effect.duration {
                                    let seconds = duration;
                                    if *seconds > 60 {
                                        let minutes = seconds / 60;
                                        effect_desc.push_str(&format!(
                                            ", Duration: {}m {}s",
                                            minutes,
                                            seconds % 60
                                        ));
                                    } else {
                                        effect_desc.push_str(&format!(", Duration: {}s", seconds));
                                    }
                                } else {
                                    effect_desc.push_str(" (Permanent)");
                                }

                                info.push_str(&format!("{}\n", effect_desc));
                            }

                            // Add a blank line after effects for better readability
                            if index < items.len() - 1 {
                                info.push('\n');
                            }
                        }
                    }
                }

                self.hover_info = Some(info);
            } else {
                println!("No stack found at position ({}, {})", x, y);
                self.hover_info = None;
            }
        } else {
            println!("Position ({}, {}) is out of bounds", x, y);
            self.hover_position = None;
            self.hover_info = None;
        }
    }

    /// Clear hover information
    pub fn clear_hover(&mut self) {
        self.hover_position = None;
        self.hover_info = None;
    }

    /// Get current hover information
    pub fn get_hover_info(&self) -> Option<String> {
        self.hover_info.clone()
    }

    /// Render the world with the specified view mode filter
    ///
    /// View modes:
    /// 0 - All entities (default)
    /// 1 - Actors only (travelers and animals)
    /// 2 - Food only
    /// 3 - Items only (non-food objects)
    /// 4 - Elevation only
    pub fn render(&mut self, world: &World, view_mode: u8) {
        let width = world.width();
        let height = world.height();
        let buffer_width = self.pixels.texture().width() as usize;
        let buffer_height = self.pixels.texture().height() as usize;
        let frame = self.pixels.frame_mut();

        // Clear the frame to black
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0; // R
            pixel[1] = 0; // G
            pixel[2] = 0; // B
            pixel[3] = 255; // A
        }

        // Draw each cell
        for y in 0..height {
            for x in 0..width {
                if let Some(stack) = world.get(x, y) {
                    // Get the most visible non-air item in the stack
                    let item = stack.visible_item().map(|i| i.1);

                    let color = if view_mode == 4 {
                        let elevation = world.elevation_grid[y][x];
                        Color::new(elevation, elevation, elevation, 255)
                    } else {
                        // Apply view mode filtering
                        let filtered_item = match view_mode {
                            1 => {
                                // VIEW_MODE_ACTORS
                                item.filter(|item_box| {
                                    matches!(item_box.item, Item::Traveler { .. })
                                })
                            }
                            2 => {
                                // VIEW_MODE_FOOD
                                item.filter(|item_box| {
                                    matches!(
                                        item_box.item,
                                        Item::Object(crate::entity::effect::Object::Food(_))
                                    )
                                })
                            }
                            3 => {
                                // VIEW_MODE_ITEMS
                                item.filter(|item_box| {
                                    matches!(item_box.item, Item::Object(_))
                                        && !matches!(
                                            item_box.item,
                                            Item::Object(crate::entity::effect::Object::Food(_))
                                        )
                                })
                            }
                            _ => item, // VIEW_MODE_ALL (default)
                        };
                        filtered_item
                            .map(|i| i.get_color())
                            .unwrap_or(Color::TRANSPARENT)
                    };

                    // Draw a cell_size x cell_size square
                    for cy in 0..self.cell_size {
                        for cx in 0..self.cell_size {
                            let px = x * self.cell_size + cx;
                            let py = y * self.cell_size + cy;

                            // Skip if out of bounds
                            if px >= buffer_width || py >= buffer_height {
                                continue;
                            }

                            // Calculate pixel index
                            let idx = (py * buffer_width + px) * 4;

                            // Set pixel color
                            if idx + 3 < frame.len() {
                                frame[idx] = color.r;
                                frame[idx + 1] = color.g;
                                frame[idx + 2] = color.b;
                                frame[idx + 3] = color.a;
                            }
                        }
                    }
                }
            }
        }

        // Highlight hovered cell if any
        if let Some((hx, hy)) = self.hover_position {
            let border_width = 2;
            let highlight_color = [255, 255, 255, 180]; // White with semi-transparency

            // Draw a border around the hovered cell
            for cy in 0..self.cell_size {
                for cx in 0..self.cell_size {
                    // Only draw the border (not the entire cell)
                    let is_border = cx < border_width
                        || cx >= self.cell_size - border_width
                        || cy < border_width
                        || cy >= self.cell_size - border_width;

                    if is_border {
                        let px = hx * self.cell_size + cx;
                        let py = hy * self.cell_size + cy;

                        // Skip if out of bounds
                        if px >= buffer_width || py >= buffer_height {
                            continue;
                        }

                        // Calculate pixel index
                        let idx = (py * buffer_width + px) * 4;

                        // Set pixel color with alpha blending
                        if idx + 3 < frame.len() {
                            frame[idx] = (frame[idx] / 2) + (highlight_color[0] / 2); // R
                            frame[idx + 1] = (frame[idx + 1] / 2) + (highlight_color[1] / 2); // G
                            frame[idx + 2] = (frame[idx + 2] / 2) + (highlight_color[2] / 2); // B
                            frame[idx + 3] = highlight_color[3]; // A
                        }
                    }
                }
            }
        }
    }

    /// Get a reference to the pixels object
    pub fn pixels(&self) -> &Pixels {
        &self.pixels
    }

    /// Get a mutable reference to the pixels object
    pub fn pixels_mut(&mut self) -> &mut Pixels {
        &mut self.pixels
    }

    /// Resize the surface
    pub fn resize_surface(&mut self, width: u32, height: u32) -> Result<(), pixels::TextureError> {
        self.pixels.resize_surface(width, height)
    }
}
