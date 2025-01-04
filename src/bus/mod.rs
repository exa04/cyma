use std::sync::Arc;

mod into_bus;
mod mono;
mod multichannel;

pub use into_bus::*;
pub use mono::*;
pub use multichannel::*;

pub type StereoBus = MultiChannelBus<2>;

pub trait Bus<T: Clone + Copy + Sized + 'static>: Clone {
    type I<'a>: ExactSizeIterator<Item = &'a T>;
    type O<'a>: Iterator;

    fn set_sample_rate(&self, sample_rate: f32);
    fn sample_rate(&self) -> f32;
    fn update(&self);
    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::O<'a>) + Sync + Send>;
}
