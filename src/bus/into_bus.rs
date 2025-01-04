// use core::slice;
// use nih_plug::prelude::AtomicF32;
// use std::{
//     iter::{Copied, Map},
//     sync::{atomic::Ordering, Arc},
// };

// use super::*;

// pub struct IntoMonoBus<const C: usize, D>
// where
//     for<'a> D: FnMut([f32; C]) -> f32 + 'static + Copy + Clone,
// {
//     dispatchers: DispatcherList<C>,
//     sample_rate: Arc<AtomicF32>,
//     downmixer: D,
// }

// impl<const C: usize, D> IntoMonoBus<C, D>
// where
//     for<'a> D: FnMut([f32; C]) -> f32 + 'static + Copy + Clone,
// {
//     pub fn new(dispatchers: DispatcherList<C>, sample_rate: Arc<AtomicF32>, downmixer: D) -> Self {
//         Self {
//             dispatchers,
//             sample_rate,
//             downmixer,
//         }
//     }
// }

// impl<const C: usize, D> Bus for IntoMonoBus<C, D>
// where
//     for<'a> D: FnMut([f32; C]) -> f32 + 'static + Copy + Clone,
// {
//     type Input = f32;
//     type InputIter<'a> = slice::Iter<'a, f32>;
//     type Output = [f32; C];
//     type OutputIter<'a> = slice::Iter<'a, [f32; C]>;

//     fn set_sample_rate(&self, sample_rate: f32) {
//         self.sample_rate.store(sample_rate, Ordering::Relaxed);
//     }

//     fn sample_rate(&self) -> f32 {
//         self.sample_rate.load(Ordering::Relaxed)
//     }

//     fn update(&self) {
//         // TODO
//     }

//     fn register_dispatcher<F>(&self, action: F) -> Rc<dyn for<'a> Fn(Self::OutputIter<'a>)>
//     where
//         F: for<'a> Fn(Self::InputIter<'a>) + 'static,
//     {
//         let mut downmix = self.downmixer.clone();

//         let dispatcher: Rc<dyn for<'a> Fn(Self::OutputIter<'a>)> = Rc::new(move |data| {
//             let mapped = data.copied().map(move |x| downmix(x)).collect::<Vec<_>>();
//             action(mapped.iter())
//         });

//         self.dispatchers
//             .write()
//             .unwrap()
//             .push(Rc::downgrade(&dispatcher));

//         dispatcher
//     }
// }
