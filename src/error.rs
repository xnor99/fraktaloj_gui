use std::num::TryFromIntError;

use opencl3::error_codes::ClError;
use sdl2::{
    render::{TextureValueError, UpdateTextureError},
    ttf::{FontError, InitError},
    video::WindowBuildError,
    IntegerOrSdlError,
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum SdlError {
    Message(String),
    WindowBuildError(WindowBuildError),
    IntegerOrSdlError(IntegerOrSdlError),
    WriteToTextureError(WriteToTextureError),
    TextureValueError(TextureValueError),
    TtfInitError(InitError),
    FontError(FontError),
}

impl From<String> for SdlError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<WindowBuildError> for SdlError {
    fn from(value: WindowBuildError) -> Self {
        Self::WindowBuildError(value)
    }
}

impl From<IntegerOrSdlError> for SdlError {
    fn from(value: IntegerOrSdlError) -> Self {
        Self::IntegerOrSdlError(value)
    }
}

impl From<WriteToTextureError> for SdlError {
    fn from(value: WriteToTextureError) -> Self {
        Self::WriteToTextureError(value)
    }
}

impl From<TextureValueError> for SdlError {
    fn from(value: TextureValueError) -> Self {
        Self::TextureValueError(value)
    }
}

impl From<InitError> for SdlError {
    fn from(value: InitError) -> Self {
        Self::TtfInitError(value)
    }
}

impl From<FontError> for SdlError {
    fn from(value: FontError) -> Self {
        Self::FontError(value)
    }
}

#[derive(Debug)]
pub struct IntegerOverflow;

impl From<TryFromIntError> for IntegerOverflow {
    fn from(_value: TryFromIntError) -> Self {
        Self
    }
}

#[derive(Debug)]
pub enum FatalError {
    SdlError(SdlError),
    OpenclError(OpenclError),
    IntegerOverflow,
}

impl From<SdlError> for FatalError {
    fn from(value: SdlError) -> Self {
        Self::SdlError(value)
    }
}

impl From<OpenclError> for FatalError {
    fn from(value: OpenclError) -> Self {
        Self::OpenclError(value)
    }
}

impl From<IntegerOverflow> for FatalError {
    fn from(_value: IntegerOverflow) -> Self {
        Self::IntegerOverflow
    }
}

impl From<TryFromIntError> for FatalError {
    fn from(_value: TryFromIntError) -> Self {
        Self::IntegerOverflow
    }
}

#[derive(Debug)]
pub enum WriteToTextureError {
    DimensionsDoNotMatch,
    UpdateTextureError(UpdateTextureError),
}

impl From<UpdateTextureError> for WriteToTextureError {
    fn from(value: UpdateTextureError) -> Self {
        Self::UpdateTextureError(value)
    }
}

#[derive(Debug)]
pub struct InvalidBufferSize;

#[derive(Debug)]
pub enum OpenclError {
    ClError(ClError),
    NoDevices,
    CompileError(String),
    IntegerOverflow,
}

impl From<ClError> for OpenclError {
    fn from(value: ClError) -> Self {
        Self::ClError(value)
    }
}

impl From<String> for OpenclError {
    fn from(value: String) -> Self {
        Self::CompileError(value)
    }
}
