//! Example which uses states and a switch view.

use bevy::{color::palettes, prelude::*};
use bevy_mod_stylebuilder::*;
use bevy_reactor_signals::{Cx, Rcx, RunContextRead};
use bevy_reactor_views::{prelude::*, Switch};

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border(3)
        .padding(3);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Play,
    Pause,
    Intro,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_state(GameState::Intro)
        .add_plugins(ReactorViewsPlugin)
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, (close_on_esc, handle_key_input))
        .run();
}

fn setup_view_root(mut commands: Commands) {
    let camera = commands
        .spawn((Camera2dBundle {
            camera: Camera::default(),
            camera_2d: Camera2d {},
            ..default()
        },))
        .id();

    commands.spawn(
        Element::<NodeBundle>::new()
            .style(style_test)
            .insert(TargetCamera(camera))
            .insert(BorderColor(palettes::css::LIME.into()))
            .children((Element::<NodeBundle>::new(), "State: ", NestedView))
            .to_root(),
    );
}

struct NestedView;

impl ViewTemplate for NestedView {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Switch::new(|cx: &Rcx| cx.use_resource::<State<GameState>>().get().clone())
            .case(GameState::Intro, || "Intro")
            .case(GameState::Pause, || "Pause")
            .fallback(|| "Play")
    }
}

fn handle_key_input(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        match state.get() {
            GameState::Intro => next_state.set(GameState::Play),
            GameState::Play => next_state.set(GameState::Pause),
            GameState::Pause => next_state.set(GameState::Play),
        }
    }
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
