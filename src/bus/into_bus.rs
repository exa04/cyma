use std::{iter::Map, sync::Arc};

use super::*;

/// Thinly wraps around a [`MultiChannelBus`] and acts like a mono bus.
///
/// Also contains a downmixing function which is called on the incoming audio to
/// allow for dispatchers to work with the audio as if it were mono.
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

    fn update(&self, cx: &mut ContextProxy) {
        self.bus.update(cx)
    }

    #[inline]
    fn set_sample_rate(&self, sample_rate: f32) {
        self.bus.set_sample_rate(sample_rate)
    }

    #[inline]
    fn sample_rate(&self) -> f32 {
        self.bus.sample_rate()
    }
}
