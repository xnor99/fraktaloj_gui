use std::{num::NonZeroUsize, ptr};

use num::Complex;
use opencl3::{
    command_queue::CommandQueue,
    context::Context,
    device::{self, Device, CL_DEVICE_TYPE_GPU},
    error_codes::ClError,
    kernel::{ExecuteKernel, Kernel},
    memory::Buffer,
    program::Program,
    types::CL_NON_BLOCKING,
};

use crate::{error::OpenclError, iteration_image::IterationImage};

use super::Renderer;

static KERNEL_SOURCE: &str = include_str!("kernel.cl");

pub struct OpenclRenderer {
    width: NonZeroUsize,
    height: NonZeroUsize,
    queue: CommandQueue,
    kernel: Kernel,
    buffer: Buffer<u32>,
}

impl OpenclRenderer {
    pub fn new(width: NonZeroUsize, height: NonZeroUsize) -> Result<Self, OpenclError> {
        let device = Device::new(
            *device::get_all_devices(CL_DEVICE_TYPE_GPU)?
                .first()
                .ok_or(OpenclError::NoDevices)?,
        );
        let context = Context::from_device(&device)?;
        let queue = CommandQueue::create_default_with_properties(&context, 0, 0)?;
        let program = Program::create_and_build_from_source(&context, KERNEL_SOURCE, "")?;
        let kernel = Kernel::create(&program, "render")?;
        let buffer = unsafe {
            Buffer::<u32>::create(
                &context,
                0,
                width
                    .checked_mul(height)
                    .ok_or(OpenclError::IntegerOverflow)?
                    .get(),
                ptr::null_mut(),
            )
        }?;
        Ok(Self {
            width,
            height,
            queue,
            kernel,
            buffer,
        })
    }
}

impl Renderer<f32> for OpenclRenderer {
    type Error = ClError;

    fn render(
        &mut self,
        center: Complex<f32>,
        horizontal_radius: f32,
        max_iterations: u32,
    ) -> Result<IterationImage, Self::Error> {
        // Shouldn't overflow because the check was done during object construction
        let work_size = self.width.get() * self.height.get();

        let kernel_event = unsafe {
            ExecuteKernel::new(&self.kernel)
                .set_arg(&(self.width.get() as u64))
                .set_arg(&center.re)
                .set_arg(&center.im)
                .set_arg(&horizontal_radius)
                .set_arg(&max_iterations)
                .set_arg(&self.buffer)
                .set_global_work_size(work_size)
                .enqueue_nd_range(&self.queue)
        }?;

        let mut iteration_image_buffer = vec![0; work_size];
        let read_buffer_event = unsafe {
            self.queue.enqueue_read_buffer(
                &self.buffer,
                CL_NON_BLOCKING,
                0,
                &mut iteration_image_buffer,
                &[kernel_event.get()],
            )
        }?;
        read_buffer_event.wait()?;

        // Buffer size was already calculated
        Ok(
            IterationImage::from_buffer(iteration_image_buffer, self.width, max_iterations)
                .unwrap(),
        )
    }
}
