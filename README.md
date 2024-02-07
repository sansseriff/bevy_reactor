# Overview

**bevy_reactor** is a framework for fine-grained reactivity in Bevy. It implements reactive
concepts such as signals built on Bevy primitives such as entities and components.

## Features

* Reactive data sources: `Mutable`, `Derived` and `Signal`.
* Tracked ownership.
* Copyable callback handles.
* Create entities that respond to reactive data sources such as mutable variables, Bevy resources
  and components.
* Simplified styling system.

## Examples

The most comprehensive example is named `complex`:

```sh
cargo run --example complex
```

## Getting Started

To use this library, you'll need to install the `ReactorPlugin` plugin. You'll also need
to create a view hierarchy and store it in a `ViewRoot` component.

In addition, if you plan on using this for UI, you'll want to install the
[bevy_mod_picking](https://github.com/aevyrie/bevy_mod_picking)
plugins: `(CorePlugin, InputPlugin, InteractionPlugin, BevyUiBackend)`.

# Usage

## Fine-Grained Reactivity

`bevy_reactor` implements "fine-grained" reactivity, which means that reactions can update
individual components or attributes, not just whole entities. In this respect, it is more like
[Leptos](https://www.leptos.dev/) or [Solid.js](https://www.solidjs.com/). There is no diffing or
VDOM as in [React.js](https://react.dev/), because the ability to do fine-grained updates makes a
VDOM unnecessary.

In coarse-grained frameworks like React or Dioxus, the "component" functions are re-run each
time the display graph is recomputed. In fine-grained frameworks, the component functions are
only executed once, but individual micro-closures defined within that function are run many times.

## Reactive Contexts

Within the reactive framework, there are different kinds of execution contexts. Some contexts
are "reactive" meaning that they automatically re-run when needed. Other kinds of contexts
can be categorized as "handlers" (that is, respond to events or callbacks), or "setup"
contexts: functions that create the various reactive elements but are not themselves reactive.

A "run context parameter" is a parameter which is passed to the functions running in these execution
contexts. There are different types of run context parameter, depending on the type of execution
context.

Here's an example of a context parameter, which is traditionally named `cx`:

```rust
element.insert_computed(|cx| {
    let counter = cx.use_resource::<Counter>();
    BackgroundColor(if counter.count & 1 == 0 {
        Color::DARK_GRAY
    } else {
        Color::MAROON
    })
})
```

The `Cx` parameter type is the most general and powerful. It's actually an amalgam of several traits,
which includes:

* `ReadMutable` - methods for reading to instances of `Mutable`.
* `WriteMutable` - methods for writing to instances of `Derived`.
* `ReadDerived` - methods for reading from instances of `Derived`.
* `RunContextWrite` - a trait that defines methods running callbacks and doing other actions
  which may mutate the world but which don't cause structural changes.
* `RunContextSetup` - a trait that defines methods for *creating* new mutables, callbacks,
  memos and effects.

The `Cx` type also has a `props` attribute which allows properties to be passed to the function
running in the execution context. Not all context param types have this.

Different types of execution contexts will have different types of context parameters. For example,
a memo callback, which computes a value reactively, will have an `Rcx` context paramter that allows
reading of reactive signals but does not allow writing or creating.

In addition, the Bevy `World` implements these same traits, but non-reactively. This means you
can access mutables and callbacks even if you are not in a reactive context. Writing to a mutable
this way will still trigger reactions, but reading a mutable won't create a tracking dependency.

## Mutables and Signals

The simplest type of reactive data source is a `Mutable`, which is simply a variable. You can
create a mutable via `create_mutable()`:

```rust
let pressed = cx.create_mutable::<bool>(false);
```
The return result is a `Mutable<T>`, which is a handle used to access the variable. Internally,
the mutable is simply a Bevy `Entity`, but with some extra type information. Because it's an entity,
it can be freely passed around, copied, captured in closures, and so on.

Creating a mutable this way causes the entity to be added to the "owned" list for the current
tracking scope. This means that when that scope is destroyed - when the object that owns this
execution context is despawned - all of the mutables, memos, effects and other reactive elements
will also be despawned.

Accessing the data in a mutable can be done in one of several ways:

* Getting the data via `mutable.get(context)`;
* Setting the data via `mutable.set(context, value)`;
* Getting a reference to the data via `mutable.as_ref(context)`;
* Transforming the data via `mutable.map(context, mapper_fn)`;
* Accessing the data via a signal: `mutable.signal()`;

The reason we need to pass in a context object (which can be `Cx`, `Rcx` or `World`) is because
we the actual data is stored in Bevy's ECS and we need a way to retrieve it. `Mutable<T>` is just
a handle, it doesn't contain the data itself - but it does contain a type parameter which remembers
what kind of data is being stored.

All of the functions which read the mutable value (`.get()`, `.as_ref()`, `.map()`) also create
a dependency entry in the current tracking scope, which will cause the computation to be re-run
when the value of the mutable changes.

The call `mutable.signal()` returns a reactive signal object. What makes this this different from
the `get()` method is that the receiver is type-erased, in other words, you can pass around a
`Signal` object without revealing the fact that the signal came from a `Mutable`. This is handy
because there are other kinds of reactive objects (like memos and derivations) which can also
produce signals. The function that reads the signal can work regardless of where the signal
came from.

Signals have an API which is similar to mutables: `.get(context)`, `.map(context, mapper)` and so
on.

Mutables are transactional: when you write to a mutable, the change does not take effect until
the next frame. There's an ECS system that "commits" the pending changes.

The `.get()`, `.set()` and `.signal()` methods given above assume that the data in the mutable
implements `Copy`. There is also a `.get_clone()` method, which works with data types that
implement `Clone`.

## Derived Signals

A derived signal is a signal resulting from a computation that depends on other signals.

```rust
/// A signal derived from a resource.
let panel_width = cx
    .create_derived(|cx| {
        let res = cx.use_resource::<PanelWidth>();
        res.0
    });
```

The `.create_derived()` method returns a `Signal<T>`. Reading a derived signal
adds all of the derived's dependencies to the current tracking scope, so if any of those
dependencies change, the caller of the derived will be re-run.

Derived signals are not memoized, however, for that we need to use `Memo` (still to be implemented).

## Tracking Scopes and Reactions

This section talks about some internal aspects of the framework which are not visible to the
outside, but which are important to understand.

A `TrackingScope` is a data structure which keeps track of all the reactive dependencies
accessed within a run context. This includes mutables, resources, components, and anything
else. It uses Bevy's change detection to determine whether a dependency has changed.

Tracking scopes are implemented as ECS components. They are often paired with "reactions",
which is another type of component that contains an action function and a cleanup function.
The action function is run whenever the tracking scope indicates that one or more dependencies
have changed. When this happens, the tracking scope is first cleared; the reaction is expected
to re-subscribe to any dependencies that are needed.

The cleanup function is called when the scope is despawned.

Tracking scopes, reactions and mutables form the basis of more advanced reactive constructs
like memos and derivations.

Tracking scopes are also "owners" meaning they have a list of entities that should be despawned
along with the tracking scope.

## Callbacks

A `Callback` is a handle to a closure. Like mutables, the closure function is stored in an ECS
component. Also like mutables, the handle can be passed around freely, and is destroyed when
the current tracking scope is despawned.

To create a callback, call `.create_callback()`:

```rust
let button_clicked = cx.create_callback(|cx| {
    println!("Button was clicked");
});
```
You can also call `.create_callback_mut()` which creates a mutable (`FnMut`) callback:

```rust
let mut click_count = 0;
let button_clicked = cx.create_callback_mut(move |cx| {
    click_count += 1;
    println!("Button clicked: {} times", click_count);
});
```

To call the callback, you need to call `.run_callback()`:

```rust
cx.run_callback(button_click, ());
// Or
world.run_callback(button_click, ());
```

## Views and Elements

`View` is a trait that describes an object that generates an entity tree. A view is kind of
like a template: it's an entity, but it's not the entity that is actually rendered, rather
it's a factory which both creates and updates the "display graph" - what would correspond to the
"DOM" in a browser.

The most common type of `View` is called `Element`. Elements are very general: they can create
any kind of entity, although they are most often used to create UI nodes:

```rust
Element::<NodeBundle>::new()
    .children((
        Element::<NodeBundle>::new(),
        Element::<NodeBundle>::new(),
    ))
```
Another kind of view is `Text`, which creates a text entity. Note that `bevy_reactor` currently
only supports creating text nodes with a single string. (The reason for this because Bevy's
implementation of how text works may change.)

All views, including elements, have a lifecycle:

* When the view is first constructed, it does nothing until the `.build()` method is called. Before
  that happens, you can call various methods to customize the view, such as adding children
  and effects.
* The framework calls the `.build()` method, which actually creates the display entities,
  attaching children, and starting any effects.
* Each view has a `TrackingScope`, and is similar to a `Reaction`. When the view's dependencies
  change, the `.react()` function of the view is called.
* When the view is ready to be despawned, the `.raze()` method is called to ensure that all
  resources are despanwed as well, including the display graph.

## Element Children

The `Element` object has a method `.children()` which accepts either a single child, or
a variable-length tuple of children. Any object that implements the `IntoView` trait can
be passed as a child view, so for example text strings implement `IntoView` and automatically
generate a text node.

```rust
Element::<NodeBundle>::new()
    .children((
        Element::<NodeBundle>::new(),
        text("Count: "),
        text_computed(|cx| {
            let counter = cx.use_resource::<Counter>();
            format!("{}", counter.count)
        }),
        ", ",
        nested_presenter.bind(()),
        ": ",
    )),
```

## Element Effects

Because views don't immediately create entities, but only do so during the build phase, any
customizations to the display graph have to be deferred until after the display entity exists.
To this end, the `Element` has a list of `effects` which are applied to the display entity
after creation.

Effects are very general, and can do things like insert children, add components, modify styles,
and so on. Effects can also spawn reactions, which are persistent tasks that modify the entity
in response to reactive signals and other dependencies.

The most general effect is created by the `.create_effect()` method, which takes the display
entity and lets you make any changes you want. However, there are more ergonomic helper methods
such as `.insert_computed()` and `.styled()` which remove some of the boilerplate.

## Conditional Rendering

Conditional rendering is accomplished using the `Cond` struct:

```rust
element.children(
    Cond::new(
        |cx| {
            let counter = cx.use_resource::<Counter>();
            counter.count & 1 == 0
        },
        || "[Even]",
        || "[Odd]",
    ))
```
The first argument is a `test` expression, and is a reactive function which returns a boolean.
The other two arguments are the `true` and `false` branch. Note that these are closures, which
means that the body of the branch is not evaluated for the branch that is not taken.

## Rendering Lists

The `For::each()` method takes two arguments: A closure which returns an iterator,
and a closure which renders a view for each element. Internally the `For` view keeps track
of the array elements, and does a `diff` when the array changes, so that only the elements
that actually changed are re-rendered.

```rust
element.children(
    For::each(
        |cx| {
            let counter = cx.use_resource::<Counter>();
            [counter.count, counter.count + 1, counter.count + 2].into_iter()
        },
        |item| format!("item: {}", item),
    ))
```

There is also `For::index` which doesn't do this diffing, and operates strictly by array
index.

## Presenters

A "presenter" is a function which can be called as a child view. Call `.bind()` to associate
the presenter with a set of properties. The function will not be called right away, but only
during the build phase.

```rust
fn setup_view_root(mut commands: Commands) {
    commands.spawn(ViewRoot::new(
        Element::<NodeBundle>::new()
            .children((
                nested_presenter.bind(()),
            )),
    ));
}

fn nested_presenter(cx: &mut Cx) -> impl View {
    Element::<NodeBundle>::new()
}
```

## Styles

An earlier version of this library implemented "CSS-like" stylesheets with dynamic selectors
and animation, but the current version provides a much simpler solution: "styles are just
functions":

```rust
fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .padding_left(12)
        .padding_right(12)
        .border(1)
        .border_color(Color::WHITE);
}

fn style_button_size_md(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xxxs.height());
}

Element::<NodeBundle>::for_entity(id)
    .named("button")
    .with_styles((
        style_button,
        style_button_size_md,
    ))
```

The `StyleBuilder` argument provides a fluent interface that allows the entity's styles to
be modified with lots of CSS-like shortcuts. For example, the following are all equivalent:

* `.border(ui::UiRect::all(ui::Val::Px(10.)))` -- a border of 10px on all sides.
* `.border(ui::Val::Px(10.))` -- Scalar is automatically converted to a rect.
* `.border(10.)` -- `Px` is assumed to be the default unit.
* `.border(10)` -- Integers are automatically converted to f32 type.

Unlike the CSS approach, there is no support for selectors, animated transitions, or serialization.
The style functions are simply executed once, in order, during the build phase. The main advantage
is that it provides a way to re-use styles without having to repeat the same properties over
and over again.

## Hover Signal

The `CreateHoverSignal` trait adds a `.create_hover_signal(entity)` method to `Cx`. This
creates a derived reactive signal which can be used in conjunction with
[bevy_mod_picking](https://github.com/aevyrie/bevy_mod_picking) to determine whether an element,
or one of it's descendants, is currently being hovered by the mouse.

```rust
let button_id = cx.create_entity();
let hovering = cx.create_hover_signal(button_id);

Element::<NodeBundle>::from_id(button_id)
    .insert(BorderColor::default())
    .create_effect(move |cx, ent| {
        let is_pressed = pressed.get(cx);
        let is_hovering = hovering.get(cx);
        let mut border = cx.world_mut().get_mut::<BorderColor>(ent).unwrap();
        border.0 = match (is_pressed, is_hovering) {
            (true, _) => Color::WHITE,
            (false, true) => Color::LIME_GREEN,
            (false, false) => Color::RED,
        };
    })
```

The function takes an entity id as input; to use this effectively you'll want to pre-allocate
the entity id before creating the view nodes.
