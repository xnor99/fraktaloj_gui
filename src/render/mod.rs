use num::Complex;

use crate::iteration_image::IterationImage;

pub mod cpu;
pub mod opencl;

pub trait Renderer<T> {
    type Error;

    fn render(
        &mut self,
        center: Complex<T>,
        horizontal_radius: T,
        max_iterations: u32,
    ) -> Result<IterationImage, Self::Error>;
}
