//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

mod callback;
mod compositor;
mod cond;
mod cx;
mod derived;
mod dynamic;
mod dynamic_keyed;
mod effect_target;
mod element;
mod r#for;
mod for_each;
mod for_index;
mod fragment;
mod hover;
mod lcs;
mod mutable;
mod node_span;
mod parent_view;
mod plugin;
mod portal;
mod presenter;
mod reaction;
mod signal;
mod style;
mod switch;
mod text;
mod tracking_scope;
mod view;

pub use callback::CallDeferred;
pub use callback::Callback;
pub use compositor::Compositor;
pub use cond::cond;
pub use cond::Cond;
pub use cx::Cx;
pub use cx::Rcx;
pub use cx::RunContextRead;
pub use cx::RunContextSetup;
pub use cx::RunContextWrite;
pub use derived::Derived;
pub use derived::ReadDerived;
pub use dynamic::Dynamic;
pub use dynamic_keyed::DynamicKeyed;
pub use effect_target::EffectTarget;
pub use effect_target::EntityEffect;
pub use element::Element;
pub use for_each::ForEach;
pub use for_index::ForIndex;
pub use fragment::Fragment;
pub use hover::CreateHoverSignal;
pub use mutable::Mutable;
pub use mutable::ReadMutable;
pub use mutable::WriteMutable;
pub use node_span::NodeSpan;
pub use parent_view::ChildView;
pub use parent_view::ChildViewTuple;
pub use parent_view::ParentView;
pub use plugin::ReactorPlugin;
pub use portal::Portal;
pub use presenter::*;
pub use r#for::For;
pub use reaction::*;
pub use signal::Signal;
pub use style::StyleBuilder;
pub use style::StyleBuilderBackground;
pub use style::StyleBuilderBorderColor;
pub use style::StyleBuilderBorderRadius;
pub use style::StyleBuilderFont;
pub use style::StyleBuilderLayout;
pub use style::StyleBuilderOutline;
pub use style::StyleBuilderPointerEvents;
pub use style::StyleBuilderZIndex;
pub use style::StyleHandle;
pub use style::StyleTuple;
pub use style::WithStyles;
pub use switch::Case;
pub use switch::Switch;
pub use text::*;
pub use tracking_scope::DespawnScopes;
pub use tracking_scope::TrackingScope;
pub use tracking_scope::TrackingScopeTracing;
pub use view::*;
// pub use style::StyleBuilderTextureAtlas;
