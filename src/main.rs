#![feature(never_type)]

use std::{num::NonZeroUsize, process::ExitCode};

use error::{FatalError, SdlError};
use num::{Complex, Zero};
use render::Renderer;
use sdl2::{event::Event, keyboard::Scancode, pixels::PixelFormatEnum};

use crate::render::opencl::OpenclRenderer;

mod error;
mod iteration_image;
mod render;

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

    // let mut renderer = ScalarCpuRenderer::new(
    //     NonZeroUsize::new(1920).unwrap(),
    //     NonZeroUsize::new(1080).unwrap(),
    // )?;

    let mut renderer = OpenclRenderer::new(
        NonZeroUsize::new(1920).unwrap(),
        NonZeroUsize::new(1080).unwrap(),
    )?;

    const ZOOM_MULTIPLIER: f32 = 1.25;
    const ZOOM_MULTIPLIER_INV: f32 = 1.0 / ZOOM_MULTIPLIER;

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
                _ => (),
            }
        }

        canvas.clear();
        let image = renderer.render(center, radius, 64).unwrap();
        image
            .write_to_texture(&mut texture)
            .map_err(SdlError::from)?;
        canvas
            .copy(&texture, None, None)
            .map_err(SdlError::Message)?;
        canvas.present();
    }
    Ok(())
}

fn main() -> ExitCode {
    // TODO: implement error handling
    app().unwrap();
    ExitCode::SUCCESS
}
