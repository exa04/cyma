use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};

use crate::utils::PeakRingBuffer;

/// Displays a waveform, retaining peak details for all frequencies within the
/// sample rate, regardless of buffer size.
///
/// This visualizer is particularly useful when visualizing audio data at a
/// high sample rate, such as 44.1kHz, in a much smaller view. It does not
/// downsample the audio, which is why, even for very small sizes, it still
/// correctly displays the peak data.
///
/// # How to use
///
/// To use this Visualizer, you need a [`PeakWaveformBuffer`](`crate::utils::ring_buffer::PeakRingBuffer`)
/// that you write to inside your plugin code, and then send to the editor
/// thread. Here's a step-by-step tutorial on how to achieve this setup.
///
/// Add a an `Arc<Mutex<PeakRingBuffer<f32, SIZE>>>` to your plugin struct.
///
/// *lib.rs*
/// ```ignore
/// pub struct YourPlugin {
///     oscilloscope_buffer: Arc<Mutex<PeakRingBuffer<f32, 1024>>>,
/// }
/// ```
///
/// *lib.rs*
/// ```ignore
/// impl Default for YourPlugin {
///     fn default() -> Self {
///         Self {
///             oscilloscope_buffer: Arc::new(Mutex::new(PeakRingBuffer::new(44100., 20.))),
///         }
///     }
/// }
/// ```
///
/// Call [`set_sample_rate()`](`crate::utils::PeakRingBuffer::set_sample_rate()`)
/// on the buffer when the sample rate is known.
///
/// *lib.rs*
/// ```ignore
/// impl Plugin for YourPlugin {
///     fn initialize(
///         &mut self,
///         _audio_io_layout: &AudioIOLayout,
///         buffer_config: &BufferConfig,
///         _context: &mut impl InitContext<Self>,
///     ) -> bool {
///         match self.visualizer_post_buffer.lock() {
///             Ok(mut buffer) => {
///                 buffer.set_sample_rate(buffer_config.sample_rate);
///             }
///             Err(_) => {
///                 // Your error handling here
///             }
///         }
///         true
///     }
/// }
/// ```
///
/// Push samples into the buffer inside your plugin's `process()` function.
///
/// *lib.rs*
/// ```ignore
/// impl Plugin for YourPlugin {
///     fn process(
///        &mut self,
///        buffer: &mut Buffer,
///        _aux: &mut AuxiliaryBuffers,
///        _context: &mut impl ProcessContext<Self>,
///     ) -> ProcessStatus {
///         for sample in buffer.iter_samples()[0] {
///             self.oscilloscope_buffer
///                 .lock()
///                 .unwrap()
///                 .enqueue(sample.clone());
///         }
///     }
/// }
/// ```
///
/// Add a new field to your editor's data struct, so you can send the buffer data to it.
///
/// *editor.rs*
/// ```ignore
/// #[derive(Lens, Clone)]
/// pub(crate) struct Data {
///     pub(crate) oscilloscope_buffer: Arc<Mutex<PeakRingBuffer<f32, 1024>>>,
///     ...
/// }
/// ```
///
/// Inside the editor's `create()` function, call `Oscilloscope::new()` and pass the buffer into it.
///
/// *editor.rs*
/// ```ignore
/// pub(crate) fn create(editor_data: Data, editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
///     create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
///         editor_data.clone().build(cx);
///         Oscilloscope::new(cx, Data::oscilloscope_buffer);
///     })
/// }
/// ```
///
/// Send the buffer to your editor by passing it through your plugin's `editor()` function.
///
/// *lib.rs*
/// ```ignore
/// impl Plugin for YourPlugin {
///     fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
///         editor::create(
///             editor::Data {
///                 oscilloscope_buffer: self.oscilloscope_buffer.clone(),
///             },
///             ...
///         )
///     }
/// }
/// ```
pub struct Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<PeakRingBuffer<f32>>>>,
{
    buffer: B,
}

impl<B> Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<PeakRingBuffer<f32>>>>,
{
    ///     Creates a new Oscilloscope.
    ///    
    ///     Takes in a `buffer`, which should be used to store the peak values. You
    ///     need to write to it inside your plugin code, thread-safely send it to
    ///     the editor thread, and then pass it into this oscilloscope. Which is
    ///     also why it is behind an `Arc<Mutex>`.
    pub fn new(cx: &mut Context, buffer: B) -> Handle<Self> {
        Self { buffer }.build(cx, |_| {})
    }
}

impl<B> View for Oscilloscope<B>
where
    B: Lens<Target = Arc<Mutex<PeakRingBuffer<f32>>>>,
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
                let ring_buf = &(binding.lock().unwrap());

                path.move_to(x, y + h / 2.);

                let mut i = 0.;
                for v in ring_buf.into_iter() {
                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
                        y + (h / 2.) * (1. - v.0) + 1.,
                    );
                    i += 1.;
                }
                for v in ring_buf.into_iter().rev() {
                    i -= 1.;
                    path.line_to(
                        x + (w / ring_buf.len() as f32) * i,
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
