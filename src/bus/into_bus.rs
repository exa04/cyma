// use core::slice;
// use std::sync::Arc;

// use super::*;

// #[derive(Clone)]
// pub struct IntoMonoBus<const C: usize, D>
// where
//     for<'a> D: Fn([f32; C]) -> f32 + 'static + Copy + Clone,
// {
//     pub(crate) bus: MultiChannelBus<C>,
//     pub(crate) downmixer: D,
// }

// impl<const C: usize, D> Bus<f32> for IntoMonoBus<C, D>
// where
//     for<'a> D: Fn([f32; C]) -> f32 + 'static + Copy + Clone,
// {
//     type I<'a> = slice::Iter<'a, f32>;
//     type O<'a> = <MultiChannelBus<C> as Bus<[f32; C]>>::I<'a>;

//     fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
//         &self,
//         dispatcher: F,
//     ) -> Arc<dyn for<'a> Fn(Self::O<'a>) + Sync + Send> {
//         self.bus.register_dispatcher(move |samples| {
//             let mono_samples = samples.map(|x| x.into_iter().sum::<f32>() / C as f32);
//         })
//     }

//     fn update(&self) {
//         self.bus.update()
//     }

//     #[inline]
//     fn set_sample_rate(&self, sample_rate: f32) {
//         self.bus.set_sample_rate(sample_rate)
//     }

//     #[inline]
//     fn sample_rate(&self) -> f32 {
//         self.bus.sample_rate()
//     }
// }
