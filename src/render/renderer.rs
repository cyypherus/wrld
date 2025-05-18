use crate::entity::item::{Color, Item};
use crate::world::World;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::PhysicalPosition;
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
    pub fn update_hover(&mut self, position: PhysicalPosition<f64>, world: &World) {
        let x = (position.x / self.cell_size as f64) as usize;
        let y = (position.y / self.cell_size as f64) as usize;

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
                let visible_items: Vec<_> = items
                    .iter()
                    .rev()
                    .filter(|item_box| !matches!(item_box.item, Item::Air))
                    .collect();

                if visible_items.is_empty() {
                    info.push_str("  <empty>\n");
                } else {
                    for (index, item_box) in visible_items.iter().enumerate() {
                        // Add the item name with index
                        info.push_str(&format!("  {}. {}\n", index + 1, item_box.get_name()));

                        // Add effects if there are any
                        if !item_box.effects.is_empty() {
                            info.push_str("     Effects:\n");
                            for effect in &item_box.effects {
                                info.push_str(&format!("       • {}\n", effect.description()));
                            }
                            // Add a blank line after effects for better readability
                            if index < visible_items.len() - 1 {
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

    /// Render the world
    pub fn render(&mut self, world: &World) {
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
                    let item = stack.visible_item().map(|i| i.0);
                    let color = item.map(|i| i.get_color()).unwrap_or(Color::TRANSPARENT);

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
