use std::num::NonZeroUsize;

use num::{traits::float::FloatCore, Complex, Zero};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{error::IntegerOverflow, iteration_image::IterationImage};

use super::Renderer;

pub struct ScalarCpuRenderer {
    width: NonZeroUsize,
    height: NonZeroUsize,
    iteration_image_buffer: Vec<u32>,
}

impl ScalarCpuRenderer {
    pub fn new(width: NonZeroUsize, height: NonZeroUsize) -> Result<Self, IntegerOverflow> {
        Ok(Self {
            width,
            height,
            iteration_image_buffer: vec![
                0;
                width.checked_mul(height).ok_or(IntegerOverflow)?.get()
            ],
        })
    }
}

fn compose<A, B, C, F: Fn(A) -> B, G: Fn(B) -> C>(f: F, g: G) -> impl Fn(A) -> C {
    move |a| g(f(a))
}

fn lerp<T: FloatCore>(t: T, min: T, max: T) -> T {
    t * (max - min) + min
}

fn lerp_inverse<T: FloatCore>(t: T, min: T, max: T) -> T {
    (t - min) / (max - min)
}

fn map<T: FloatCore>(value: T, from_source: T, to_source: T, from_target: T, to_target: T) -> T {
    lerp(
        lerp_inverse(value, from_source, to_source),
        from_target,
        to_target,
    )
}

impl<T: FloatCore + Send + Sync> Renderer<T> for ScalarCpuRenderer {
    type Error = !;

    fn render(
        &mut self,
        center: Complex<T>,
        horizontal_radius: T,
        max_iterations: u32,
    ) -> Result<IterationImage, Self::Error> {
        let width_float = T::from(self.width.get()).unwrap_or(T::infinity());
        let height_float = T::from(self.height.get()).unwrap_or(T::infinity());
        let vertical_radius = horizontal_radius * height_float / width_float;

        let max_offset = Complex::new(horizontal_radius, vertical_radius);
        let bottom_left = center - max_offset;
        let top_right = center + max_offset;

        let four = T::from(4).unwrap();
        self.iteration_image_buffer
            .par_iter_mut()
            .enumerate()
            .map(compose(
                |(i, pixel)| {
                    (
                        i % self.width,
                        self.height.get() - i / self.width - 1,
                        pixel,
                    )
                },
                |(x, y, pixel)| {
                    (
                        Complex::new(
                            map(
                                T::from(x).unwrap_or(T::infinity()),
                                T::zero(),
                                width_float,
                                bottom_left.re,
                                top_right.re,
                            ),
                            map(
                                T::from(y).unwrap_or(T::infinity()),
                                T::zero(),
                                height_float,
                                bottom_left.im,
                                top_right.im,
                            ),
                        ),
                        pixel,
                    )
                },
            ))
            .for_each(|(c, pixel)| {
                let mut z = Complex::<T>::zero();
                let mut iteration = 0;
                while iteration < max_iterations && z.norm_sqr() < four {
                    z = z * z + c;
                    iteration += 1;
                }
                *pixel = iteration;
            });

        // Buffer size was already calculated
        Ok(IterationImage::from_buffer(
            self.iteration_image_buffer.clone(),
            self.width,
            max_iterations,
        )
        .unwrap())
    }
}
