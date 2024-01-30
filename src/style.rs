#![allow(missing_docs)]
//! Defines fluent builder for styles.

use std::sync::Arc;

use bevy::{
    asset::AssetPath,
    prelude::*,
    ui::{self, ZIndex},
};
use impl_trait_for_tuples::*;

use crate::element_effect::ElementEffect;
use crate::{Element, TrackingScope};

pub type StyleBuilderFn = dyn Fn(&mut StyleBuilder) + Send + Sync + 'static;

pub struct StyleBuilder<'a, 'w> {
    target: &'a mut EntityWorldMut<'w>,
    style: ui::Style,
    style_changed: bool,
}

/// Trait that represents a CSS color
pub trait ColorParam {
    fn to_val(self) -> Option<Color>;
}

impl ColorParam for Option<Color> {
    fn to_val(self) -> Option<Color> {
        self
    }
}

impl ColorParam for Color {
    fn to_val(self) -> Option<Color> {
        Some(self)
    }
}

impl ColorParam for &str {
    fn to_val(self) -> Option<Color> {
        Some(Color::hex(self).unwrap())
    }
}

/// Trait that represents a CSS "length"
pub trait LengthParam {
    fn to_val(self) -> ui::Val;
}

impl LengthParam for ui::Val {
    fn to_val(self) -> ui::Val {
        self
    }
}

impl LengthParam for f32 {
    fn to_val(self) -> ui::Val {
        ui::Val::Px(self)
    }
}

impl LengthParam for i32 {
    fn to_val(self) -> ui::Val {
        ui::Val::Px(self as f32)
    }
}

/// Trait that represents a CSS Z-index
pub trait ZIndexParam {
    fn to_val(self) -> ZIndex;
}

impl ZIndexParam for ZIndex {
    fn to_val(self) -> ZIndex {
        self
    }
}

impl ZIndexParam for i32 {
    fn to_val(self) -> ZIndex {
        ZIndex::Local(self)
    }
}

/// Trait that represents CSS edge widths (margin, padding, etc.)
pub trait UiRectParam {
    fn to_uirect(self) -> ui::UiRect;
}

impl UiRectParam for ui::UiRect {
    fn to_uirect(self) -> ui::UiRect {
        self
    }
}

impl UiRectParam for ui::Val {
    fn to_uirect(self) -> ui::UiRect {
        ui::UiRect::all(self)
    }
}

impl UiRectParam for f32 {
    fn to_uirect(self) -> ui::UiRect {
        ui::UiRect::all(ui::Val::Px(self))
    }
}

impl UiRectParam for i32 {
    fn to_uirect(self) -> ui::UiRect {
        ui::UiRect::all(ui::Val::Px(self as f32))
    }
}

impl<H: LengthParam, V: LengthParam> UiRectParam for (H, V) {
    fn to_uirect(self) -> ui::UiRect {
        ui::UiRect::axes(self.0.to_val(), self.1.to_val())
    }
}

/// Trait that represents an optional float
pub trait OptFloatParam {
    fn to_val(self) -> Option<f32>;
}

impl OptFloatParam for Option<f32> {
    fn to_val(self) -> Option<f32> {
        self
    }
}

impl OptFloatParam for f32 {
    fn to_val(self) -> Option<f32> {
        Some(self)
    }
}

impl OptFloatParam for i32 {
    fn to_val(self) -> Option<f32> {
        Some(self as f32)
    }
}

impl<'a, 'w> StyleBuilder<'a, 'w> {
    pub fn background_image(&mut self, _img: Option<AssetPath<'static>>) -> &mut Self {
        // TODO: Need to load the asset path from the asset server.
        // Will need a helper component for this.
        // self.props.push(StyleProp::BackgroundImage(img));
        self
    }

    pub fn background_color(&mut self, color: impl ColorParam) -> &mut Self {
        if let Some(color) = color.to_val() {
            self.target.insert(ui::BackgroundColor(color));
        } else {
            self.target.remove::<ui::BackgroundColor>();
        }
        self
    }

    pub fn border_color(&mut self, color: impl ColorParam) -> &mut Self {
        if let Some(color) = color.to_val() {
            self.target.insert(ui::BorderColor(color));
        } else {
            self.target.remove::<ui::BorderColor>();
        }
        self
    }

    // pub fn color(&mut self, color: impl ColorParam) -> &mut Self {
    // Will need a helper component for this.
    //     self.props.push(StyleProp::Color(color.to_val()));
    //     self
    // }

    pub fn z_index(&mut self, index: impl ZIndexParam) -> &mut Self {
        match index.to_val() {
            ZIndex::Local(0) => self.target.remove::<ZIndex>(),
            val => self.target.insert(val),
        };
        self
    }

    pub fn display(&mut self, disp: ui::Display) -> &mut Self {
        self.style.display = disp;
        self.style_changed = true;
        self
    }

    pub fn position(&mut self, pos: ui::PositionType) -> &mut Self {
        self.style.position_type = pos;
        self.style_changed = true;
        self
    }

    pub fn overflow(&mut self, ov: ui::OverflowAxis) -> &mut Self {
        self.style.overflow.x = ov;
        self.style.overflow.y = ov;
        self.style_changed = true;
        self
    }

    pub fn overflow_x(&mut self, ov: ui::OverflowAxis) -> &mut Self {
        self.style.overflow.x = ov;
        self.style_changed = true;
        self
    }

    pub fn overflow_y(&mut self, ov: ui::OverflowAxis) -> &mut Self {
        self.style.overflow.y = ov;
        self.style_changed = true;
        self
    }

    pub fn direction(&mut self, dir: ui::Direction) -> &mut Self {
        self.style.direction = dir;
        self.style_changed = true;
        self
    }

    pub fn left(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.left = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn right(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.right = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn top(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.top = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn bottom(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.bottom = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn width(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.width = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn height(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.height = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn min_width(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.min_width = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn min_height(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.min_height = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn max_width(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.max_width = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn max_height(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.max_height = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn aspect_ratio(&mut self, length: impl OptFloatParam) -> &mut Self {
        self.style.aspect_ratio = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn margin(&mut self, rect: impl UiRectParam) -> &mut Self {
        self.style.margin = rect.to_uirect();
        self.style_changed = true;
        self
    }

    pub fn margin_left(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.margin.left = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn margin_right(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.margin.right = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn margin_top(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.margin.top = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn margin_bottom(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.margin.bottom = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn padding(&mut self, rect: impl UiRectParam) -> &mut Self {
        self.style.padding = rect.to_uirect();
        self.style_changed = true;
        self
    }

    pub fn padding_left(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.padding.left = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn padding_right(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.padding.right = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn padding_top(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.padding.top = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn padding_bottom(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.padding.bottom = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn border(&mut self, rect: impl UiRectParam) -> &mut Self {
        self.style.border = rect.to_uirect();
        self.style_changed = true;
        self
    }

    pub fn border_left(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.border.left = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn border_right(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.border.right = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn border_top(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.border.top = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn border_bottom(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.border.bottom = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn flex_direction(&mut self, dir: ui::FlexDirection) -> &mut Self {
        self.style.flex_direction = dir;
        self.style_changed = true;
        self
    }

    pub fn flex_wrap(&mut self, w: ui::FlexWrap) -> &mut Self {
        self.style.flex_wrap = w;
        self.style_changed = true;
        self
    }

    pub fn flex(&mut self, grow: f32, shrink: f32, basis: impl LengthParam) -> &mut Self {
        self.style.flex_grow = grow;
        self.style.flex_shrink = shrink;
        self.style.flex_basis = basis.to_val();
        self.style_changed = true;
        self
    }

    pub fn flex_grow(&mut self, n: f32) -> &mut Self {
        self.style.flex_grow = n;
        self.style_changed = true;
        self
    }

    pub fn flex_shrink(&mut self, n: f32) -> &mut Self {
        self.style.flex_shrink = n;
        self.style_changed = true;
        self
    }

    pub fn flex_basis(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.flex_basis = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn row_gap(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.row_gap = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn column_gap(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.column_gap = length.to_val();
        self.style_changed = true;
        self
    }

    pub fn gap(&mut self, length: impl LengthParam) -> &mut Self {
        self.style.row_gap = length.to_val();
        self.style.column_gap = self.style.row_gap;
        self.style_changed = true;
        self
    }

    pub fn align_items(&mut self, align: ui::AlignItems) -> &mut Self {
        self.style.align_items = align;
        self.style_changed = true;
        self
    }

    pub fn align_self(&mut self, align: ui::AlignSelf) -> &mut Self {
        self.style.align_self = align;
        self.style_changed = true;
        self
    }

    pub fn align_content(&mut self, align: ui::AlignContent) -> &mut Self {
        self.style.align_content = align;
        self.style_changed = true;
        self
    }

    pub fn justify_items(&mut self, justify: ui::JustifyItems) -> &mut Self {
        self.style.justify_items = justify;
        self.style_changed = true;
        self
    }

    pub fn justify_self(&mut self, justify: ui::JustifySelf) -> &mut Self {
        self.style.justify_self = justify;
        self.style_changed = true;
        self
    }

    pub fn justify_content(&mut self, justify: ui::JustifyContent) -> &mut Self {
        self.style.justify_content = justify;
        self.style_changed = true;
        self
    }

    pub fn grid_auto_flow(&mut self, flow: ui::GridAutoFlow) -> &mut Self {
        self.style.grid_auto_flow = flow;
        self.style_changed = true;
        self
    }

    pub fn grid_template_rows(&mut self, rows: Vec<ui::RepeatedGridTrack>) -> &mut Self {
        self.style.grid_template_rows = rows;
        self.style_changed = true;
        self
    }

    pub fn grid_template_columns(&mut self, columns: Vec<ui::RepeatedGridTrack>) -> &mut Self {
        self.style.grid_template_columns = columns;
        self.style_changed = true;
        self
    }

    pub fn grid_auto_rows(&mut self, rows: Vec<ui::GridTrack>) -> &mut Self {
        self.style.grid_auto_rows = rows;
        self.style_changed = true;
        self
    }

    pub fn grid_auto_columns(&mut self, columns: Vec<ui::GridTrack>) -> &mut Self {
        self.style.grid_auto_columns = columns;
        self.style_changed = true;
        self
    }

    pub fn grid_row(&mut self, val: ui::GridPlacement) -> &mut Self {
        self.style.grid_row = val;
        self.style_changed = true;
        self
    }

    pub fn grid_row_start(&mut self, val: i16) -> &mut Self {
        self.style.grid_row.set_start(val);
        self.style_changed = true;
        self
    }

    pub fn grid_row_span(&mut self, val: u16) -> &mut Self {
        self.style.grid_row.set_span(val);
        self.style_changed = true;
        self
    }

    pub fn grid_row_end(&mut self, val: i16) -> &mut Self {
        self.style.grid_row.set_end(val);
        self.style_changed = true;
        self
    }

    pub fn grid_column(&mut self, val: ui::GridPlacement) -> &mut Self {
        self.style.grid_column = val;
        self.style_changed = true;
        self
    }

    pub fn grid_column_start(&mut self, val: i16) -> &mut Self {
        self.style.grid_column.set_start(val);
        self.style_changed = true;
        self
    }

    pub fn grid_column_span(&mut self, val: u16) -> &mut Self {
        self.style.grid_column.set_span(val);
        self.style_changed = true;
        self
    }

    pub fn grid_column_end(&mut self, val: i16) -> &mut Self {
        self.style.grid_column.set_end(val);
        self.style_changed = true;
        self
    }

    // LineBreak(BreakLineOn),

    pub fn outline_color(&mut self, color: impl ColorParam) -> &mut Self {
        match (color.to_val(), self.target.get_mut::<ui::Outline>()) {
            (Some(color), Some(mut outline)) => {
                outline.color = color;
            }
            (None, Some(_)) => {
                self.target.remove::<ui::Outline>();
            }
            (Some(color), None) => {
                self.target.insert(ui::Outline {
                    color,
                    ..Default::default()
                });
            }
            (None, None) => (),
        };
        self
    }

    pub fn outline_width(&mut self, length: impl LengthParam) -> &mut Self {
        match self.target.get_mut::<ui::Outline>() {
            Some(mut outline) => {
                outline.width = length.to_val();
            }
            None => {
                self.target.insert(ui::Outline {
                    width: length.to_val(),
                    ..Default::default()
                });
            }
        }
        self
    }

    pub fn outline_offset(&mut self, length: impl LengthParam) -> &mut Self {
        match self.target.get_mut::<ui::Outline>() {
            Some(mut outline) => {
                outline.offset = length.to_val();
            }
            None => {
                self.target.insert(ui::Outline {
                    offset: length.to_val(),
                    ..Default::default()
                });
            }
        }
        self
    }

    // pub fn pointer_events(&mut self, pe: PointerEvents) -> &mut Self {
    //     self.props.push(StyleProp::PointerEvents(pe));
    //     self
    // }

    // pub fn font(&mut self, path: Option<AssetPath<'static>>) -> &mut Self {
    //     self.props.push(StyleProp::Font(path));
    //     self
    // }

    // pub fn font_size(&mut self, val: f32) -> &mut Self {
    //     self.props.push(StyleProp::FontSize(val));
    //     self
    // }

    // pub fn scale_x(&mut self, scale: f32) -> &mut Self {
    //     self.props.push(StyleProp::ScaleX(scale));
    //     self
    // }

    // pub fn scale_y(&mut self, scale: f32) -> &mut Self {
    //     self.props.push(StyleProp::ScaleY(scale));
    //     self
    // }

    // pub fn scale(&mut self, scale: f32) -> &mut Self {
    //     self.props.push(StyleProp::Scale(scale));
    //     self
    // }

    // pub fn rotation(&mut self, rot: f32) -> &mut Self {
    //     self.props.push(StyleProp::Rotation(rot));
    //     self
    // }

    // pub fn translation(&mut self, trans: Vec3) -> &mut Self {
    //     self.props.push(StyleProp::Translation(trans));
    //     self
    // }

    // pub fn transition(&mut self, transition: &[Transition]) -> &mut Self {
    //     self.props
    //         .push(StyleProp::Transition(Vec::from(transition)));
    //     self
    // }
}

/// `StyleTuple` - a variable-length tuple of [`StyleHandle`]s.
pub trait StyleTuple: Sync + Send {
    fn apply(&self, ctx: &mut StyleBuilder);
}

/// Empty tuple.
impl StyleTuple for () {
    fn apply(&self, _ctx: &mut StyleBuilder) {}
}

impl<F: Fn(&mut StyleBuilder) + Send + Sync + 'static> StyleTuple for F {
    fn apply(&self, ctx: &mut StyleBuilder) {
        (self)(ctx);
    }
}

impl StyleTuple for StyleHandle {
    fn apply(&self, ctx: &mut StyleBuilder) {
        if let Some(s) = self.style.as_ref() {
            s.apply(ctx);
        }
    }
}

#[impl_for_tuples(1, 16)]
impl StyleTuple for Tuple {
    for_tuples!( where #( Tuple: StyleTuple )* );

    fn apply(&self, ctx: &mut StyleBuilder) {
        for_tuples!( #( self.Tuple.apply(ctx); )* );
    }
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct ApplyStylesEffect<S: StyleTuple> {
    pub(crate) styles: S,
}

impl<S: StyleTuple> ElementEffect for ApplyStylesEffect<S> {
    // For a style builder, run the builder over the target entity.
    fn start(&mut self, _tracking: &mut TrackingScope, target: Entity, world: &mut World) {
        let mut target = world.entity_mut(target);
        let mut style = ui::Style::default();
        if let Some(s) = target.get::<ui::Style>() {
            style.clone_from(s);
        }
        let mut ctx = StyleBuilder {
            target: &mut target,
            style,
            style_changed: false,
        };
        self.styles.apply(&mut ctx);
        if ctx.style_changed {
            ctx.target.insert(ctx.style);
        }
    }
}

pub trait WithStyles {
    /// Apply a set of style builders to a target.
    fn with_styles<S: StyleTuple + 'static>(self, styles: S) -> Self;
}

impl<B: Bundle + Default> WithStyles for Element<B> {
    fn with_styles<S: StyleTuple + 'static>(mut self, styles: S) -> Self {
        self.add_effect(Box::new(ApplyStylesEffect { styles }));
        self
    }
}

#[derive(Default, Clone)]
pub struct StyleHandle {
    pub style: Option<Arc<dyn StyleTuple>>,
}

impl StyleHandle {
    pub fn new<S: StyleTuple + 'static>(style: S) -> Self {
        Self {
            style: Some(Arc::new(style)),
        }
    }

    pub fn none() -> Self {
        Self { style: None }
    }
}
