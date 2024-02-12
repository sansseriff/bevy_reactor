use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

/// Options for rendering rounded corners.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Default)]
pub enum RoundedCorners {
    None,
    #[default]
    All,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    Top,
    Right,
    Bottom,
    Left,
}

impl RoundedCorners {
    /// Convert the `RoundedCorners` to a `Vec4` for use in a shader.
    pub fn to_vec(&self, radius: f32) -> Vec4 {
        match self {
            RoundedCorners::None => Vec4::new(0.0, 0.0, 0.0, 0.0),
            RoundedCorners::All => Vec4::new(radius, radius, radius, radius),
            RoundedCorners::TopLeft => Vec4::new(radius, 0.0, 0.0, 0.0),
            RoundedCorners::TopRight => Vec4::new(0.0, radius, 0.0, 0.0),
            RoundedCorners::BottomRight => Vec4::new(0.0, 0.0, radius, 0.0),
            RoundedCorners::BottomLeft => Vec4::new(0.0, 0.0, 0.0, radius),
            RoundedCorners::Top => Vec4::new(radius, radius, 0.0, 0.0),
            RoundedCorners::Right => Vec4::new(0.0, radius, radius, 0.0),
            RoundedCorners::Bottom => Vec4::new(0.0, 0.0, radius, radius),
            RoundedCorners::Left => Vec4::new(radius, 0.0, 0.0, radius),
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct RoundedRectMaterial {
    #[uniform(0)]
    pub(crate) color: Vec4,
    #[uniform(1)]
    pub(crate) radius: Vec4, // TopLeft, TopRight, BottomRight, BottomLeft
}

impl UiMaterial for RoundedRectMaterial {
    fn fragment_shader() -> ShaderRef {
        "obsidian_ui://shaders/rounded_rect.wgsl".into()
    }
}
