use bevy::{
    asset::Asset,
    pbr::{
        ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
        StandardMaterial,
    },
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, CompareFunction, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
};
// use bevy_color::LinearRgba;

/// Material for overlays
// #[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
// struct OverlayMaterial {
//     pub(crate) color: LinearRgba,
// }

// #[allow(unused_variables)]
// impl Material for OverlayMaterial {
//     fn specialize(
//         pipeline: &MaterialPipeline<Self>,
//         descriptor: &mut RenderPipelineDescriptor,
//         layout: &MeshVertexBufferLayout,
//         key: MaterialPipelineKey<Self>,
//     ) -> Result<(), SpecializedMeshPipelineError> {
//         if let Some(ref mut depth_stencil) = descriptor.depth_stencil {
//             depth_stencil.depth_write_enabled = true;
//             depth_stencil.depth_compare = CompareFunction::Less;
//         }
//         Ok(())
//     }
// }

#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct UnderlayExtension {}

#[allow(unused_variables)]
impl MaterialExtension for UnderlayExtension {
    fn specialize(
        pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(ref mut depth_stencil) = descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = true;
            depth_stencil.depth_compare = CompareFunction::Less;
        }
        Ok(())
    }
}

pub type UnderlayMaterial = ExtendedMaterial<StandardMaterial, UnderlayExtension>;
