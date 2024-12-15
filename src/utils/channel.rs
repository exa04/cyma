// TODO: Document stuff
// TODO: Provide a builder or something for the Inlet
// TODO: Multi-Outlet - 1 input, multiple outputs
// TODO: Stereo-In/Outlets
// TODO: Settle on a fitting skeumorphism ("outlet consumer" sounds kinda weird - might just be me)

use blinkcast::alloc::{Receiver, Sender};
use nih_plug::buffer::Buffer;
use nih_plug::prelude::AtomicF32;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};

#[derive(Clone)]
pub struct MonoChannel {
    sender: Sender<f32>,
    sample_rate: Arc<AtomicF32>,
}

impl MonoChannel {
    pub fn new(size: usize) -> MonoChannel {
        Self {
            sender: Sender::<f32>::new(size),
            sample_rate: Default::default(),
        }
    }
}

impl Default for MonoChannel {
    fn default() -> MonoChannel {
        MonoChannel {
            sender: Sender::<f32>::new(4096),
            sample_rate: Default::default(),
        }
    }
}

impl MonoChannel {
    #[inline]
    pub fn enqueue_buffer_summing(&mut self, buffer: &mut Buffer) {
        let channels = buffer.channels() as f32;
        for mut x in buffer.iter_samples() {
            self.try_send(x.iter_mut().map(|x| *x).sum::<f32>() / channels);
        }
    }

    #[inline]
    pub fn try_send(&mut self, value: f32) {
        self.sender.send(value);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
    }

    pub fn get_consumer(self) -> MonoChannelConsumer {
        MonoChannelConsumer {
            receiver: self.sender.new_receiver(),
            sample_rate: self.sample_rate.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MonoChannelConsumer {
    receiver: Receiver<f32>,
    sample_rate: Arc<AtomicF32>,
}

impl MonoChannelConsumer {
    #[inline]
    pub fn receive(&mut self) -> Vec<f32> {
        let mut new_block = Vec::new();
        while let Some(x) = self.receiver.recv() {
            new_block.push(x);
        }
        new_block
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut channel: MonoChannel = Default::default();
        channel.try_send(10.0);
        channel.try_send(20.0);

        let mut consumer = channel.get_consumer();

        consumer.receive();
    }
}
