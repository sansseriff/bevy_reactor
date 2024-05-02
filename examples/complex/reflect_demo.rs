use bevy::{
    prelude::*,
    reflect::{ReflectKind, ReflectRef},
};
use bevy_reactor::*;
use obsidian_ui::controls::{
    Checkbox, InspectorFieldLabel, InspectorFieldReadonlyValue, InspectorGroup,
};

#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct {
    selected: bool,
    scale: f32,
    color: Srgba,
    position: Vec3,

    unlit: Option<bool>,
    roughness: Option<f32>,
}

trait Inspectable: Send + Sync {
    fn name(&self, cx: &Cx) -> String;
    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect;
}

struct PropertyInspector {
    // Need a reference to the entity being inspected
    target: Box<dyn Inspectable>,
}

impl PropertyInspector {
    fn create_fields(&self, cx: &mut Cx, target: &dyn Inspectable, _path_prefix: &str) -> ViewRef {
        match target.reflect(cx).reflect_ref() {
            ReflectRef::Struct(str) => {
                let factories = cx.use_resource::<InspectorFactoryRegistry>();
                let num_fields = str.field_len();
                let mut fields: Vec<ViewRef> = Vec::with_capacity(num_fields);
                for findex in 0..num_fields {
                    // let name = str.name_at(findex).unwrap();
                    let field = str.field_at(findex).unwrap();
                    if field.reflect_kind() == ReflectKind::Enum
                        && field
                            .reflect_type_path()
                            .starts_with("core::option::Option")
                    {
                        let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                            panic!("Expected ReflectRef::Enum");
                        };
                        if enum_ref.variant_name() == "None" {
                            // println!("Skipping None for {}", name);
                            continue;
                        }
                    }

                    let name = str.name_at(findex).unwrap();
                    let field = str.field_at(findex).unwrap();
                    for factory in factories.0.iter().rev() {
                        if factory.create_inspector(name, field, &mut fields) {
                            break;
                        }
                    }
                }
                Fragment::from_slice(&fields).into()
            }
            ReflectRef::TupleStruct(_) => todo!(),
            ReflectRef::Tuple(_) => todo!(),
            ReflectRef::List(_) => todo!(),
            ReflectRef::Array(_) => todo!(),
            ReflectRef::Map(_) => todo!(),
            ReflectRef::Enum(_) => todo!(),
            ReflectRef::Value(_) => todo!(),
        }
    }
}

impl ViewTemplate for PropertyInspector {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        InspectorGroup {
            title: self.target.name(cx).into(),
            body: self.create_fields(cx, self.target.as_ref(), ""),
            expanded: Signal::Constant(true),
        }
    }
}

struct InspectableResource<T: Resource + Reflect> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource + Reflect> InspectableResource<T> {
    fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> Inspectable for InspectableResource<T> {
    fn name(&self, cx: &Cx) -> String {
        let res = cx.use_resource::<T>();
        res.reflect_short_type_path().to_string()
    }

    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect {
        cx.use_resource::<T>().as_reflect()
    }
}

pub struct ResourcePropertyInspector<T: Resource> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource> ResourcePropertyInspector<T> {
    pub fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> ViewTemplate for ResourcePropertyInspector<T> {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        PropertyInspector {
            target: Box::new(InspectableResource::<T>::new()),
        }
        .to_ref()
    }
}

/// Trait that defines a factory for creating inspectors. Multiple factories can be registered,
/// and the first one that returns true will be used to create the inspector.
pub trait InspectorFactory: Sync + Send {
    /// Examine the reflect data and decide what kind of widget to create to edit the
    /// data. Can return false if the data is not in a supported format.
    fn create_inspector(&self, name: &str, reflect: &dyn Reflect, views: &mut Vec<ViewRef>)
        -> bool;
}

#[derive(Resource, Default)]
struct InspectorFactoryRegistry(pub Vec<Box<dyn InspectorFactory>>);

#[derive(Default)]
struct DefaultInspectorFactory;

impl InspectorFactory for DefaultInspectorFactory {
    fn create_inspector(
        &self,
        name: &str,
        reflect: &dyn Reflect,
        views: &mut Vec<ViewRef>,
    ) -> bool {
        match reflect.reflect_ref() {
            ReflectRef::Struct(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "Struct:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::TupleStruct(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "TupleStruct:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::Tuple(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "Tuple:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::List(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "List:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::Array(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "Array:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::Map(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "Map:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::Enum(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into(),
                        ..default()
                    }
                    .to_ref(),
                );
                views.push(
                    InspectorFieldReadonlyValue {
                        children: "Enum:TODO".into(),
                        ..default()
                    }
                    .to_ref(),
                );
            }
            ReflectRef::Value(v) => match v.reflect_type_path() {
                "bool" => {
                    views.push(
                        InspectorFieldLabel {
                            children: name.into(),
                            ..default()
                        }
                        .to_ref(),
                    );
                    views.push(
                        Checkbox {
                            checked: Signal::Constant(true),
                            style: StyleHandle::new(|ss: &mut StyleBuilder| {
                                ss.justify_self(JustifySelf::Start);
                            }),
                            ..default()
                        }
                        .to_ref(),
                    );
                }

                _ => {
                    views.push(
                        InspectorFieldLabel {
                            children: name.into(),
                            ..default()
                        }
                        .to_ref(),
                    );
                    views.push(
                        InspectorFieldReadonlyValue {
                            children: format!("Value: {}", v.reflect_type_path()).into(),
                            ..default()
                        }
                        .to_ref(),
                    );
                }
            },
        }
        true
    }
}

#[derive(Default)]
struct BevyTypesInspectorFactory;

impl InspectorFactory for BevyTypesInspectorFactory {
    fn create_inspector(
        &self,
        name: &str,
        reflect: &dyn Reflect,
        views: &mut Vec<ViewRef>,
    ) -> bool {
        match reflect.reflect_ref() {
            ReflectRef::Struct(st) => match st.reflect_type_path() {
                "glam::Vec3" => {
                    views.push(
                        InspectorFieldLabel {
                            children: name.into(),
                            ..default()
                        }
                        .to_ref(),
                    );
                    views.push(
                        InspectorFieldReadonlyValue {
                            children: "Vec3:TODO".into(),
                            ..default()
                        }
                        .to_ref(),
                    );
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}

pub trait RegisterInspectorFactory {
    fn register_inspector<T: InspectorFactory + Default + 'static>(&mut self) -> &mut Self;
}

impl RegisterInspectorFactory for App {
    fn register_inspector<T: InspectorFactory + Default + 'static>(&mut self) -> &mut Self {
        match self
            .world_mut()
            .get_resource_mut::<InspectorFactoryRegistry>()
        {
            Some(mut registry) => {
                registry.0.push(Box::<T>::default());
            }
            None => {
                self.world_mut()
                    .insert_resource(InspectorFactoryRegistry(vec![Box::<T>::default()]));
            }
        }
        self
    }
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspector::<DefaultInspectorFactory>()
            .register_inspector::<BevyTypesInspectorFactory>();
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::*;

    use super::*;

    #[test]
    fn test_reflection() {
        let test_struct: Box<dyn Reflect> = Box::new(TestStruct {
            selected: true,
            scale: 1.0,
            color: Srgba::BLUE,
            position: Vec3::new(0.0, 0.0, 0.0),
            unlit: Some(true),
            roughness: Some(0.5),
        });

        assert!(test_struct.reflect_kind() == ReflectKind::Struct);
        let ReflectRef::Struct(str) = test_struct.reflect_ref() else {
            panic!("Expected ReflectRef::Struct");
        };
        assert_eq!(str.field_len(), 6);
        for i in 0..str.field_len() {
            let name = str.name_at(i).unwrap();
            println!("Field: {:?}", name);
        }
        // let reflected = TestStruct::reflect(&test_struct);

        // assert_eq!(reflected.selected, true);
        // assert_eq!(reflected.scale, 1.0);
        // assert_eq!(reflected.color, Color::rgb(1.0, 0.0, 0.0));
        // assert_eq!(reflected.position, Vec3::new(0.0, 0.0, 0.0));
        // assert_eq!(reflected.unlit, Some(true));
        // assert_eq!(reflected.roughness, Some(0.5));
    }
}
