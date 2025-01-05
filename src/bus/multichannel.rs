use core::slice;
use crossbeam_channel::{bounded, Receiver, Sender};
use nih_plug::buffer::Buffer;
use nih_plug::nih_dbg;
use nih_plug::prelude::AtomicF32;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc, RwLock, Weak};

use super::*;

#[derive(Clone)]
pub struct MultiChannelBus<const C: usize> {
    dispatchers: Arc<RwLock<Vec<Weak<dyn Fn(slice::Iter<'_, [f32; C]>) + Sync + Send>>>>,
    channel: (Sender<[f32; C]>, Receiver<[f32; C]>),
    sample_rate: Arc<AtomicF32>,
}

impl<const C: usize> MultiChannelBus<C> {
    pub fn new(size: usize) -> Self {
        let channel = bounded(size);
        Self {
            dispatchers: RwLock::new(vec![]).into(),
            channel,
            sample_rate: Arc::new(f32::NAN.into()),
        }
    }
}

impl<const C: usize> Default for MultiChannelBus<C> {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl<const C: usize> MultiChannelBus<C> {
    #[inline]
    pub fn send_buffer(&self, buffer: &mut Buffer) {
        for mut x in buffer.iter_samples() {
            let mut array = [0.0; C];

            for i in 0..C {
                if let Some(sample) = x.get_mut(i) {
                    array[i] = sample.clone();
                } else {
                    break;
                }
            }

            nih_dbg!(&array);
            self.send(array);
        }
    }

    #[inline]
    pub fn send(&self, value: [f32; C]) {
        self.channel.0.try_send(value);
    }

    // pub fn into_mono<D>(self, downmixer: D) -> IntoMonoBus<C, D>
    // where
    //     for<'a> D: Fn([f32; C]) -> f32 + 'static + Copy + Clone,
    // {
    //     IntoMonoBus {
    //         bus: self,
    //         downmixer,
    //     }
    // }

    // pub fn into_mono_summing(
    //     self,
    // ) -> IntoMonoBus<C, impl Fn([f32; C]) -> f32 + 'static + Copy + Clone> {
    //     self.into_mono(|sample| sample.into_iter().sum::<f32>() / C as f32)
    // }

    // pub fn into_mono_from_channel_index(
    //     self,
    //     channel: usize,
    // ) -> IntoMonoBus<C, impl Fn([f32; C]) -> f32 + 'static + Copy + Clone> {
    //     self.into_mono(move |sample| sample[channel])
    // }
}

impl<const C: usize> Bus<[f32; C]> for MultiChannelBus<C> {
    type I<'a> = slice::Iter<'a, [f32; C]>;
    type O<'a> = Self::I<'a>;

    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::I<'a>) + Sync + Send> {
        let dispatcher: Arc<dyn for<'a> Fn(Self::I<'a>) + Sync + Send> = Arc::new(dispatcher);

        let downgraded = Arc::downgrade(&dispatcher);

        let mut dispatchers = self.dispatchers.write().unwrap();

        if let Some(pos) = dispatchers.iter().position(|d| d.upgrade().is_none()) {
            dispatchers[pos] = downgraded;
            dispatchers.retain(|d| d.upgrade().is_some());
        } else {
            dispatchers.push(downgraded);
        }

        dispatcher
    }

    fn update(&self) {
        let samples = self.channel.1.try_iter().collect::<Vec<_>>();

        if samples.is_empty() {
            return;
        }

        self.dispatchers
            .read()
            .unwrap()
            .iter()
            .filter_map(|d| d.upgrade())
            .for_each(|d| d(samples.iter()));
    }

    fn set_sample_rate(&self, sample_rate: f32) {
        self.sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate.load(Ordering::Relaxed)
    }
}
