use core::slice;
use crossbeam_channel::{bounded, Receiver, Sender};
use nih_plug::buffer::Buffer;
use nih_plug::prelude::AtomicF32;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc, RwLock, Weak};

use super::*;

#[derive(Clone)]
pub struct MonoBus {
    dispatchers: Arc<RwLock<Vec<Weak<dyn Fn(slice::Iter<'_, f32>) + Sync + Send>>>>,
    channel: (Sender<f32>, Receiver<f32>),
    sample_rate: Arc<AtomicF32>,
}

impl MonoBus {
    pub fn new(size: usize) -> Self {
        let channel = bounded(size);
        Self {
            dispatchers: RwLock::new(vec![]).into(),
            channel,
            sample_rate: Arc::new(f32::NAN.into()),
        }
    }
}

impl Default for MonoBus {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl MonoBus {
    #[inline]
    pub fn send_buffer_summing(&self, buffer: &mut Buffer) {
        let channels = buffer.channels();

        if channels == 1 {
            for mut x in buffer.iter_samples() {
                self.send(*x.get_mut(0).unwrap());
            }
        } else {
            for mut x in buffer.iter_samples() {
                self.send(x.iter_mut().map(|x| *x).sum::<f32>() / channels as f32);
            }
        }
    }

    #[inline]
    pub fn send(&self, value: f32) {
        self.channel.0.try_send(value);
    }
}

// TODO Cleanup automatically

impl Bus<f32> for MonoBus {
    type I<'a> = slice::Iter<'a, f32>;
    type O<'a> = Self::I<'a>;

    fn set_sample_rate(&self, sample_rate: f32) {
        self.sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate.load(Ordering::Relaxed)
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

    fn register_dispatcher<F: for<'a> Fn(Self::I<'a>) + Sync + Send + 'static>(
        &self,
        dispatcher: F,
    ) -> Arc<dyn for<'a> Fn(Self::I<'a>) + Sync + Send> {
        let dispatcher: Arc<dyn for<'a> Fn(Self::I<'a>) + Sync + Send> = Arc::new(dispatcher);

        self.dispatchers
            .write()
            .unwrap()
            .push(Arc::downgrade(&dispatcher));

        dispatcher
    }
}
