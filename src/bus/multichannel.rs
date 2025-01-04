// use core::slice;
// use crossbeam_channel::{bounded, Receiver, Sender};
// use nih_plug::buffer::Buffer;
// use nih_plug::prelude::AtomicF32;
// use std::rc::Rc;
// use std::sync::atomic::Ordering;
// use std::sync::{atomic, Arc, RwLock};

// use super::*;

// type DispatcherList<const C: usize> =
//     Arc<RwLock<Vec<Weak<dyn Fn(<&Vec<[f32; C]> as IntoIterator>::IntoIter)>>>>;

// #[derive(Clone)]
// pub struct MultiChannelBus<const C: usize> {
//     dispatchers: DispatcherList<C>,
//     channel: (Sender<[f32; C]>, Receiver<[f32; C]>),
//     sample_rate: Arc<AtomicF32>,
// }

// impl<const C: usize> MultiChannelBus<C> {
//     pub fn new(size: usize) -> Self {
//         let channel = bounded(size);
//         Self {
//             dispatchers: RwLock::new(vec![]).into(),
//             channel,
//             sample_rate: Arc::new(f32::NAN.into()),
//         }
//     }
// }

// impl<const C: usize> Default for MultiChannelBus<C> {
//     fn default() -> Self {
//         Self::new(4096)
//     }
// }

// impl<const C: usize> MultiChannelBus<C> {
//     #[inline]
//     pub fn send_buffer(&mut self, buffer: &mut Buffer) {
//         for mut x in buffer.iter_samples() {
//             let mut array = [0.0; C];

//             for i in 0..C {
//                 if let Some(sample) = x.get_mut(i) {
//                     array[i] = sample.clone();
//                 } else {
//                     break;
//                 }
//             }

//             self.send(array);
//         }
//     }

//     #[inline]
//     pub fn send(&self, value: [f32; C]) {
//         self.channel.0.try_send(value);
//     }
// }

// impl<const C: usize> Bus for MultiChannelBus<C> {
//     type Input = [f32; C];
//     type InputIter<'a> = slice::Iter<'a, [f32; C]>;
//     type Output = Self::Input;
//     type OutputIter<'a> = Self::InputIter<'a>;

//     fn set_sample_rate(&self, sample_rate: f32) {
//         self.sample_rate
//             .store(sample_rate, atomic::Ordering::Relaxed);
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
//         let dispatcher: Rc<dyn for<'a> Fn(Self::OutputIter<'a>)> = Rc::new(action);

//         self.dispatchers
//             .write()
//             .unwrap()
//             .push(Rc::downgrade(&dispatcher));

//         dispatcher
//     }
// }
