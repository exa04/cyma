// TODO: Document stuff
// TODO: Provide a builder or something for the Inlet
// TODO: Multi-Outlet - 1 input, multiple outputs
// TODO: Stereo-In/Outlets
// TODO: Settle on a fitting skeumorphism ("outlet consumer" sounds kinda weird - might just be me)

use arc_swap::ArcSwap;
use crossbeam_queue::ArrayQueue;
use nih_plug::buffer::Buffer;
use nih_plug::prelude::AtomicF32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{atomic, Arc};

pub trait Outlet {
    type Consumer: OutletConsumer + 'static;
    fn get_consumer(&self) -> Self::Consumer;
}

pub trait OutletConsumer: Send + Sync {
    fn receive(&self) -> Arc<Vec<f32>>;
    fn get_sample_rate(&self) -> f32;
}

pub struct MonoInlet {
    buffer: Arc<ArrayQueue<f32>>,
    sample_rate: Arc<AtomicF32>,
}

impl Default for MonoInlet {
    fn default() -> Self {
        Self {
            buffer: Arc::new(ArrayQueue::new(4096)),
            sample_rate: Default::default(),
        }
    }
}

impl MonoInlet {
    #[inline]
    pub fn enqueue_buffer_summing(&mut self, buffer: &mut Buffer) {
        let channels = buffer.channels() as f32;
        for mut x in buffer.iter_samples() {
            self.buffer
                .force_push(x.iter_mut().map(|x| *x).sum::<f32>() / channels);
        }
    }
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
    }
    pub fn try_send(&mut self, value: f32) {
        self.buffer.force_push(value);
    }
    pub fn create_outlet(&mut self) -> MonoOutlet {
        MonoOutlet {
            buffer: self.buffer.clone(),
            sample_rate: self.sample_rate.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MonoOutlet {
    buffer: Arc<ArrayQueue<f32>>,
    sample_rate: Arc<AtomicF32>,
}

impl Outlet for MonoOutlet {
    type Consumer = MonoOutlet;
    fn get_consumer(&self) -> Self::Consumer {
        self.clone()
    }
}

impl OutletConsumer for MonoOutlet {
    #[inline]
    fn receive(&self) -> Arc<Vec<f32>> {
        let mut new_block = Vec::new();
        while let Some(x) = self.buffer.pop() {
            new_block.push(x);
        }
        new_block.into()
    }
    fn get_sample_rate(&self) -> f32 {
        self.sample_rate.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct MonoMultiOutlet {
    total_consumers: Arc<AtomicU32>,
    read_state: Arc<AtomicU32>,
    buffer: Arc<ArrayQueue<f32>>,
    current_block: Arc<ArcSwap<Vec<f32>>>,
}
