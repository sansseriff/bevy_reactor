use bevy::prelude::*;
use bevy::{ecs::world::World, ui::GhostNode};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::UiBuilder;

pub trait ForIndexBuilder {
    fn for_index<
        Item: Send + Sync + 'static + Clone + PartialEq,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, usize, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self;
}

impl<'w> ForIndexBuilder for UiBuilder<'w> {
    /// Construct child elements from an array of items. Unlike `for_each`, this doesn't attempt
    /// to re-order the list if items change. Instead, it is strictly based on the array index:
    /// if an item at index n changes, then the children created by n will be rebuilt. The index
    /// is also passed to the builder function.
    fn for_index<
        Item: Send + Sync + 'static + Clone + PartialEq,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, usize, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self {
        // Create an entity to represent the condition.
        let mut owner = self.spawn(Name::new("Cond"));
        let owner_id = owner.id();

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(owner.world().last_change_tick());
        let mut reaction = ForIndexReaction {
            items,
            each,
            fallback,
            fallback_ent: None,
            state: Vec::new(),
        };

        // Safety: this should be safe because we don't use owner any more after this
        // point.
        let world = unsafe { owner.world_mut() };
        // Trigger the initial reaction.
        reaction.react(owner_id, world, &mut tracking);
        world
            .entity_mut(owner_id)
            .insert((GhostNode, tracking, ReactionCell::new(reaction)));
        self
    }
}

#[derive(Clone)]
struct ListItem<Item: Clone> {
    child: Entity,
    item: Item,
}

/// A reaction that handles the conditional rendering logic.
struct ForIndexReaction<
    Item: Clone + PartialEq,
    ItemIter: Iterator<Item = Item>,
    ItemFn: Fn(&Rcx) -> ItemIter,
    EachFn: Send + Sync + 'static + Fn(&Item, usize, &mut UiBuilder),
    FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
> where
    Self: Send + Sync,
{
    items: ItemFn,
    each: EachFn,
    fallback: FallbackFn,
    fallback_ent: Option<Entity>,
    state: Vec<ListItem<Item>>,
}

impl<
        Item: Send + Sync + Clone + PartialEq,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Send + Sync + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, usize, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    > Reaction for ForIndexReaction<Item, ItemIter, ItemFn, EachFn, FallbackFn>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        // Create a reactive context and call the test condition.
        let rcx = Rcx::new(world, owner, tracking);
        let iter = (self.items)(&rcx);
        let hint = iter.size_hint().0;
        let mut prev_len = self.state.len();
        if hint > prev_len {
            self.state.reserve(hint - prev_len);
        }

        let mut index = 0usize;
        for item in iter {
            if index < prev_len {
                // Overwrite existing items.
                let entry = &mut self.state[index];
                if item != entry.item {
                    world.entity_mut(entry.child).despawn_descendants();
                    (self.each)(&item, index, &mut UiBuilder::new(world, entry.child));
                    entry.item = item.clone();
                }
            } else {
                // Append new items.
                let child_id = world.spawn(GhostNode).id();
                world.entity_mut(owner).add_child(child_id);
                (self.each)(&item, index, &mut UiBuilder::new(world, child_id));
                self.state.push(ListItem {
                    child: child_id,
                    item: item.clone(),
                });
            }
            index += 1;
        }

        // Raze surplus items.

        while index < prev_len {
            prev_len -= 1;
            let entry = &mut self.state[prev_len];
            world.entity_mut(entry.child).remove_parent();
            world.entity_mut(entry.child).despawn_recursive();
            self.state.pop();
        }

        // Handle fallback
        let item_count = self.state.len();
        match self.fallback_ent {
            // If there are > 0 items, destroy fallback if present.
            Some(fb_ent) if item_count > 0 => {
                world.entity_mut(fb_ent).despawn_recursive();
                self.fallback_ent = None;
            }

            // If there are no items, render fallback unless already rendered.
            None if item_count == 0 => {
                let mut fallback_ent = world.spawn(GhostNode);
                fallback_ent.set_parent(owner);
                let fallback_id = fallback_ent.id();
                let mut builder = UiBuilder::new(world, fallback_id);
                (self.fallback)(&mut builder);
                self.fallback_ent = Some(fallback_id);
            }

            // Otherwise, no change.
            _ => {}
        }
    }
}
