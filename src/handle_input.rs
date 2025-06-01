use crate::{Application, ViewMode};
use winit::dpi::LogicalPosition;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::ControlFlow;
use winit::window::Window;

impl Application {
    pub fn handle_input(
        &mut self,
        window: &Window,
        event: &Event<'_, ()>,
        control_flow: &mut ControlFlow,
    ) -> bool {
        let true = self.input.update(event) else {
            return false;
        };

        // Close the window if escape is pressed
        if self.input.key_pressed(VirtualKeyCode::Escape) || self.input.close_requested() {
            *control_flow = ControlFlow::Exit;
            return true;
        }

        // Toggle pause with the space bar
        if self.input.key_pressed(VirtualKeyCode::Space) {
            self.paused = !self.paused;
            println!(
                "Simulation {}",
                if self.paused { "paused" } else { "resumed" }
            );
            window.request_redraw();
        }

        // Increase speed with Up arrow
        if self.input.key_pressed(VirtualKeyCode::Up) {
            self.current_speed = (self.current_speed + 5).min(800);
            println!("Simulation speed: {}%", self.current_speed);
            window.request_redraw();
        }

        // Decrease speed with Down arrow
        if self.input.key_pressed(VirtualKeyCode::Down) {
            self.current_speed = self.current_speed.saturating_sub(5).max(5); // Minimum 25% speed
            println!("Simulation speed: {}%", self.current_speed);
            window.request_redraw();
        }

        // Toggle view mode with number keys
        if self.input.key_pressed(VirtualKeyCode::Key1) {
            self.current_view_mode = ViewMode::All;
            println!("View mode: All (default)");
            window.request_redraw();
        } else if self.input.key_pressed(VirtualKeyCode::Key2) {
            self.current_view_mode = ViewMode::Actors;
            println!("View mode: Actors only");
            window.request_redraw();
        } else if self.input.key_pressed(VirtualKeyCode::Key3) {
            self.current_view_mode = ViewMode::Food;
            println!("View mode: Food only");
            window.request_redraw();
        } else if self.input.key_pressed(VirtualKeyCode::Key4) {
            self.current_view_mode = ViewMode::Items;
            println!("View mode: Items only");
            window.request_redraw();
        } else if self.input.key_pressed(VirtualKeyCode::Key5) {
            self.current_view_mode = ViewMode::Elevation;
            println!("View mode: Elevation");
            window.request_redraw();
        }

        if let Some(size) = self.input.window_resized() {
            self.dpi_factor = window.scale_factor();
            println!("Updated DPI scaling factor: {}", self.dpi_factor);

            if let Err(e) = self.renderer.resize_surface(size.width, size.height) {
                eprintln!("Error resizing surface: {}", e);
                *control_flow = ControlFlow::Exit;
                return true;
            }
        }

        if let Some(position) = self.input.mouse() {
            let logical_position = LogicalPosition::new(position.0 as f64, position.1 as f64);
            let pos = (
                (logical_position.x / self.dpi_factor) as f32,
                (logical_position.y / self.dpi_factor) as f32,
            );
            self.mouse_position = Some(pos);
            self.renderer.update_hover(pos, &self.world);
            window.request_redraw();
        }

        if self.input.mouse_pressed(0) || self.input.mouse_pressed(1) || self.input.mouse_pressed(2)
        {
            self.show_info = !self.show_info;
            window.request_redraw();
        }
        false
    }

    pub fn redraw(&mut self, control_flow: &mut ControlFlow) {
        // Render the world with current view mode
        self.renderer.render(&self.world, self.current_view_mode);

        // Draw paused status if needed
        // Draw simulation status (paused/speed) and view mode
        let view_mode_text = match self.current_view_mode {
            ViewMode::All => "All",
            ViewMode::Actors => "Actors Only",
            ViewMode::Food => "Food Only",
            ViewMode::Items => "Items Only",
            ViewMode::Elevation => "Elevation Only",
        };

        let status_text = if self.paused {
            format!(
                "PAUSED - Press SPACE to resume - Speed: {}% - View: {}",
                self.current_speed, view_mode_text
            )
        } else {
            format!(
                "Speed: {}% - Use UP/DOWN arrows to adjust, SPACE to pause - View: {}",
                self.current_speed, view_mode_text
            )
        };

        self.text_renderer.draw_text(
            &status_text,
            10,
            10,
            [255, 255, 50], // yellow
            [0, 0, 0, 200], // background
            self.renderer.pixels_mut(),
        );

        // Render hover information
        if self.show_info && self.mouse_position.is_some() {
            // Get the hover info and render it
            if let Some(hover_info) = self.renderer.hover_info.clone() {
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
                let mouse_x = self.mouse_position.unwrap().0 as usize;
                let mouse_y = self.mouse_position.unwrap().1 as usize;

                // Initial position (offset from cursor)
                let mut x = mouse_x + 15;
                let mut y = mouse_y + 15;

                // Adjust if text would go off-screen
                if x + text_width > self.window_width as usize {
                    x = mouse_x.saturating_sub(text_width + 15);
                }

                if y + text_height > self.window_height as usize {
                    y = mouse_y.saturating_sub(text_height + 15);
                }

                // Final safety clamp to ensure we're on screen
                let safe_x = x.min((self.window_width as usize).saturating_sub(text_width));
                let safe_y = y
                    .min(self.window_height as usize)
                    .saturating_sub(text_height);

                self.text_renderer.draw_text(
                    &hover_info,
                    safe_x,
                    safe_y,
                    [255, 255, 255], // white
                    [0, 0, 0, 200],  // slightly more opaque black background
                    self.renderer.pixels_mut(),
                );
            }
        }

        if let Err(e) = self.renderer.pixels().render() {
            eprintln!("Error rendering: {}", e);
            *control_flow = ControlFlow::Exit;
        }
    }
}
