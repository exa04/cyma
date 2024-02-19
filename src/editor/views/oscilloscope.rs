use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::PeakWaveformRingBuffer;

/// A peak Oscilloscope.
///
/// Takes in a PeakWaveformRingBuffer to display the waveform, retaining peak
/// details for all frequencies within the sample rate, regardless of buffer
/// size.
///
/// # How to use
///
/// To use this Visualizer, you need a PeakWaveformBuffer. It also needs to be
/// made thread-safe so that you can send it to the editor thread. So, usually,
/// you'll need to follow these steps:
///
/// - Add a an `Arc<Mutex<PeakWaveformRingBuffer<f32, SIZE>>>` to your plugin struct (providing your own `SIZE` - typically, anything from 500 to 5000 is fine)
/// - Initialize the buffer by calling `set_sample_rate()` on it inside your plugin's `initialize()` function
/// - Push samples into the buffer inside your plugin's `process()` function
/// - Send a `clone()` of the `Arc<Mutex>` to the editor thread via `editor::create()`
///
/// Then, inside the editor's `create()` function, you'll only need to:
///
/// - Call `Oscilloscope::new()` and pass the `Arc` into it.
///
/// # Example
///
/// Here's a (somewhat opinionated) and very long example. Note that this
/// example only pushes the left channel data into the oscilloscope.
///
/// ## Plugin code (`lib.rs`)
///
/// ```no_run
/// pub struct YourPlugin {
///     oscilloscope_buffer: Arc<Mutex<PeakWaveformRingBuffer<f32, 1024>>>,
///     ...
/// }
///
/// impl Default for YourPlugin {
///     fn default() -> Self {
///         Self {
///             oscilloscope_buffer: Arc::new(Mutex::new(PeakWaveformRingBuffer::new(44100., 20.))),
///             ...
///         }
///     }
/// }
///
/// impl Plugin for YourPlugin {
///     fn process(
///        &mut self,
///        buffer: &mut Buffer,
///        _aux: &mut AuxiliaryBuffers,
///        _context: &mut impl ProcessContext<Self>,
///     ) -> ProcessStatus {
///         ...
///         for sample in buffer.iter_samples()[0] {
///             self.oscilloscope_buffer
///                 .lock()
///                 .unwrap()
///                 .enqueue(sample.clone());
///         }
///         ...
///     }
///
///     fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
///         editor::create(
///             editor::Data {
///                 oscilloscope_buffer: self.oscilloscope_buffer.clone(),
///             },
///             ...
///         )
///     }
///
///     ...
///
/// }
/// ```
///
/// ## Editor code (`lib.rs`)
///
/// ```no_run
/// #[derive(Lens, Clone)]
/// pub(crate) struct Data {
///     pub(crate) oscilloscope_buffer: Arc<Mutex<PeakWaveformRingBuffer<f32, 1024>>>,
///     ...
/// }
///
/// pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
///     create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
///         editor_data.clone().build(cx);
///         Oscilloscope::new(cx, Data::post_buffer);
///     })
/// }
/// ```
pub struct Oscilloscope<B, const BUFFER_SIZE: usize>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
{
    buffer: B,
}

impl<B, const BUFFER_SIZE: usize> Oscilloscope<B, BUFFER_SIZE>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
{
    pub fn new(cx: &mut Context, buffer: B) -> Handle<Self> {
        Self { buffer }.build(cx, |cx| {})
    }
}

impl<B, const BUFFER_SIZE: usize> View for Oscilloscope<B, BUFFER_SIZE>
where
    B: Lens<Target = Arc<Mutex<PeakWaveformRingBuffer<f32, BUFFER_SIZE>>>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("22-visualizer")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        // Background
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                path.move_to(x, y);
                path.line_to(x + w, y);
                path.line_to(x + w, y + h);
                path.line_to(x, y + h);
                path.close();
                path
            },
            &vg::Paint::color(cx.background_color().into()),
        );

        // Waveform
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.buffer.get(cx);
                let ring_buf = &(binding.lock().unwrap()).ring_buffer;

                path.move_to(x, y + h / 2.);

                let mut i = 0.;
                for v in ring_buf.into_iter() {
                    path.line_to(
                        x + (w / BUFFER_SIZE as f32) * i,
                        y + (h / 2.) * (1. - v.0) + 1.,
                    );
                    i += 1.;
                }
                for v in ring_buf.into_iter().rev() {
                    i -= 1.;
                    path.line_to(
                        x + (w / BUFFER_SIZE as f32) * i,
                        y + (h / 2.) * (1. - v.1) + 1.,
                    );
                }
                path.close();
                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(line_width),
        );
    }
}
