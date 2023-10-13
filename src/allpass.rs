use std::boxed;

use circular_buffer::CircularBuffer;
use fundsp::{audionode::AudioNode, prelude::An};
use numeric_array::typenum::{*, self};

#[derive(Clone)]
pub struct AllPass {
    x_buffer: Box<CircularBuffer::<41100, f64>>,
    y_buffer: Box<CircularBuffer::<41100, f64>>,
    delay: usize,
    gain: f64,
}

impl AllPass {
    pub fn new(delay: usize, gain: f64) -> Self {

        let mut buffer1 = CircularBuffer::<41100, f64>::boxed();
        for _ in 0..41100 {
            buffer1.push_back(0.0);
        }
        let mut buffer2 = CircularBuffer::<41100, f64>::boxed();
        for _ in 0..41100 {
            buffer2.push_back(0.0);
        }

        let gain = if gain > 1.0 {
            1.0
        } else {
            gain
        };

        Self {
            x_buffer: buffer1,
            y_buffer: buffer2,
            delay,
            gain: gain,
        }
    }

    pub fn process_sample(&mut self, x: f64) -> f64 {
        let y = 
        -self.gain * x 
        + self.x_buffer.get(self.delay).unwrap() 
        + self.gain * self.y_buffer.get(self.delay).unwrap();

        self.x_buffer.push_front(x);
        self.y_buffer.push_front(y);

        y
    }
}

impl AudioNode for AllPass {
    const ID: u64 = 9997;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {
        self.x_buffer.clear();
        self.y_buffer.clear();
    }

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x = input[0] as f64;
        
        let y = 
        -self.gain * x 
        + self.x_buffer.get(self.delay).unwrap() 
        + self.gain * self.y_buffer.get(self.delay).unwrap();

        self.x_buffer.push_front(x);
        self.y_buffer.push_front(y);

        [y].into()
    }
}

pub fn my_allpass(delay: usize, gain: f64) -> An<AllPass> {
    An(AllPass::new(delay, gain))
}