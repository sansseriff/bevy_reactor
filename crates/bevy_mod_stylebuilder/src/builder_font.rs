#![allow(missing_docs)]

use crate::text_styles::InheritableFontColor;
use crate::{InheritableFont, InheritableFontSize, MaybeHandleOrPath};

use crate::{ColorParam, OptFloatParam, StyleBuilder, StyleCommands};
use bevy::prelude::*;

pub trait StyleBuilderFont {
    fn color(&mut self, color: impl ColorParam) -> &mut Self;
    fn font<'p>(&mut self, path: impl Into<MaybeHandleOrPath<'p, Font>>) -> &mut Self;
    fn font_size(&mut self, val: impl OptFloatParam) -> &mut Self;
}

impl<'a, 'w> StyleBuilderFont for StyleBuilder<'a, 'w> {
    fn color(&mut self, color: impl ColorParam) -> &mut Self {
        match color.to_val() {
            Some(color) => self.target.insert(InheritableFontColor(color)),
            None => self.target.remove::<InheritableFontColor>(),
        };
        self
    }

    fn font<'p>(&mut self, path: impl Into<MaybeHandleOrPath<'p, Font>>) -> &mut Self {
        let font = match path.into() {
            MaybeHandleOrPath::Handle(h) => Some(h),
            MaybeHandleOrPath::Path(p) => Some(self.load_asset::<Font>(p)),
            MaybeHandleOrPath::None => None,
        };
        match font {
            Some(font) => self.target.insert(InheritableFont(font)),
            None => self.target.remove::<InheritableFont>(),
        };
        self
    }

    fn font_size(&mut self, val: impl OptFloatParam) -> &mut Self {
        match val.to_val() {
            Some(size) => self.target.insert(InheritableFontSize(size)),
            None => self.target.remove::<InheritableFontSize>(),
        };
        self
    }
}

impl<'a, 'w> StyleBuilderFont for StyleCommands<'a, 'w> {
    fn color(&mut self, color: impl ColorParam) -> &mut Self {
        match color.to_val() {
            Some(color) => self.target.insert(InheritableFontColor(color)),
            None => self.target.remove::<InheritableFontColor>(),
        };
        self
    }

    fn font<'p>(&mut self, path: impl Into<MaybeHandleOrPath<'p, Font>>) -> &mut Self {
        let font = match path.into() {
            MaybeHandleOrPath::Handle(h) => Some(h),
            MaybeHandleOrPath::Path(p) => Some(self.load_asset::<Font>(p)),
            MaybeHandleOrPath::None => None,
        };
        match font {
            Some(font) => self.target.insert(InheritableFont(font)),
            None => self.target.remove::<InheritableFont>(),
        };
        self
    }

    fn font_size(&mut self, val: impl OptFloatParam) -> &mut Self {
        match val.to_val() {
            Some(size) => self.target.insert(InheritableFontSize(size)),
            None => self.target.remove::<InheritableFontSize>(),
        };
        self
    }
}
