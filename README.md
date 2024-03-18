<div align="center">
  <h1>Plext</h1>
</div>
<div align="center">
  <a href="https://github.com/223230/plext/actions/workflows/test.yml"><img src="https://github.com/223230/plext/actions/workflows/test.yml/badge.svg"></a>
  <a href="https://github.com/223230/plext/actions/workflows/docs.yml"><img src="https://github.com/223230/plext/actions/workflows/docs.yaml/badge.svg"></a>
</div>
<br/>
<div align="center">
  Views and associated data structures for plug-in UIs made using <a href="https://github.com/robbert-vdh/nih-plug">nih-plug</a> and <a href="https://github.com/vizia/vizia">VIZIA</a>.
</div>

> [!CAUTION]
> **Don't use this in production!** This library is *very* early in development
> and a stable release is yet to be made. Expect frequent breaking changes.

<hr/>

## Overview

**Plext** is a collection of flexible, composable views that you can use to make
any plug-in UI with ease. Here's an example:

## 🧰 What's included

Check out [this](https://github.com/223230/plext/milestone/1) milestone to see
what views will eventually be added. Do you think something's missing? File a
feature request so it can be added!

### 📊 Visualizers

**General/Utility**
  - Grid backdrop
  - Unit ruler

**Peak/Waveform Analysis**
  - Peak graph
  - Oscilloscope
  - Static waveform

### 🎛️ Controls

### 🛠️ Utils

**Buffers**
  - **RingBuffer** - A generic circular buffer
  - **WaveformBuffer** - A buffer for waveform analysis
  - **PeakBuffer** - A buffer for peak analysis

## 🍔 Composing views

Plext's most powerful feature is composability.

For example, by combining views such as the `Grid`, `UnitRuler`, and
`PeakGraph`, you can make this real-time peak analyzer that you can style
however you want.

![Peak visualizer](doc/composability_demo.png)

```rust
fn peak_graph(cx: &mut Context) {
    HStack::new(cx, |cx| {
        ZStack::new(cx, |cx| {
            Grid::new(
                cx,
                (-32.0, 8.0),
                0.0,
                vec![6.0, 0.0, -6.0, -12.0, -18.0, -24.0, -30.0],
            )
            .color(Color::rgb(60, 60, 60));

            PeakGraph::new(cx, Data::peak_buffer, (-32.0, 8.0), true)
                .color(Color::rgba(255, 255, 255, 160))
                .background_color(Color::rgba(255, 255, 255, 60));
        })
        .border_color(Color::rgb(80, 80, 80))
        .border_width(Pixels(1.))
        .background_color(Color::rgb(16, 16, 16));

        UnitRuler::new(
            cx,
            (-32.0, 8.0),
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
        .width(Pixels(32.));
    })
    .col_between(Pixels(8.));
}
```

## 🙋 Contributing

This project is in a really early stage, which is why I won't be accepting code
contributions just yet. If you want to contribute, you can feel free to play
around with it and report any bugs, glitches, or other oddities by filing an
[issue](https://github.com/223230/plext/issues).

## 📃 License

This project is licensed under the [MPL](LICENSE).