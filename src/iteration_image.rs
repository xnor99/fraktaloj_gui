use std::num::NonZeroUsize;

use sdl2::render::Texture;

use crate::error::{InvalidBufferSize, WriteToTextureError};

pub struct IterationImage {
    buffer: Vec<u32>,
    width: NonZeroUsize,
    max_iterations: u32,
}

impl IterationImage {
    pub fn from_buffer(
        buffer: Vec<u32>,
        width: NonZeroUsize,
        max_iterations: u32,
    ) -> Result<Self, InvalidBufferSize> {
        if buffer.is_empty() || buffer.len() % width.get() != 0 {
            return Err(InvalidBufferSize);
        }
        Ok(Self {
            buffer,
            width,
            max_iterations,
        })
    }

    pub fn write_to_texture(&self, texture: &mut Texture) -> Result<(), WriteToTextureError> {
        let query = texture.query();
        let height = self.buffer.len() / self.width.get();
        if self.width.get() != query.width as usize || height != query.height as usize {
            return Err(WriteToTextureError::DimensionsDoNotMatch);
        }
        let max_iterations = f64::from(self.max_iterations);
        let texture_data = self
            .buffer
            .iter()
            .flat_map(|&iter_count| {
                let t = (f64::from(iter_count) * 255.0 / max_iterations) as u8;
                [t; 3]
            })
            .collect::<Vec<_>>();
        texture.update(None, &texture_data, self.width.get() * 3)?;
        Ok(())
    }
}
