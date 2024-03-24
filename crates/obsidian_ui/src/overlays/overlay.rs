use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};
use bevy_color::LinearRgba;
use bevy_reactor::*;

use super::{
    overlay_material::{UnderlayExtension, UnderlayMaterial},
    shape_builder::ShapeBuilder,
};

/// A transluent overlay that can be used to display diagnostic information in the 3d world.
#[derive(Default)]
pub struct Overlay<F: Fn(&Rcx, &mut ShapeBuilder)>
where
    Self: EffectTarget,
    Self: ParentView,
{
    /// Debug name for this element.
    debug_name: String,

    /// The visible entity for this overlay.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<ChildView>,

    /// List of effects to be added to the element.
    effects: Vec<Box<dyn EntityEffect>>,

    /// Material for the overlay.
    material: Handle<StandardMaterial>,

    /// Material for the underlay.
    underlay_material: Handle<UnderlayMaterial>,

    /// Mesh for the overly
    mesh: Handle<Mesh>,

    /// Color of the overlay.
    color: Signal<LinearRgba>,

    /// Transform of the overlay.
    transform: Signal<Transform>,

    /// Occlusion opacity, 0.0 to 1.0. This represents the opacity of the overlay when it is
    /// occluded by other objects.
    underlay: f32,
    // - blend_mode (signal)
    // - sides
    /// Reactive drawing function
    draw: F,
}

impl<F: Fn(&Rcx, &mut ShapeBuilder)> Overlay<F> {
    /// Construct a new `Overlay`.
    pub fn new(draw: F) -> Self {
        Self {
            debug_name: String::new(),
            display: None,
            children: Vec::new(),
            effects: Vec::new(),
            material: Handle::default(),
            underlay_material: Handle::default(),
            mesh: Handle::default(),
            color: Signal::Constant(LinearRgba::default()),
            transform: Signal::Constant(Transform::default()),
            underlay: 0.3,
            draw,
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity, draw: F) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(node),
            children: Vec::new(),
            effects: Vec::new(),
            material: Handle::default(),
            underlay_material: Handle::default(),
            mesh: Handle::default(),
            color: Signal::Constant(LinearRgba::default()),
            transform: Signal::Constant(Transform::default()),
            underlay: 0.3,
            draw,
        }
    }

    /// Set the debug name for this element.
    pub fn named(mut self, name: &str) -> Self {
        self.debug_name = name.to_string();
        self
    }

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&self, world: &mut World) {
        let flat = self.child_nodes();
        world
            .entity_mut(self.display.unwrap())
            .replace_children(&flat);
    }

    /// "Underlay" controls the opacity of the overlay when it is occluded by other objects.
    /// A value of 0 means that occluded portions of the overlay are completely invisible,
    /// while a value of 1 means that the overlay is completely visible even when occluded.
    pub fn with_underlay(mut self, underlay: f32) -> Self {
        self.underlay = underlay;
        self
    }

    /// Set the color for this overlay.
    pub fn with_color(mut self, color: impl Into<LinearRgba>) -> Self {
        self.color = Signal::Constant(color.into());
        self
    }

    /// Set the color for this overlay as a signal.
    pub fn with_color_signal(mut self, color: impl Into<Signal<LinearRgba>>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the transform for this overlay.
    pub fn with_transform(mut self, transform: impl Into<Transform>) -> Self {
        self.transform = Signal::Constant(transform.into());
        self
    }

    /// Set the transform for this overlay as a signal.
    pub fn with_transform_signal(mut self, transform: impl Into<Signal<Transform>>) -> Self {
        self.transform = transform.into();
        self
    }
}

impl<F: Fn(&Rcx, &mut ShapeBuilder)> EffectTarget for Overlay<F> {
    fn add_effect(&mut self, effect: Box<dyn EntityEffect>) {
        self.effects.push(effect);
    }
}

impl<F: Fn(&Rcx, &mut ShapeBuilder)> ParentView for Overlay<F> {
    fn children(&self) -> &Vec<ChildView> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<ChildView> {
        &mut self.children
    }
}

impl<F: Send + Sync + 'static + Fn(&Rcx, &mut ShapeBuilder)> View for Overlay<F> {
    fn nodes(&self) -> NodeSpan {
        match self.display {
            None => NodeSpan::Empty,
            Some(node) => NodeSpan::Node(node),
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        world.entity_mut(view_entity).insert(Name::new("Overlay"));

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh_handle = meshes.add(mesh);
        self.mesh = mesh_handle.clone();

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let material = materials.add(StandardMaterial {
            // base_color: LinearRgba::default().into(),
            unlit: true,
            alpha_mode: bevy::pbr::AlphaMode::Blend,
            ..Default::default()
        });
        self.material = material.clone();

        let mut underlay_materials = world
            .get_resource_mut::<Assets<UnderlayMaterial>>()
            .unwrap();
        let underlay_material = underlay_materials.add(UnderlayMaterial {
            base: StandardMaterial {
                base_color: LinearRgba::default().into(),
                unlit: true,
                alpha_mode: bevy::pbr::AlphaMode::Blend,
                ..Default::default()
            },
            extension: UnderlayExtension {},
        });
        self.underlay_material = underlay_material.clone();

        let bundle = (
            Name::new(self.debug_name.clone()),
            MaterialMeshBundle::<StandardMaterial> {
                material,
                mesh: mesh_handle,
                ..default()
            },
            underlay_material,
            NotShadowCaster,
            NotShadowReceiver,
        );

        // Build display entity if it doesn't already exist.
        let display = match self.display {
            Some(display) => {
                world.entity_mut(display).insert(bundle);
                display
            }
            None => {
                let entity = world.spawn(bundle).id();
                self.display = Some(entity);
                entity
            }
        };

        // TODO: only insert an underlay material if the underlay is between 0 and 1 (exclusive).
        // If it's zero, the underly is invisible.
        // If it's one, then we can just disable the depth test on the primary material.
        // if self.underlay > 0.0 && self.underlay < 1.0 {}

        // Build the overlay mesh the first time.
        let mut tracking = TrackingScope::new(world.read_change_tick());
        self.react(view_entity, world, &mut tracking);

        // Start reactions
        self.start_reaction(
            ChangeColorReaction {
                color: self.color,
                underlay: self.underlay,
                material: self.material.clone(),
                underlay_material: self.underlay_material.clone(),
            },
            display,
            world,
            &mut tracking,
        );
        self.start_reaction(
            ChangeTransformReaction {
                mesh: display,
                transform: self.transform,
            },
            display,
            world,
            &mut tracking,
        );
        for effect in self.effects.iter_mut() {
            effect.start(display, world, &mut tracking);
        }
        world.entity_mut(view_entity).insert(tracking);

        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewHandle::spawn(&child.view, view_entity, world));
        }

        self.attach_children(world);
    }

    fn react(&mut self, _view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        // Rebuild the overlay mesh.
        let re = Rcx::new(world, tracking);
        let mut builder = ShapeBuilder::new();
        (self.draw)(&re, &mut builder);
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.get_mut(self.mesh.clone()).unwrap();
        builder.build(mesh);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.display.is_some());
        self.raze_children(world);

        // Delete the display node.
        world.entity_mut(self.display.unwrap()).remove_parent();
        world.entity_mut(self.display.unwrap()).despawn();
        self.display = None;

        // Delete all reactions.
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
        // info!("children_changed handled");
        self.attach_children(world);
        true
    }
}

impl<F: Send + Sync + 'static + Fn(&Rcx, &mut ShapeBuilder)> From<Overlay<F>> for ViewHandle {
    fn from(value: Overlay<F>) -> Self {
        ViewHandle::new(value)
    }
}

/// Reactive effect which changes the color of the overlay.
pub struct ChangeColorReaction {
    color: Signal<LinearRgba>,
    underlay: f32,
    material: Handle<StandardMaterial>,
    underlay_material: Handle<UnderlayMaterial>,
}

impl Reaction for ChangeColorReaction {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, tracking);
        let mut color = self.color.get(&re);

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let material = materials.get_mut(self.material.id()).unwrap();
        material.base_color = color.into();

        let mut underlay_materials = world
            .get_resource_mut::<Assets<UnderlayMaterial>>()
            .unwrap();
        let underlay_material = underlay_materials
            .get_mut(self.underlay_material.id())
            .unwrap();
        color.alpha *= self.underlay;
        underlay_material.base.base_color = color.into();
    }
}

/// Reactive effect which changes the transform of the overlay.
pub struct ChangeTransformReaction {
    mesh: Entity,
    transform: Signal<Transform>,
}

impl Reaction for ChangeTransformReaction {
    fn react(&mut self, _owner: Entity, world: &mut World, _tracking: &mut TrackingScope) {
        let next_transform = self.transform.get(world);
        let mut entt = world.entity_mut(self.mesh);
        let mut transform = entt.get_mut::<Transform>().unwrap();
        *transform = next_transform;
    }
}
