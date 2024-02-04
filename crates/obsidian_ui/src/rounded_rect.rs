use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

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
