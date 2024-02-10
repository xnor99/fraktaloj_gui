#![feature(never_type)]

use std::{
    num::NonZeroUsize,
    process::ExitCode,
    time::{Duration, Instant},
};

use error::{FatalError, SdlError};
use iteration_image::IterationImage;
use num::{Complex, Zero};
use render::Renderer;
use sdl2::{
    event::Event,
    keyboard::Scancode,
    pixels::{Color, PixelFormatEnum},
    rwops::RWops,
    ttf,
};

use crate::{
    error::OpenclError,
    render::{cpu::ScalarCpuRenderer, opencl::OpenclRenderer},
};

mod error;
mod iteration_image;
mod render;

static FONT: &[u8] = include_bytes!("font.ttf");

fn measure_render<T, R: Renderer<T>>(
    renderer: &mut R,
    center: Complex<T>,
    horizontal_radius: T,
    max_iterations: u32,
) -> Result<(IterationImage, Duration), R::Error> {
    let instant = Instant::now();
    let result = renderer.render(center, horizontal_radius, max_iterations);
    let duration = instant.elapsed();
    match result {
        Ok(image) => Ok((image, duration)),
        Err(error) => Err(error),
    }
}

enum RendererChoice {
    Cpu,
    Opencl,
}

fn app() -> Result<(), FatalError> {
    let sdl = sdl2::init().map_err(SdlError::from)?;
    let video = sdl.video().map_err(SdlError::from)?;
    let window = video
        .window("Fraktaloj GUI", 1920, 1080)
        .build()
        .map_err(SdlError::from)?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(SdlError::from)?;
    let mut events = sdl.event_pump().map_err(SdlError::from)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(Some(PixelFormatEnum::RGB24), 1920, 1080)
        .map_err(SdlError::from)?;

    let ttf_context = ttf::init().map_err(SdlError::from)?;
    let font = ttf_context
        .load_font_from_rwops(RWops::from_bytes(FONT).map_err(SdlError::from)?, 48)
        .map_err(SdlError::from)?;

    let mut cpu_renderer = ScalarCpuRenderer::new(
        NonZeroUsize::new(1920).unwrap(),
        NonZeroUsize::new(1080).unwrap(),
    )?;

    let mut opencl_renderer = OpenclRenderer::new(
        NonZeroUsize::new(1920).unwrap(),
        NonZeroUsize::new(1080).unwrap(),
    )?;

    let opencl_display_string = format!(
        "OpenCL: {}",
        opencl_renderer.device_name().map_err(OpenclError::from)?
    );

    let mut renderer_choice = RendererChoice::Cpu;
    let mut max_iterations = 256_u32;

    const ZOOM_MULTIPLIER: f64 = 1.25;
    const ZOOM_MULTIPLIER_INV: f64 = 1.0 / ZOOM_MULTIPLIER;

    let mut center = Complex::zero();
    let mut radius = 2.0;

    'main_loop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    scancode: Some(Scancode::Up),
                    ..
                } => center.im += 0.1 * radius,
                Event::KeyDown {
                    scancode: Some(Scancode::Left),
                    ..
                } => center.re -= 0.1 * radius,
                Event::KeyDown {
                    scancode: Some(Scancode::Down),
                    ..
                } => center.im -= 0.1 * radius,
                Event::KeyDown {
                    scancode: Some(Scancode::Right),
                    ..
                } => center.re += 0.1 * radius,
                Event::KeyDown {
                    scancode: Some(Scancode::PageUp),
                    ..
                } => radius *= ZOOM_MULTIPLIER_INV,
                Event::KeyDown {
                    scancode: Some(Scancode::PageDown),
                    ..
                } => radius *= ZOOM_MULTIPLIER,
                Event::KeyDown {
                    scancode: Some(Scancode::Home),
                    ..
                } => {
                    center = Complex::zero();
                    radius = 2.0;
                }
                Event::KeyDown {
                    scancode: Some(Scancode::Num1),
                    ..
                } => renderer_choice = RendererChoice::Cpu,
                Event::KeyDown {
                    scancode: Some(Scancode::Num2),
                    ..
                } => renderer_choice = RendererChoice::Opencl,
                Event::KeyDown {
                    scancode: Some(Scancode::Period),
                    ..
                } => {
                    if let Some(new_value) = max_iterations.checked_mul(2) {
                        max_iterations = new_value;
                    }
                }
                Event::KeyDown {
                    scancode: Some(Scancode::Comma),
                    ..
                } => {
                    if let Some(new_value @ 1..) = max_iterations.checked_div(2) {
                        max_iterations = new_value;
                    }
                }
                _ => (),
            }
        }

        canvas.clear();
        let (image, duration) = match renderer_choice {
            RendererChoice::Cpu => {
                measure_render(&mut cpu_renderer, center, radius, max_iterations).unwrap()
            }
            RendererChoice::Opencl => {
                measure_render(&mut opencl_renderer, center, radius, max_iterations)
                    .map_err(OpenclError::from)?
            }
        };
        image
            .write_to_texture(&mut texture)
            .map_err(SdlError::from)?;
        canvas.copy(&texture, None, None).map_err(SdlError::from)?;
        let text = font
            .render(&format!(
                "{}\nTime to render: {:.2} ms\nMax iterations: {max_iterations}",
                match renderer_choice {
                    RendererChoice::Cpu => "Multithreaded Scalar CPU",
                    RendererChoice::Opencl => &opencl_display_string,
                },
                duration.as_secs_f64() * 1e3
            ))
            .blended_wrapped(Color::RED, 0)
            .map_err(SdlError::from)?;
        let rect = {
            let mut rect = text.rect();
            rect.offset(16, 16);
            rect
        };
        let text_texture = texture_creator
            .create_texture_from_surface(text)
            .map_err(SdlError::from)?;
        canvas
            .copy(&text_texture, None, rect)
            .map_err(SdlError::from)?;
        canvas.present();
    }
    Ok(())
}

fn main() -> ExitCode {
    // TODO: implement error handling
    app().unwrap();
    ExitCode::SUCCESS
}
