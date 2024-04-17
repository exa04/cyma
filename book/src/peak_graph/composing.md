# Adding Grid Lines

Now that we've already got a `ZStack` (a view that stacks its child views on top
of each other), lets add some more views. We can add a `Grid` to display grid
lines behind our graph. This view takes in a value scaling, a range, a vector of
values where a grid line should be, and an orientation.

```rust
Grid::new(
    cx,
    ValueScaling::Linear,
    (-32., 8.),
    vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
    Orientation::Horizontal,
)
.color(Color::rgb(60, 60, 60));
```

So let's put it behind the graph! We'll add it as the first child of the
`ZStack`, so that the graph gets drawn above it.

```rust
// editor.rs
pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);

        ZStack::new(cx, |cx| {
            Grid::new(
                cx,
                ValueScaling::Linear,
                (-32., 8.),
                vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
                Orientation::Horizontal,
            )
            .color(Color::rgb(60, 60, 60));

            Graph::new(cx, Data::peak_buffer, (-32., 8.), ValueScaling::Decibels)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
        })
        .background_color(Color::rgb(16, 16, 16));
    })
}
```

This is nice, but we really need a unit ruler for this grid to be useful. Cyma's
`UnitRuler` works in a similar way to the `Grid` - you specify a range of
values, a scaling, and whether it should be oriented horizontally or vertically.
However, instead of a vector of values, you pass a vector of tuples where for
each element, the first tuple value is the position of each label on the ruler,
and the second value is the text on each label.

For our `UnitRuler`, we'll just label every grid line, adhering to the scaling
and range of our graph and grid.

```rust
UnitRuler::new(
    cx,
    (-32.0, 8.0),
    ValueScaling::Linear,
    vec![
        (6.0, "6db"),
        (0.0, "0db"),
        (-6.0, "-6db"),
        (-12.0, "-12db"),
        (-18.0, "-18db"),
        (-24.0, "-24db"),
        (-30.0, "-30db"),
    ],
    Orientation::Vertical,
)
.font_size(12.)
.color(Color::rgb(160, 160, 160))
.width(Pixels(48.));
```

Using an `HStack`, we can place this ruler next to the `ZStack` containing our
graph and grid. 

```rust
// editor.rs
pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        assets::register_noto_sans_light(cx);
        editor_data.clone().build(cx);

        HStack::new(cx, |cx| {
            ZStack::new(cx, |cx| {
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (-32., 8.),
                    vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
                    Orientation::Horizontal,
                )
                .color(Color::rgb(60, 60, 60));

                Graph::new(cx, Data::peak_buffer, (-32.0, 8.0), ValueScaling::Decibels)
                    .color(Color::rgba(255, 255, 255, 160))
                    .background_color(Color::rgba(255, 255, 255, 60));
            })
            .background_color(Color::rgb(16, 16, 16));

            UnitRuler::new(
                cx,
                (-32.0, 8.0),
                ValueScaling::Linear,
                vec![
                    (6.0, "6db"),
                    (0.0, "0db"),
                    (-6.0, "-6db"),
                    (-12.0, "-12db"),
                    (-18.0, "-18db"),
                    (-24.0, "-24db"),
                    (-30.0, "-30db"),
                ],
                Orientation::Vertical,
            )
            .font_size(12.)
            .color(Color::rgb(160, 160, 160))
            .width(Pixels(48.));
        })
        .col_between(Pixels(8.))
        .background_color(Color::rgb(0, 0, 0));
    })
}
```
