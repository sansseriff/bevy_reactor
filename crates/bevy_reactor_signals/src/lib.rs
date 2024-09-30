//! Implementation of the reactive signals pattern for Bevy.
#![warn(missing_docs)]

use bevy::app::{App, Plugin, Update};

mod callback;
mod cx;
mod derived;
mod ecx;
mod mutable;
mod rcx;
mod reaction;
mod signal;
mod tracking_scope;

use callback::cleanup_callbacks;
pub use callback::{Callback, CallbackOwner, RunCallback};
pub use cx::Cx;
pub use cx::RunContextRead;
pub use cx::RunContextSetup;
pub use derived::{create_derived, Derived, ReadDerived};
pub use ecx::Ecx;
pub use mutable::{create_mutable, CreateMutable, Mutable, ReadMutable, WriteMutable};
pub use rcx::Rcx;
pub use reaction::*;
pub use signal::IntoSignal;
pub use signal::Signal;
pub use tracking_scope::TrackingScope;
pub use tracking_scope::TrackingScopeTracing;
use tracking_scope::{cleanup_tracking_scopes, run_reactions};

/// Plugin that adds the reactive UI system to the app.
pub struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        cleanup_tracking_scopes(app.world_mut());
        cleanup_callbacks(app.world_mut());
        app.add_systems(Update, run_reactions);
    }
}
