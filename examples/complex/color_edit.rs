use bevy::{
    ecs::system::Resource,
    prelude::default,
    ui::{self, node_bundles::NodeBundle},
};
use bevy_color::{Hsla, Srgba};
use bevy_reactor::*;
use obsidian_ui::controls::{
    button, gradient_slider, ButtonProps, ButtonVariant, GradientSliderProps,
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
            ui::RepeatedGridTrack::auto(1),
        ])
        .grid_auto_flow(ui::GridAutoFlow::Row);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch);
}

pub fn color_edit(cx: &mut Cx) -> impl View {
    // let state = cx.use_resource::<ColorEditState>();

    Element::<NodeBundle>::new()
        .with_styles(style_color_edit)
        .children((
            Element::<NodeBundle>::new()
                .with_styles(style_mode_selector)
                .children((
                    button.bind(ButtonProps {
                        children: "RGB",
                        variant: cx.create_derived(|cx| {
                            match cx.use_resource::<ColorEditState>().mode {
                                ColorMode::Rgb => ButtonVariant::Selected,
                                _ => ButtonVariant::Default,
                            }
                        }),
                        on_click: Some(cx.create_callback(|cx: &mut Cx| {
                            cx.world_mut().resource_mut::<ColorEditState>().mode = ColorMode::Hsl;
                        })),
                        ..default()
                    }),
                    button.bind(ButtonProps {
                        children: "HSL",
                        variant: cx.create_derived(|cx| {
                            match cx.use_resource::<ColorEditState>().mode {
                                ColorMode::Hsl => ButtonVariant::Selected,
                                _ => ButtonVariant::Default,
                            }
                        }),
                        on_click: Some(cx.create_callback(|cx: &mut Cx| {
                            cx.world_mut().resource_mut::<ColorEditState>().mode = ColorMode::Hsl;
                        })),
                        ..default()
                    }),
                    button.bind(ButtonProps {
                        children: "Recent",
                        variant: cx.create_derived(|cx| {
                            match cx.use_resource::<ColorEditState>().mode {
                                ColorMode::Recent => ButtonVariant::Selected,
                                _ => ButtonVariant::Default,
                            }
                        }),
                        on_click: Some(cx.create_callback(|cx: &mut Cx| {
                            cx.world_mut().resource_mut::<ColorEditState>().mode =
                                ColorMode::Recent;
                        })),
                        ..default()
                    }),
                )),
            Element::<NodeBundle>::new()
                .with_styles(style_sliders)
                .children((
                    gradient_slider.bind(GradientSliderProps {
                        gradient: cx.create_derived(|cx| {
                            let rgb = cx.use_resource::<ColorEditState>().rgb;
                            vec![
                                Srgba::new(0.0, rgb.green, rgb.blue, 1.0),
                                Srgba::new(1.0, rgb.green, rgb.blue, 1.0),
                            ]
                        }),
                        min: Signal::Constant(0.),
                        max: Signal::Constant(255.),
                        value: cx.create_derived(|cx| cx.use_resource::<ColorEditState>().rgb.red),
                        style: StyleHandle::new(style_slider),
                        precision: 1,
                        on_change: Some(cx.create_callback(move |cx| {
                            cx.world_mut().resource_mut::<ColorEditState>().rgb.red = cx.props;
                        })),
                        ..default()
                    }),
                    // gradient_slider.bind(GradientSliderProps {
                    //     gradient: Signal::Constant(vec![
                    //         Srgba::new(0.5, 0.5, 0.5, 1.0),
                    //         Srgba::new(1.0, 0.0, 0.0, 1.0),
                    //     ]),
                    //     min: Signal::Constant(0.),
                    //     max: Signal::Constant(255.),
                    //     value: saturation.signal(),
                    //     style: StyleHandle::new(style_slider),
                    //     precision: 1,
                    //     on_change: Some(cx.create_callback(move |cx| {
                    //         saturation.set(cx, cx.props);
                    //     })),
                    //     ..default()
                    // }),
                    // gradient_slider.bind(GradientSliderProps {
                    //     gradient: Signal::Constant(vec![
                    //         Srgba::from(Hsla::new(0.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(60.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(120.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(180.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(240.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(300.0, 1.0, 0.5, 1.0)),
                    //         Srgba::from(Hsla::new(360.0, 1.0, 0.5, 1.0)),
                    //     ]),
                    //     min: Signal::Constant(0.),
                    //     max: Signal::Constant(255.),
                    //     value: saturation.signal(),
                    //     style: StyleHandle::new(style_slider),
                    //     precision: 1,
                    //     on_change: Some(cx.create_callback(move |cx| {
                    //         saturation.set(cx, cx.props);
                    //     })),
                    //     ..default()
                    // }),
                    // gradient_slider.bind(GradientSliderProps {
                    //     gradient: Signal::Constant(vec![
                    //         Srgba::new(0.5, 0.5, 0.5, 0.0),
                    //         Srgba::new(0.5, 0.5, 0.5, 1.5),
                    //     ]),
                    //     min: Signal::Constant(0.),
                    //     max: Signal::Constant(255.),
                    //     value: saturation.signal(),
                    //     style: StyleHandle::new(style_slider),
                    //     precision: 1,
                    //     on_change: Some(cx.create_callback(move |cx| {
                    //         saturation.set(cx, cx.props);
                    //     })),
                    //     ..default()
                    // }),
                )),
        ))
}
