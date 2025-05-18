use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use pixels::Pixels;

/// Simple text renderer using embedded-graphics
pub struct TextRenderer {
    width: u32,
    height: u32,
}

/// Custom drawing target for pixels buffer
struct PixelsDrawTarget<'a> {
    frame: &'a mut [u8],
    width: u32,
}

impl OriginDimensions for PixelsDrawTarget<'_> {
    fn size(&self) -> Size {
        Size::new(
            self.width,
            (self.frame.len() / 4 / self.width as usize) as u32,
        )
    }
}

impl DrawTarget for PixelsDrawTarget<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=u32::MAX, y @ 0..=u32::MAX)) = coord.try_into() {
                if x < self.width {
                    let idx = (y * self.width + x) as usize * 4;
                    if idx + 3 < self.frame.len() {
                        self.frame[idx] = color.r();
                        self.frame[idx + 1] = color.g();
                        self.frame[idx + 2] = color.b();
                        self.frame[idx + 3] = 255;
                    }
                }
            }
        }
        Ok(())
    }
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        TextRenderer {
            width: 800,
            height: 600,
        }
    }

    /// Draw text with a background
    pub fn draw_text(
        &mut self,
        text: &str,
        x: usize,
        y: usize,
        text_color: [u8; 3],
        bg_color: [u8; 4],
        pixels: &mut Pixels,
    ) {
        // Update dimensions if needed
        self.width = pixels.texture().width();
        self.height = pixels.texture().height();

        let frame = pixels.frame_mut();

        // Create a drawing target for the pixels buffer
        let mut display = PixelsDrawTarget {
            frame,
            width: self.width,
        };

        // Split text into lines
        let lines: Vec<&str> = text.lines().collect();

        // Calculate background dimensions
        let char_width = 6;
        let char_height = 10;
        let line_spacing = 2;
        let padding = 4;

        let longest_line = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let text_width = longest_line * char_width + padding * 2;
        let text_height = lines.len() * (char_height + line_spacing) + padding * 2;

        // Draw background with alpha blending
        let bg_style =
            PrimitiveStyle::with_fill(Rgb888::new(bg_color[0], bg_color[1], bg_color[2]));
        Rectangle::new(
            Point::new(x as i32, y as i32),
            Size::new(text_width as u32, text_height as u32),
        )
        .into_styled(bg_style)
        .draw(&mut display)
        .unwrap();

        // Draw each line of text
        let text_style = MonoTextStyle::new(
            &FONT_6X10,
            Rgb888::new(text_color[0], text_color[1], text_color[2]),
        );

        for (i, line) in lines.iter().enumerate() {
            let line_y = y + ((i + 1) * char_height);

            Text::new(
                line,
                Point::new(x as i32 + padding as i32, line_y as i32),
                text_style,
            )
            .draw(&mut display)
            .unwrap();
        }
    }
}
