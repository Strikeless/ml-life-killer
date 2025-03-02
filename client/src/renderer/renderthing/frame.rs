pub const PIXEL_BITS: usize = 4;

pub struct RenderFrame<'a> {
    pub width: u32,
    pub height: u32,
    pub buffer: &'a mut [u8],
}

impl<'a> RenderFrame<'a> {
    pub fn pixels(&self) -> impl Iterator<Item = &[u8; PIXEL_BITS]> {
        self.buffer
            .chunks_exact(PIXEL_BITS)
            .map(|chunk| chunk.try_into().unwrap())
    }

    pub fn pixels_mut(&mut self) -> impl Iterator<Item = &mut [u8; PIXEL_BITS]> {
        self.buffer
            .chunks_exact_mut(PIXEL_BITS)
            .map(|chunk| chunk.try_into().unwrap())
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<&[u8; PIXEL_BITS]> {
        let index = (x as usize + y as usize * self.width as usize) * PIXEL_BITS;

        if index + PIXEL_BITS >= self.buffer.len() {
            return None;
        }

        Some(
            self.buffer[index..index + PIXEL_BITS]
                .as_ref()
                .try_into()
                .unwrap(),
        )
    }

    pub fn pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut [u8; PIXEL_BITS]> {
        let index = (x as usize + y as usize * self.width as usize) * PIXEL_BITS;

        if index + PIXEL_BITS >= self.buffer.len() {
            return None;
        }

        Some(
            self.buffer[index..index + PIXEL_BITS]
                .as_mut()
                .try_into()
                .unwrap(),
        )
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: [u8; PIXEL_BITS]) {
        if let Some(pixel) = self.pixel_mut(x, y) {
            *pixel = color;
        }
    }

    pub fn draw_square(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: [u8; PIXEL_BITS],
    ) {
        for y in y..y + height {
            for x in x..x + width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    pub fn fill(&mut self, color: [u8; PIXEL_BITS]) {
        for pixel in self.pixels_mut() {
            *pixel = color;
        }
    }
}
