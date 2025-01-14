use std::{iter::Map, sync::Arc};

use super::*;

#[derive(Clone)]
pub struct IntoMonoBus<const C: usize, D>
where
    for<'a> D: Fn(&'a [f32; C]) -> &'a f32 + 'static + Copy + Clone + Send + Sync,
{
    pub(crate) bus: MultiChannelBus<C>,
    pub(crate) downmixer: D,
}

impl<const C: usize, D> Bus<f32> for IntoMonoBus<C, D>
where
    for<'a> D: Fn(&'a [f32; C]) -> &'a f32 + 'static + Copy + Clone + Send + Sync,
{
    type I<'a> = Map<Self::O<'a>, D>;
    type O<'a> = <MultiChannelBus<C> as Bus<[f32; C]>>::I<'a>;

    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::O<'a>) + Sync + Send> {
        let downmixer = self.downmixer.clone();
        self.bus.register_dispatcher(move |samples| {
            let mono_samples = samples.map(downmixer);
            dispatcher(mono_samples);
        })
    }

    fn update(&self) {
        self.bus.update()
    }

    #[inline]
    fn set_sample_rate(&self, sample_rate: f32) {
        self.bus.set_sample_rate(sample_rate)
    }

    fn is_empty(&self) -> bool {
        self.bus.is_empty()
    }

    #[inline]
    fn sample_rate(&self) -> f32 {
        self.bus.sample_rate()
    }
}
