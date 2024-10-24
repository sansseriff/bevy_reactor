use bevy::{asset::embedded_asset, prelude::*};

pub mod animation;
pub mod colors;
pub mod controls;
pub mod cursor;
pub mod focus_signal;
pub mod hover_signal;
pub mod input_dispatch;
mod materials;
pub mod rounded_corners;
pub mod scrolling;
pub mod size;
pub mod tab_navigation;
pub mod typography;

pub mod prelude {
    pub use crate::colors;
    pub use crate::controls::*;
    pub use crate::focus_signal::CreateFocusSignal;
    pub use crate::hover_signal::CreateHoverSignal;
    pub use crate::rounded_corners::RoundedCorners;
    pub use crate::size::Size;
    pub use crate::tab_navigation::{handle_tab_navigation, TabGroup, TabIndex};
    pub use crate::typography;
    pub use crate::ObsidianUiPlugin;
}

pub struct ObsidianUiPlugin;
use input_dispatch::InputDispatchPlugin;
use materials::{GradientRectMaterial, SliderRectMaterial, SwatchRectMaterial};

impl Plugin for ObsidianUiPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/fonts/Open_Sans/static/OpenSans-Bold.ttf");
        embedded_asset!(app, "assets/fonts/Open_Sans/static/OpenSans-BoldItalic.ttf");
        embedded_asset!(app, "assets/fonts/Open_Sans/static/OpenSans-Medium.ttf");
        embedded_asset!(
            app,
            "assets/fonts/Open_Sans/static/OpenSans-MediumItalic.ttf"
        );
        embedded_asset!(app, "assets/fonts/Open_Sans/static/OpenSans-Regular.ttf");
        embedded_asset!(app, "assets/fonts/Open_Sans/static/OpenSans-Italic.ttf");

        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-Bold.ttf");
        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-BoldItalic.ttf");
        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-Medium.ttf");
        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-MediumItalic.ttf");
        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-Regular.ttf");
        embedded_asset!(app, "assets/fonts/Fira_Sans/FiraSans-Italic.ttf");

        embedded_asset!(app, "assets/fonts/Inter/static/Inter_18pt-Bold.ttf");
        embedded_asset!(app, "assets/fonts/Inter/static/Inter_18pt-BoldItalic.ttf");
        embedded_asset!(app, "assets/fonts/Inter/static/Inter_18pt-Medium.ttf");
        embedded_asset!(app, "assets/fonts/Inter/static/Inter_18pt-MediumItalic.ttf");

        embedded_asset!(app, "assets/icons/add_box.png");
        embedded_asset!(app, "assets/icons/add.png");
        embedded_asset!(app, "assets/icons/checkmark.png");
        embedded_asset!(app, "assets/icons/chevron_down.png");
        embedded_asset!(app, "assets/icons/chevron_up.png");
        embedded_asset!(app, "assets/icons/chevron_left.png");
        embedded_asset!(app, "assets/icons/chevron_right.png");
        embedded_asset!(app, "assets/icons/close.png");
        embedded_asset!(app, "assets/icons/disc.png");
        embedded_asset!(app, "assets/icons/gradient_thumb.png");
        embedded_asset!(app, "assets/icons/lock.png");
        embedded_asset!(app, "assets/icons/redo.png");
        embedded_asset!(app, "assets/icons/remove.png");
        embedded_asset!(app, "assets/icons/tune.png");
        embedded_asset!(app, "assets/icons/undo.png");
        embedded_asset!(app, "assets/icons/zoom_in.png");
        embedded_asset!(app, "assets/icons/zoom_out.png");
        embedded_asset!(app, "assets/shaders/gradient_rect.wgsl");
        embedded_asset!(app, "assets/shaders/swatch_rect.wgsl");
        embedded_asset!(app, "assets/shaders/slider_rect.wgsl");
        app.add_plugins((
            UiMaterialPlugin::<GradientRectMaterial>::default(),
            UiMaterialPlugin::<SliderRectMaterial>::default(),
            UiMaterialPlugin::<SwatchRectMaterial>::default(),
            animation::BistableTransitionPlugin,
            animation::AnimatedTransitionPlugin,
            controls::ControlEventsPlugin,
            InputDispatchPlugin,
        ))
        // .add_plugins((
        //     EventListenerPlugin::<MenuCloseEvent>::default(),
        // ))
        .add_systems(
            Update,
            (
                scrolling::handle_scroll_events,
                scrolling::update_scroll_positions,
                hover_signal::update_hover_states,
                cursor::update_cursor,
            ),
        );
        // .init_resource::<RecentColors>()
        // .add_systems(PostUpdate, floating::position_floating);
    }
}
