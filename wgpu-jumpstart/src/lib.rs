#![allow(clippy::single_match)]

mod error;
pub use error::GpuInitError;

mod color;
mod gpu;
mod instances;
mod render_pipeline_builder;
mod shape;
mod uniform;

mod transform_uniform;

pub use wgpu;
pub use {
    color::Color,
    gpu::{Gpu, Surface},
    instances::Instances,
    render_pipeline_builder::RenderPipelineBuilder,
    shape::Shape,
    transform_uniform::TransformUniform,
    uniform::Uniform,
};

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
