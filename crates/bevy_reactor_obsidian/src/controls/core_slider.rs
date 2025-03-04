use bevy::prelude::*;

use super::Disabled;

#[derive(Clone, Debug, Component)]
pub struct ValueChange<T>(pub T);

impl<T: Send + Sync + 'static> Event for ValueChange<T> {
    type Traversal = &'static Parent;

    const AUTO_PROPAGATE: bool = true;
}

/// A headless slider widget, which can be used to build custom sliders. This component emits
/// [`ValueChange`] events when the slider value changes. Note that the value in the event is
/// unclamped - the reason is that the receiver may want to quantize or otherwise modify the value
/// before clamping. It is the receiver's responsibility to update the slider's value when
/// the value change event is received.
#[derive(Component)]
#[require(DragState)]
pub struct CoreSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl CoreSlider {
    /// Constructg a new [`CoreSlider`].
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self { value, min, max }
    }

    /// Get the current value of the slider.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set the value of the slider, clamping it to the min and max values.
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
}

/// Component used to manage the state of a slider during dragging.
#[derive(Component, Default)]
pub struct DragState {
    /// Whether the slider is currently being dragged.
    dragging: bool,
    /// The value of the slider when dragging started.
    offset: f32,
}

pub(crate) fn slider_on_drag_start(
    mut trigger: Trigger<Pointer<DragStart>>,
    mut q_state: Query<(&CoreSlider, &mut DragState, Has<Disabled>)>,
) {
    if let Ok((slider, mut drag, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if !disabled {
            drag.dragging = true;
            drag.offset = slider.value;
        }
    }
}

pub(crate) fn slider_on_drag(
    mut trigger: Trigger<Pointer<Drag>>,
    mut q_state: Query<(&ComputedNode, &CoreSlider, &mut DragState)>,
    mut commands: Commands,
) {
    if let Ok((node, slider, drag)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if drag.dragging {
            let distance = trigger.event().distance;
            // Measure node width and slider value.
            let slider_width = node.size().x;
            let range = slider.max - slider.min;
            let new_value = if range > 0. {
                drag.offset + (distance.x * range) / slider_width
            } else {
                slider.min + range * 0.5
            };
            commands.trigger_targets(ValueChange(new_value), trigger.entity());
        }
    }
}

pub(crate) fn slider_on_drag_end(
    mut trigger: Trigger<Pointer<DragEnd>>,
    mut q_state: Query<(&CoreSlider, &mut DragState)>,
) {
    if let Ok((_slider, mut drag)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if drag.dragging {
            drag.dragging = false;
        }
    }
}
