//! The main means of inter-process communication in Cyma.

use std::{any::Any, hint::spin_loop, marker::PhantomData, sync::Arc, thread, time::Duration};

mod into_bus;
mod mono;
mod multichannel;

pub use into_bus::*;
pub use mono::*;
pub use multichannel::*;
use vizia_plug::vizia::prelude::*;

/// A bus for stereo data.
pub type StereoBus = MultiChannelBus<2>;

/// A MPMC system for sending samples and processing them via some dispatchers.
///
/// A Bus can receive audio data from the Plugin thread and send it to some
/// dispatchers which are dynamically added on the GUI thread. In this way, it
/// "fans out" new signal data to visualizers.
pub trait Bus<T: Clone + Copy + Sized + 'static>: Clone + Send + Sync
where
    Self: 'static,
{
    type I<'a>: ExactSizeIterator<Item = &'a T>;
    type O<'a>: Iterator;

    /// Informs the Bus and its subscribers of the current sample rate.
    ///
    /// Call this inside your plugin's [`initialize`](nih_plug::prelude::Plugin::initialize)
    /// function.
    fn set_sample_rate(&self, sample_rate: f32);

    /// The current sample rate.
    fn sample_rate(&self) -> f32;

    /// Calls all registered dispatchers and provides them with the latest
    /// audio data, if any is available.
    fn update(&self, cx: &mut ContextProxy);

    /// Registers a new dispatcher and returns a handle to it.
    ///
    /// When the handle goes out of scope, the dispatcher will not be called
    /// anymore. Visualizers need to store it so that it will keep on being called.
    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::O<'a>) + Send + Sync>;

    /// Spawns a new thread that will continuously call [`update`](Self::update),
    /// so long as the GUI lives.
    fn subscribe(self: &Arc<Self>, cx: &mut Context) {
        let bus = self.clone();
        cx.spawn(move |cx| loop {
            bus.update(cx);
            thread::sleep(Duration::from_millis(15));
        });
    }
}
