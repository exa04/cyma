# Displaying a Graph

Now for the fun part! Let's add a graph to our editor. The Graph is a view that
takes in a lens to some buffer, a range of values it can display, and a
`ValueScaling`, so a type of scaling it should apply to the data.

We want our graph to show us the range of -32 dB up to 8 dB, and we want to
scale our data as decibels. So, let's just add a Graph with exactly these
parameters.

```rust
// editor.rs
pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);

        Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels);
    })
}
```

Next, we'll want to style our graph, and we could either do this via CSS or by
using style modifiers. For colocation's sake, we'll go with style modifiers, but
this really boils down to personal preference. We have two style modifiers at
our disposal here; `background_color` and `color`.

  - `background_color` modifies the fill color of the graph
  - `color` modifies the stroke color of the graph

If we also want to change the color of the graph's backdrop, we can put it
inside a `ZStack` and then change the stack's background color.

```rust
// editor.rs
pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);

        ZStack::new(cx, |cx| {
            Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
        })
        .background_color(Color::rgb(16, 16, 16));
    })
}
```