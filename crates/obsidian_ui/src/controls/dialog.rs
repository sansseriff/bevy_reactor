use bevy::{prelude::*, ui};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
};
use bevy_reactor::*;

use crate::{
    colors,
    hooks::{BistableTransitionState, CreateBistableTransition},
};

// Dialog background overlay
fn style_dialog_overlay(ss: &mut StyleBuilder) {
    ss.position(PositionType::Absolute)
        .display(ui::Display::Flex)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .left(0)
        .top(0)
        .right(0)
        .bottom(0)
        .z_index(100)
        .background_color("#222c");
}

fn style_dialog(ss: &mut StyleBuilder) {
    ss.background_color("#333")
        .position(PositionType::Relative)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Stretch)
        .border_color(colors::U1)
        .width(200)
        .border(2);
    // .scale(0.5)
    // .transition(&[Transition {
    //     property: TransitionProperty::Transform,
    //     duration: 0.3,
    //     timing: timing::EASE_IN_OUT,
    //     ..default()
    // }])
    // .selector(".entering > &,.entered > &", |ss| ss.scale(1.));
}

/// Dialog box properties.
#[derive(Clone, Default)]
pub struct DialogProps {
    /// The width of the dialog, one of several standard widths.
    pub width: ui::Val,

    /// Signal that controls whether the dialog is open. Note that when this becomes false,
    /// the dialog will still remain visible until it completes its closing animation.
    pub open: Signal<bool>,

    /// The content of the dialog.
    pub children: ViewHandle,

    /// Callback called when the dialog's close button is clicked.
    pub on_close: Option<Callback>,

    /// Callback called when the dialog has completed it's closing animation.
    pub on_exited: Option<Callback>,
}

/// Displays a modal dialog box. This will display the dialog frame and the backdrop overlay.
/// Use the dialog header/body/footer controls to get the standard layout.
pub fn dialog(cx: &mut Cx<DialogProps>) -> impl View {
    let on_close = cx.props.on_close;
    let on_exited = cx.props.on_exited;
    let state = cx.create_bistable_transition(cx.props.open, 0.3);
    let children = cx.props.children.clone();

    cx.create_effect(move |ve| {
        let state = state.get(ve);
        println!("state: {:?}", state);
        if state == BistableTransitionState::Exited {
            if let Some(on_exited) = on_exited {
                ve.run_callback(on_exited, ());
            }
        }
    });

    Cond::new(
        move |cx| state.get(cx) != BistableTransitionState::Exited,
        move || {
            Portal::new(
                Element::<NodeBundle>::new()
                    .with_styles(style_dialog_overlay)
                    .insert(
                        // Click on backdrop sends close signal.
                        On::<Pointer<Click>>::run(move |world: &mut World| {
                            if let Some(on_close) = on_close {
                                world.run_callback(on_close, ());
                            }
                        }),
                    )
                    // .class_names(state.as_class_name())
                    .children(
                        Element::<NodeBundle>::new()
                            .with_styles(style_dialog)
                            .with_child(&children),
                    ),
            )
        },
        || (),
    )
}

fn style_dialog_header(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::SpaceBetween)
        .border_color("#0008")
        .border_bottom(1)
        .padding((12, 6));
}

/// Dialog header properties.
#[derive(Clone, Default)]
pub struct DialogHeaderProps {
    /// The content of the dialog header.
    pub children: ViewHandle,
}

/// Displays a standard dialog header.
pub fn dialog_header(cx: &mut Cx<DialogHeaderProps>) -> impl View {
    Element::<NodeBundle>::new()
        .with_styles(style_dialog_header)
        .with_child(&cx.props.children)
}

fn style_dialog_body(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch)
        .justify_content(ui::JustifyContent::FlexStart)
        .padding((12, 6))
        .min_height(200);
}

/// Dialog body properties.
#[derive(Clone, Default)]
pub struct DialogBodyProps {
    /// The content of the dialog header.
    pub children: ViewHandle,
}

/// Displays a standard dialog body.
pub fn dialog_body(cx: &mut Cx<DialogBodyProps>) -> impl View {
    Element::<NodeBundle>::new()
        .with_styles(style_dialog_body)
        .with_child(&cx.props.children)
}

fn style_dialog_footer(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexEnd)
        .align_items(ui::AlignItems::Center)
        .border_color("#0008")
        .border_top(1)
        .column_gap(4)
        .padding((8, 6));
}

/// Dialog header properties.
#[derive(Clone, Default)]
pub struct DialogFooterProps {
    /// The content of the dialog header.
    pub children: ViewHandle,
}

/// Displays a standard dialog footer.
pub fn dialog_footer(cx: &mut Cx<DialogFooterProps>) -> impl View {
    Element::<NodeBundle>::new()
        .with_styles(style_dialog_footer)
        .with_child(&cx.props.children)
}
