# Setting up a PeakBuffer

Let's start off by adding a `PeakBuffer` to our Plug-in. This buffer will store
the last 10 seconds of audio, which we will then pass to the editor to draw our
peak graph.

Since we'll use this buffer to store samples on the plugin thread and to display
a graph on the editor thread, we need to make it thread-safe. So let's just
wrap it in an `Arc<Mutex>>`.

```rust
// lib.rs
pub struct PeakGraphPlugin {
    params: Arc<DemoParams>,
    peak_buffer: Arc<Mutex<PeakBuffer>>,
}
```

Now, we need to provide a default for this peak buffer inside the `default()`
function. So we'll call `PeakBuffer::new()`.

This function takes in a size, sample rate, and duration, and creates a
`PeakBuffer` according to these parameters. Based on them, the buffer enqueues
the last seconds of audio. It's usually kept quite small - we'll go with 800
samples over 10 seconds. As for our sample rate, we'll just go with 44.1 kHz for
now.

```rust
// lib.rs
impl Default for PeakGraphPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DemoParams::default()),
            peak_buffer: Arc::new(Mutex::new(PeakBuffer::new(800, 44100.0, 10.0))),
        }
    }
}
```

Despite the buffer's small size, it will still store all relevant peak
information. It does this by keeping track of the local maxima within the last
10 seconds of audio.

> Other buffers, like the `MinimaBuffer` and `OscilloscopeBuffer` work in quite
> a similar way, where they accumulate local maxima and minima to retain
> information and avoid naïve downsampling.

There's a glaring issue with this, though. The host could set any sample rate,
and the buffer would still work as if it's dealing with 44.1 kHz audio. So,
let's set its sample rate to the actual host sample rate once we know it. We'll
lock the `peak_buffer` mutex to gain access to the buffer, and then we'll set
the sample rate.

```rust
// lib.rs
fn initialize(
    &mut self,
    _audio_io_layout: &AudioIOLayout,
    buffer_config: &BufferConfig,
    _context: &mut impl InitContext<Self>,
) -> bool {
    match self.peak_buffer.lock() {
        Ok(mut buffer) => {
            buffer.set_sample_rate(buffer_config.sample_rate);
        }
        Err(_) => return false,
    }

    true
}
```

Now, we can add our peak buffer to the editor's `Data` struct.

```rust
// editor.rs
#[derive(Lens, Clone)]
pub(crate) struct Data {
    peak_buffer: Arc<Mutex<PeakBuffer>>,
}
impl Data {
    pub(crate) fn new(peak_buffer: Arc<Mutex<PeakBuffer>>) -> Self {
        Self { peak_buffer }
    }
}
```

And finally, when we call the editor's `create` function from our plugin, we can
pass the `peak_buffer` by cloning a reference to it.

```rust
// lib.rs
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    editor::create(
        editor::Data::new(self.peak_buffer.clone()),
        self.params.editor_state.clone(),
    )
}
```

And just like that, we've now added a `PeakGraph` to our plug-in, to which the
plug-in thread writes, and from which the editor thread reads. Great! We will
now be able to use Views inside the editor to display our data.