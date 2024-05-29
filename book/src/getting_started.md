# Getting Started

Cyma is intended for use with nih-plug and VIZIA. To get started, just add
it to your `Cargo.toml`.

```diff
  [dependencies]
  nih_plug = { ... }
  nih_plug_vizia = { ... }
+ cyma = { git = "https://github.com/223230/cyma" }
```

Then, you can use Cyma where you need it, by using `cyma::prelude::*`. This
will import the most important parts of Cyma.