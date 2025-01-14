use std::{any::Any, hint::spin_loop, marker::PhantomData, sync::Arc, thread, time::Duration};

mod into_bus;
mod mono;
mod multichannel;

pub use into_bus::*;
pub use mono::*;
pub use multichannel::*;
use nih_plug::nih_dbg;
use nih_plug_vizia::vizia::prelude::*;

pub type StereoBus = MultiChannelBus<2>;

pub trait Bus<T: Clone + Copy + Sized + 'static>: Clone + Send + Sync
where
    Self: 'static,
{
    type I<'a>: ExactSizeIterator<Item = &'a T>;
    type O<'a>: Iterator;

    fn set_sample_rate(&self, sample_rate: f32);
    fn sample_rate(&self) -> f32;
    fn update(&self);
    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::O<'a>) + Send + Sync>;

    fn subscribe(self: &Arc<Self>, cx: &mut Context) {
        let bus = self.clone();
        cx.spawn(move |cx| loop {
            bus.update();
            thread::sleep(Duration::from_millis(15));
        });
    }
}
