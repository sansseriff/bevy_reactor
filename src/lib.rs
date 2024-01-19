//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

mod accessor;
mod bundle;
mod cond;
mod cx;
mod element;
mod mutable;
mod node_span;
mod plugin;
mod presenter;
mod reaction;
mod scope;
mod text;
mod view;
mod view_tuple;

pub use cond::cond;
pub use cond::Cond;
pub use cx::Cx;
pub use cx::Rcx;
pub use cx::ReactiveContext;
pub use cx::ReactiveContextMut;
pub use element::Element;
pub use mutable::Mutable;
pub use plugin::ReactorPlugin;
pub use presenter::*;
pub use reaction::*;
pub(crate) use scope::DespawnScopes;
pub(crate) use scope::TrackingScope;
pub use text::*;
pub use view::*;
