use bevy::{
    color::{Hsla, Srgba},
    ecs::system::Resource,
    ui::{self, node_bundles::NodeBundle},
};
use bevy_reactor::*;
use obsidian_ui::{
    controls::{Button, ButtonVariant, ColorGradient, GradientSlider},
    RoundedCorners,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    Rgb,
    Hsl,
    Recent,
}

#[derive(Default, Resource)]
pub struct ColorEditState {
    pub mode: ColorMode,
    pub rgb: Srgba,
    pub hsl: Hsla,
    pub recent: Vec<Srgba>,
}

fn style_color_edit(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .align_items(ui::AlignItems::Stretch)
        .flex_direction(ui::FlexDirection::Column)
        .gap(4);
}

fn style_mode_selector(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .gap(1);
}

fn style_sliders(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Grid)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::fr(1, 1.),
            ui::RepeatedGridTrack::px(1, 32.),
        ])
        .grid_auto_flow(ui::GridAutoFlow::Row)
        .align_items(ui::AlignItems::Center)
        .row_gap(4)
        .column_gap(4);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch);
}

pub struct ColorEdit;

impl ViewTemplate for ColorEdit {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        // let state = cx.use_resource::<ColorEditState>();

        let mode = cx.create_memo(|cx| cx.use_resource::<ColorEditState>().mode);

        Element::<NodeBundle>::new()
            .style(style_color_edit)
            .children((
                Element::<NodeBundle>::new()
                    .style(style_mode_selector)
                    .children((
                        Button::new()
                            .children("RGB")
                            .corners(RoundedCorners::Left)
                            .variant_signal(cx.create_derived(|cx| {
                                match cx.use_resource::<ColorEditState>().mode {
                                    ColorMode::Rgb => ButtonVariant::Selected,
                                    _ => ButtonVariant::Default,
                                }
                            }))
                            .on_click(cx.create_callback(|cx: &mut Cx, ()| {
                                cx.world_mut().resource_mut::<ColorEditState>().mode =
                                    ColorMode::Rgb;
                            })),
                        Button::new()
                            .children("HSL")
                            .corners(RoundedCorners::None)
                            .variant_signal(cx.create_derived(|cx| {
                                match cx.use_resource::<ColorEditState>().mode {
                                    ColorMode::Hsl => ButtonVariant::Selected,
                                    _ => ButtonVariant::Default,
                                }
                            }))
                            .on_click(cx.create_callback(|cx: &mut Cx, ()| {
                                cx.world_mut().resource_mut::<ColorEditState>().mode =
                                    ColorMode::Hsl;
                            })),
                        Button::new()
                            .children("Recent")
                            .corners(RoundedCorners::Right)
                            .variant_signal(cx.create_derived(|cx| {
                                match cx.use_resource::<ColorEditState>().mode {
                                    ColorMode::Recent => ButtonVariant::Selected,
                                    _ => ButtonVariant::Default,
                                }
                            }))
                            .on_click(cx.create_callback(|cx: &mut Cx, ()| {
                                cx.world_mut().resource_mut::<ColorEditState>().mode =
                                    ColorMode::Recent;
                            })),
                    )),
                DynamicKeyed::new(
                    move |cx| mode.get(cx),
                    |mode| match mode {
                        ColorMode::Rgb => RgbSliders.to_view_ref(),
                        ColorMode::Hsl => HslSliders.to_view_ref(),
                        ColorMode::Recent => TextStatic::new("Recent".to_string()).into(),
                    },
                ),
            ))
    }
}

struct RgbSliders;

impl ViewTemplate for RgbSliders {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new().style(style_sliders).children((
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let rgb = cx.use_resource::<ColorEditState>().rgb;
                    ColorGradient::new(&[
                        Srgba::new(0.0, rgb.green, rgb.blue, 1.0),
                        Srgba::new(1.0, rgb.green, rgb.blue, 1.0),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(255.))
                .value(cx.create_derived(|cx| cx.use_resource::<ColorEditState>().rgb.red * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut().resource_mut::<ColorEditState>().rgb.red = value / 255.0;
                })),
            text_computed(|cx| {
                format!("{:.0}", cx.use_resource::<ColorEditState>().rgb.red * 255.0)
            }),
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let rgb = cx.use_resource::<ColorEditState>().rgb;
                    ColorGradient::new(&[
                        Srgba::new(rgb.red, 0.0, rgb.blue, 1.0),
                        Srgba::new(rgb.red, 1.0, rgb.blue, 1.0),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(255.))
                .value(
                    cx.create_derived(|cx| cx.use_resource::<ColorEditState>().rgb.green * 255.0),
                )
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut().resource_mut::<ColorEditState>().rgb.green = value / 255.0;
                })),
            text_computed(|cx| {
                format!(
                    "{:.0}",
                    cx.use_resource::<ColorEditState>().rgb.green * 255.0
                )
            }),
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let rgb = cx.use_resource::<ColorEditState>().rgb;
                    ColorGradient::new(&[
                        Srgba::new(rgb.red, rgb.green, 0.0, 1.0),
                        Srgba::new(rgb.red, rgb.green, 1.0, 1.0),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(255.))
                .value(cx.create_derived(|cx| cx.use_resource::<ColorEditState>().rgb.blue * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut().resource_mut::<ColorEditState>().rgb.blue = value / 255.0;
                })),
            text_computed(|cx| {
                format!(
                    "{:.0}",
                    cx.use_resource::<ColorEditState>().rgb.blue * 255.0
                )
            }),
            AlphaSlider,
        ))
    }
}

struct HslSliders;

impl ViewTemplate for HslSliders {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new().style(style_sliders).children((
            GradientSlider::new()
                .gradient(Signal::Constant(ColorGradient::new(&[
                    Srgba::from(Hsla::new(0.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(60.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(120.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(180.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(240.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(300.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(360.0, 1.0, 0.5, 1.0)),
                ])))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(360.))
                .value(cx.create_derived(|cx| cx.use_resource::<ColorEditState>().hsl.hue * 360.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut().resource_mut::<ColorEditState>().hsl.hue = value / 360.0;
                })),
            text_computed(|cx| {
                format!("{:.0}", cx.use_resource::<ColorEditState>().hsl.hue * 360.0)
            }),
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let hsl = cx.use_resource::<ColorEditState>().hsl;
                    ColorGradient::new(&[
                        Srgba::from(Hsla::new(hsl.hue, 0.0, hsl.lightness, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, 0.5, hsl.lightness, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, 1.0, hsl.lightness, 1.0)),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(100.))
                .value(cx.create_derived(|cx| {
                    cx.use_resource::<ColorEditState>().hsl.saturation * 100.0
                }))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut()
                        .resource_mut::<ColorEditState>()
                        .hsl
                        .saturation = value / 100.0;
                })),
            text_computed(|cx| {
                format!(
                    "{:.0}",
                    cx.use_resource::<ColorEditState>().hsl.saturation * 100.0
                )
            }),
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let hsl = cx.use_resource::<ColorEditState>().hsl;
                    ColorGradient::new(&[
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 0.0, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 0.5, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 1.0, 1.0)),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(100.))
                .value(
                    cx.create_derived(|cx| {
                        cx.use_resource::<ColorEditState>().hsl.lightness * 100.0
                    }),
                )
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut()
                        .resource_mut::<ColorEditState>()
                        .hsl
                        .lightness = value / 100.0;
                })),
            text_computed(|cx| {
                format!(
                    "{:.0}",
                    cx.use_resource::<ColorEditState>().hsl.lightness * 100.0
                )
            }),
            AlphaSlider,
        ))
    }
}

struct AlphaSlider;

impl ViewTemplate for AlphaSlider {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        Fragment::new((
            GradientSlider::new()
                .gradient(cx.create_derived(|cx| {
                    let rgb = cx.use_resource::<ColorEditState>().rgb;
                    ColorGradient::new(&[
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 0.0),
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 0.2),
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 0.4),
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 0.6),
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 0.8),
                        Srgba::new(rgb.red, rgb.green, rgb.blue, 1.0),
                    ])
                }))
                .min(Signal::Constant(0.))
                .max(Signal::Constant(255.))
                .value(
                    cx.create_derived(|cx| cx.use_resource::<ColorEditState>().rgb.alpha * 255.0),
                )
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.world_mut().resource_mut::<ColorEditState>().rgb.alpha = value / 255.0;
                })),
            text_computed(|cx| {
                format!(
                    "{:.0}",
                    cx.use_resource::<ColorEditState>().rgb.alpha * 255.0
                )
            }),
        ))
    }
}
