// TODO: Document stuff
// TODO: Provide a builder or something for the Inlet
// TODO: Multi-Outlet - 1 input, multiple outputs
// TODO: Stereo-In/Outlets
// TODO: Settle on a fitting skeumorphism ("outlet consumer" sounds kinda weird - might just be me)

//use arc_swap::ArcSwap;
use crossbeam_queue::ArrayQueue;
use nih_plug::buffer::Buffer;
use std::sync::{
    // atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

pub struct MonoInlet {
    buffer: Arc<ArrayQueue<f32>>,
    sample_rate: f32,
}

impl Default for MonoInlet {
    fn default() -> Self {
        Self {
            buffer: Arc::new(ArrayQueue::new(4096)),
            sample_rate: 44_100.0,
        }
    }
}

impl MonoInlet {
    #[inline]
    pub fn enqueue_buffer_summing(&mut self, buffer: &mut Buffer) {
        for mut x in buffer.iter_samples() {
            self.buffer.force_push(x.iter_mut().map(|x| *x).sum());
        }
    }
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
    pub fn try_send(&mut self, value: f32) {
        self.buffer.force_push(value);
    }
    pub fn create_outlet(&mut self) -> MonoOutlet {
        MonoOutlet {
            buffer: self.buffer.clone(),
            sample_rate: self.sample_rate,
        }
    }
    /*pub fn create_multi_outlet(&mut self) -> MonoMultiOutlet {
        MonoMultiOutlet {
            total_consumers: Default::default(),
            read_state: Default::default(),
            buffer: self.buffer.clone(),
            current_block: Arc::new(ArcSwap::new(Arc::new(Vec::new()))),
        }
    }*/
}

pub trait Outlet: Clone {
    type Consumer: OutletConsumer + 'static;
    fn get_consumer(self) -> Self::Consumer;
    fn get_sample_rate(&self) -> f32;
}

pub trait OutletConsumer: Send + Sync {
    fn receive(&self) -> Arc<Vec<f32>>;
}

#[derive(Clone)]
pub struct MonoOutlet {
    buffer: Arc<ArrayQueue<f32>>,
    sample_rate: f32,
}

impl Outlet for MonoOutlet {
    type Consumer = MonoOutlet;
    fn get_consumer(self) -> Self::Consumer {
        self
    }
    fn get_sample_rate(&self) -> f32 {
        self.sample_rate
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
}
/*
#[derive(Clone)]
pub struct MonoMultiOutlet {
    total_consumers: Arc<AtomicU32>,
    read_state: Arc<AtomicU32>,
    buffer: Arc<ArrayQueue<f32>>,
    current_block: Arc<ArcSwap<Vec<f32>>>,
}

impl Outlet for MonoMultiOutlet {
    type Consumer = MonoMultiOutletConsumer;
    fn get_consumer(self) -> Self::Consumer {
        let consumers = self.total_consumers.load(Ordering::SeqCst);
        let id = !consumers & (consumers + 1);
        self.total_consumers.store(consumers ^ id, Ordering::SeqCst);
        MonoMultiOutletConsumer {
            id,
            total_consumers: self.total_consumers.clone(),
            read_state: self.read_state.clone(),
            buffer: self.buffer.clone(),
            current_block: self.current_block.clone(),
        }
    }
}

pub struct MonoMultiOutletConsumer {
    id: u32,
    total_consumers: Arc<AtomicU32>,
    read_state: Arc<AtomicU32>,
    buffer: Arc<ArrayQueue<f32>>,
    current_block: Arc<ArcSwap<Vec<f32>>>,
}

impl OutletConsumer for MonoMultiOutletConsumer {
    #[inline]
    fn receive(&self) -> Arc<Vec<f32>> {
        let total_consumers = self.total_consumers.load(Ordering::SeqCst);
        let read_state = self.read_state.load(Ordering::SeqCst);

        let block = if total_consumers == read_state {
            self.read_state.store(0, Ordering::SeqCst);
            let mut new_block = Vec::new();

            while let Some(x) = self.buffer.pop() {
                new_block.push(x);
            }

            let new_block = Arc::new(new_block);

            self.current_block.store(new_block.clone());
            new_block
        } else {
            self.current_block.load().clone()
        };

        self.read_state.fetch_and(self.id, Ordering::SeqCst);

        block
    }
}
*/
