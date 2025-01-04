use std::{
    rc::{Rc, Weak},
    sync::{Arc, RwLock},
};

mod mono;
mod multichannel;

pub use mono::*;
pub use multichannel::*;

pub trait Bus<T: Clone + Copy + Sized + 'static>: Clone {
    type I<'a>: Iterator<Item = &'a T>;

    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::I<'a>) + Sync + Send>;
    fn update(&self);
    fn set_sample_rate(&self, sample_rate: f32);
    fn sample_rate(&self) -> f32;
}
