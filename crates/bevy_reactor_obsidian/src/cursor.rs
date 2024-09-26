use bevy::{
    picking::{focus::HoverMap, pointer::PointerId},
    prelude::*,
    render::view::cursor::{CursorIcon, CustomCursor},
};
use bevy_mod_stylebuilder::{MaybeHandleOrPath, StyleBuilder};

#[allow(missing_docs)]
pub trait StyleBuilderCursor {
    fn cursor(&mut self, icon: CursorIcon) -> &mut Self;
    fn cursor_image<'p>(
        &mut self,
        path: impl Into<MaybeHandleOrPath<'p, Image>>,
        origin: Vec2,
    ) -> &mut Self;
    fn cursor_hidden(&mut self) -> &mut Self;
}

impl<'a, 'w> StyleBuilderCursor for StyleBuilder<'a, 'w> {
    fn cursor(&mut self, icon: CursorIcon) -> &mut Self {
        match self.target.get_mut::<CursorIcon>() {
            Some(mut cursor) => {
                *cursor = icon;
            }
            None => {
                self.target.insert(icon);
            }
        };
        self
    }

    fn cursor_image<'p>(
        &mut self,
        path: impl Into<MaybeHandleOrPath<'p, Image>>,
        origin: Vec2,
    ) -> &mut Self {
        let image = match path.into() {
            MaybeHandleOrPath::Handle(h) => Some(h),
            MaybeHandleOrPath::Path(p) => Some(self.load_asset::<Image>(p)),
            MaybeHandleOrPath::None => None,
        };
        match (image, self.target.get_mut::<CursorIcon>()) {
            (Some(image), Some(mut cursor)) => {
                *cursor = CursorIcon::Custom(CustomCursor::Image {
                    handle: image,
                    hotspot: (origin.x as u16, origin.y as u16),
                });
            }
            (Some(image), None) => {
                self.target.insert(CursorIcon::Custom(CustomCursor::Image {
                    handle: image,
                    hotspot: (origin.x as u16, origin.y as u16),
                }));
            }
            (None, Some(_)) => {
                self.target.remove::<CursorIcon>();
            }
            _ => (),
        };
        self
    }

    fn cursor_hidden(&mut self) -> &mut Self {
        //     match self.target.get_mut::<CursorIcon>() {
        //         Some(mut cursor) => {
        //             *cursor = CursorIcon::System(SystemCursorIcon::);
        //         }
        //         None => {
        //             self.target.insert(Cursor::Hidden);
        //         }
        //     };
        //     self
        todo!();
    }
}

pub(crate) fn update_cursor(
    mut commands: Commands,
    hover_map: Option<Res<HoverMap>>,
    parent_query: Query<&Parent>,
    cursor_query: Query<&CursorIcon>,
    mut q_windows: Query<(Entity, &mut Window, Option<&CursorIcon>)>,
) {
    let cursor = hover_map.and_then(|hover_map| match hover_map.get(&PointerId::Mouse) {
        Some(hover_set) => hover_set.keys().find_map(|entity| {
            cursor_query.get(*entity).ok().or_else(|| {
                parent_query
                    .iter_ancestors(*entity)
                    .find_map(|e| cursor_query.get(e).ok())
            })
        }),
        None => None,
    });

    let mut windows_to_change: Vec<Entity> = Vec::new();
    for (entity, _window, prev_cursor) in q_windows.iter_mut() {
        match (cursor, prev_cursor) {
            (Some(cursor), Some(prev_cursor)) if cursor == prev_cursor => continue,
            (None, None) => continue,
            _ => {
                windows_to_change.push(entity);
            }
        }
    }
    windows_to_change.iter().for_each(|entity| {
        if let Some(cursor) = cursor {
            commands.entity(*entity).insert(cursor.clone());
        } else {
            commands.entity(*entity).insert(CursorIcon::default());
        }
    });
}
