// This shader draws a rounded rect with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0)
var<uniform> num_color_stops: i32;

@group(1) @binding(1)
var<uniform> color_stops: array<vec4<f32>, 8>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x * f32(num_color_stops - 1);
    let color_index_lo = clamp(i32(floor(t)), 0, num_color_stops - 1);
    let color_index_hi = clamp(i32(ceil(t)), 0, num_color_stops - 1);
    let color_lo = color_stops[color_index_lo];
    let color_hi = color_stops[color_index_hi];
    let color = mix(color_lo, color_hi, t - f32(color_index_lo));

    return vec4<f32>(
        srgb_to_linear(color.rgb),
        color.w);
}

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let a = 0.055;
    let srgbLow = srgb / 12.92;
    let srgbHigh = pow((srgb + a) / (1.0 + a), vec3<f32>(2.4, 2.4, 2.4));
    let linear = mix(srgbLow, srgbHigh, step(vec3<f32>(0.04045, 0.04045, 0.04045), srgb));
    return linear;
}
