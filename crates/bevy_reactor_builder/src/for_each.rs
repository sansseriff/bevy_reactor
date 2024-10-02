use std::ops::Range;

use bevy::prelude::*;
use bevy::{ecs::world::World, ui::GhostNode};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::lcs::lcs;
use crate::UiBuilder;

pub trait ForEachBuilder {
    /// Construct child elements from an array of items. The callback is called once for each
    /// array element; its argument is the item, which must be equals-comparable, and it builds
    /// children for that item. During rebuild, the list of child views may be re-ordered based
    /// on a comparison of the items from the previous build.
    fn for_each<
        Item: Send + Sync + 'static + Clone + PartialEq,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self;

    /// Construct an keyed for loop for an array of items. This version accepts a custom
    /// comparator function, which can be used if the items don't implement `PartialEq`.
    fn for_each_cmp<
        Item: Send + Sync + 'static + Clone,
        CmpFn: Send + Sync + 'static + Fn(&Item, &Item) -> bool,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        cmp: CmpFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self;
}

impl<'w> ForEachBuilder for UiBuilder<'w> {
    fn for_each<
        Item: Send + Sync + 'static + Clone + PartialEq,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self {
        self.for_each_cmp(items, |a, b| a == b, each, fallback);
        self
    }

    fn for_each_cmp<
        Item: Send + Sync + 'static + Clone,
        CmpFn: Send + Sync + 'static + Fn(&Item, &Item) -> bool,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    >(
        &mut self,
        items: ItemFn,
        cmp: CmpFn,
        each: EachFn,
        fallback: FallbackFn,
    ) -> &mut Self {
        // Create an entity to represent the condition.
        let mut owner = self.spawn(Name::new("Cond"));
        let owner_id = owner.id();

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(owner.world().last_change_tick());
        let mut reaction = ForEachReaction {
            items,
            cmp,
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
struct ForEachReaction<
    Item: Clone,
    CmpFn: Fn(&Item, &Item) -> bool,
    ItemIter: Iterator<Item = Item>,
    ItemFn: Fn(&Rcx) -> ItemIter,
    EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
    FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
> where
    Self: Send + Sync,
{
    items: ItemFn,
    cmp: CmpFn,
    each: EachFn,
    fallback: FallbackFn,
    fallback_ent: Option<Entity>,
    state: Vec<ListItem<Item>>,
}

impl<
        Item: Send + Sync + Clone,
        CmpFn: Send + Sync + Fn(&Item, &Item) -> bool,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Send + Sync + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    > ForEachReaction<Item, CmpFn, ItemIter, ItemFn, EachFn, FallbackFn>
{
    /// Uses the sequence of key values to match the previous array items with the updated
    /// array items. Matching items are patched, other items are inserted or deleted.
    ///
    /// # Arguments
    ///
    /// * `bc` - [`BuildContext`] used to build individual elements.
    /// * `prev_state` - Array of view state elements from previous update.
    /// * `prev_range` - The range of elements we are comparing in `prev_state`.
    /// * `next_state` - Array of view state elements to be built.
    /// * `next_range` - The range of elements we are comparing in `next_state`.
    #[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
    fn build_recursive(
        &self,
        world: &mut World,
        // owner: Entity,
        prev_state: &[ListItem<Item>],
        prev_range: Range<usize>,
        next_items: &[Item],
        next_range: Range<usize>,
        out: &mut Vec<ListItem<Item>>,
    ) {
        // Look for longest common subsequence.
        // prev_start and next_start are *relative to the slice*.
        let (prev_start, next_start, lcs_length) = lcs(
            &prev_state[prev_range.clone()],
            &next_items[next_range.clone()],
            |a, b| (self.cmp)(&a.item, b),
        );

        // If there was nothing in common
        if lcs_length == 0 {
            // Raze old elements
            for i in prev_range {
                let prev = &prev_state[i];
                world.entity_mut(prev.child).despawn_recursive();
            }
            // Build new elements
            for i in next_range {
                let child_id = world.spawn(GhostNode).id();
                (self.each)(&next_items[i], &mut UiBuilder::new(world, child_id));
                out.push(ListItem {
                    child: child_id,
                    item: next_items[i].clone(),
                });
            }
            return;
        }

        // Adjust prev_start and next_start to be relative to the entire state array.
        let prev_start = prev_start + prev_range.start;
        let next_start = next_start + next_range.start;

        // Stuff that precedes the LCS.
        if prev_start > prev_range.start {
            if next_start > next_range.start {
                // Both prev and next have entries before lcs, so recurse
                self.build_recursive(
                    world,
                    // owner,
                    prev_state,
                    prev_range.start..prev_start,
                    next_items,
                    next_range.start..next_start,
                    out,
                )
            } else {
                // Deletions
                for i in prev_range.start..prev_start {
                    let prev = &prev_state[i];
                    world.entity_mut(prev.child).despawn_recursive();
                }
            }
        } else if next_start > next_range.start {
            // Insertions
            for i in next_range.start..next_start {
                let child_id = world.spawn(GhostNode).id();
                (self.each)(&next_items[i], &mut UiBuilder::new(world, child_id));
                out.push(ListItem {
                    child: child_id,
                    item: next_items[i].clone(),
                });
            }
        }

        // For items that match, copy over the view and value.
        for i in 0..lcs_length {
            let prev = &prev_state[prev_start + i];
            out.push(prev.clone());
        }

        // Stuff that follows the LCS.
        let prev_end = prev_start + lcs_length;
        let next_end = next_start + lcs_length;
        if prev_end < prev_range.end {
            if next_end < next_range.end {
                // Both prev and next have entries after lcs, so recurse
                self.build_recursive(
                    world,
                    // owner,
                    prev_state,
                    prev_end..prev_range.end,
                    next_items,
                    next_end..next_range.end,
                    out,
                );
            } else {
                // Deletions
                for i in prev_end..prev_range.end {
                    let prev = &prev_state[i];
                    world.entity_mut(prev.child).despawn_recursive();
                }
            }
        } else if next_end < next_range.end {
            // Insertions
            for i in next_end..next_range.end {
                let child_id = world.spawn(GhostNode).id();
                (self.each)(&next_items[i], &mut UiBuilder::new(world, child_id));
                out.push(ListItem {
                    child: child_id,
                    item: next_items[i].clone(),
                });
            }
        }
    }
}

impl<
        Item: Send + Sync + Clone,
        CmpFn: Send + Sync + Fn(&Item, &Item) -> bool,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Send + Sync + Fn(&Rcx) -> ItemIter,
        EachFn: Send + Sync + 'static + Fn(&Item, &mut UiBuilder),
        FallbackFn: Send + Sync + 'static + Fn(&mut UiBuilder),
    > Reaction for ForEachReaction<Item, CmpFn, ItemIter, ItemFn, EachFn, FallbackFn>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        // Create a reactive context and call the test condition.
        let iter = (self.items)(&Rcx::new(world, owner, tracking));
        let hint = iter.size_hint().0;
        let items: Vec<Item> = iter.collect();
        let mut next_state: Vec<ListItem<Item>> = Vec::with_capacity(hint);
        let next_len = items.len();
        let prev_len = self.state.len();

        self.build_recursive(
            world,
            // owner,
            &self.state,
            0..prev_len,
            &items,
            0..next_len,
            &mut next_state,
        );
        let children: Vec<Entity> = next_state.iter().map(|i| i.child).collect();
        world.entity_mut(owner).replace_children(&children);
        self.state = std::mem::take(&mut next_state);

        // Handle fallback
        match self.fallback_ent {
            // If there are > 0 items, destroy fallback if present.
            Some(fb_ent) if next_len > 0 => {
                world.entity_mut(fb_ent).despawn_recursive();
                self.fallback_ent = None;
            }

            // If there are no items, render fallback unless already rendered.
            None if next_len == 0 => {
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
