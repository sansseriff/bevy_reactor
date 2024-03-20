use bevy::{prelude::*, ui};
use bevy_color::Srgba;
use bevy_mod_picking::prelude::*;
use bevy_reactor::*;
// use bevy_tabindex::TabIndex;

use crate::{colors, size::Size};

/// Button properties
#[derive(Default)]
pub struct SwatchProps {
    /// Color to display
    pub color: Signal<Srgba>,

    /// For swatch grids, whether this swatch is selected.
    pub selected: Signal<bool>,

    /// Swatch vertical size.
    pub size: Size,

    /// Additional styles to be applied to the button.
    pub styles: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback>,
}

fn style_swatch(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .padding((12, 0))
        .border(0)
        .color(colors::FOREGROUND);
}

/// Color swatch widget.
pub struct Swatch(SwatchProps);

impl Swatch {
    /// Create a new swatch.
    pub fn new(props: SwatchProps) -> Self {
        Self(props)
    }
}

impl Widget for Swatch {
    type View = Element<NodeBundle>;

    fn create(&self, _cx: &mut Cx) -> Element<NodeBundle> {
        let color = self.0.color;
        let size = self.0.size;

        Element::<NodeBundle>::new()
            .named("color_swatch")
            .with_styles((
                style_swatch,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height());
                },
                self.0.styles.clone(),
            ))
            .insert((
                // TabIndex(0),
                // AccessibilityNode::from(NodeBuilder::new(Role::Button)),
                {
                    let on_click = self.0.on_click;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        if let Some(on_click) = on_click {
                            world.run_callback(on_click, ());
                        }
                    })
                },
            ))
            .create_effect(move |cx, ent| {
                let color = color.get(cx);
                let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                bg.0 = color.into();
            })
    }
}
