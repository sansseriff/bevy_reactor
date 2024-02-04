// This shader draws a rounded rect with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

// struct RoundedRectMaterial {
//     @location(0) color: vec4<f32>,
//     // @location(1) @interpolate(flat) radius: vec4<f32>
// }

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@group(1) @binding(1)
var<uniform> radius: vec4<f32>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // the UVs are now adjusted around the middle of the rect.
    let uv = in.uv * 2.0 - 1.0;

    // circle alpha, the higher the power the harsher the falloff.
    let alpha = 1.0 - pow(sqrt(dot(uv, uv)), 100.0);

    return vec4<f32>(color.rgb, alpha);
}
