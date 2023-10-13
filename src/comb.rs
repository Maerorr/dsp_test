use circular_buffer::CircularBuffer;
use fundsp::{audionode::AudioNode, prelude::An};
use numeric_array::typenum::{*, self};
use rand::Rng;

use crate::filter::{BiquadCoefficients, BiquadFilter};

const MAX_INDEX: usize = 41100;

#[derive(Clone, Copy)]
pub enum CombType {
    POSITIVE,
    NEGATIVE,
}

#[derive(Clone)]
pub struct CombFilter {
    buffer: Box<CircularBuffer::<MAX_INDEX, f64>>,
    x_buffer: Box<CircularBuffer::<MAX_INDEX, f64>>,
    buffer_index: usize,
    delay: usize,
    feedback: f64,
    comb_type: CombType,
    use_lpf: bool,
    lpf_g: f64,
}

impl CombFilter {
    pub fn new(delay: usize, feedback: f64, comb_type: CombType, use_lpf: bool, lpf_g: f64) -> Self {
        let mut feedback = if feedback > 1.0 {
            println!("Feedback for Comb Filter too high. Setting to 1.0");
            1.0
        } else if feedback < 0.0 {
            println!("Feedback for Comb Filter too low. Setting to 0.0");
            0.0
        } else {
            feedback
        };

        let mut buffer = CircularBuffer::<MAX_INDEX, f64>::boxed();
        for _ in 0..MAX_INDEX {
            buffer.push_back(0.0);
        }

        let mut x_buf = CircularBuffer::<MAX_INDEX, f64>::boxed();
        if use_lpf {
            for _ in 0..MAX_INDEX {
                x_buf.push_back(0.0);
            }
        }
        
        Self {
            buffer: buffer,
            x_buffer: x_buf,
            buffer_index: 0,
            delay,
            feedback,
            comb_type,
            use_lpf,
            lpf_g,
        }
    }

    pub fn new_comb(delay: usize, feedback: f64, comb_type: CombType) -> Self {
        Self::new(delay, feedback, comb_type, false, 0.0)
    }

    pub fn new_lpf_comb(delay: usize, feedback: f64, lpf_g: f64) -> Self {
        Self::new(delay, feedback, CombType::POSITIVE, true, lpf_g)
    }

    pub fn process_sample(&mut self, x: f64) -> f64 {
        let delayed_sample = self.buffer.get(self.delay).unwrap();
        let mut y = 0.0f64;
        if self.use_lpf {
            y = x 
            + delayed_sample * self.feedback
            // the lpf part
            - self.lpf_g * self.x_buffer.get(self.delay + 1).unwrap() 
            + self.lpf_g * self.buffer.get(1).unwrap();
        } else {
            match self.comb_type {
                CombType::POSITIVE => {
                    y = x + delayed_sample * self.feedback;
                },
                CombType::NEGATIVE => {
                    y = x - delayed_sample * self.feedback;
                }
            }
        }
        
        self.buffer.push_front(y);
        if self.use_lpf {
            self.x_buffer.push_front(x);
        }
        self.buffer_index = (self.buffer_index + 1) % self.delay;
        y
    }
}

impl AudioNode for CombFilter {
    const ID: u64 = 9997;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {
        self.buffer_index = 0;
    }

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let delayed_sample = self.buffer.get(self.delay).unwrap();
        let mut y = 0.0f64;
        match self.comb_type {
            CombType::POSITIVE => {
                y = input[0] + delayed_sample * self.feedback;
            },
            CombType::NEGATIVE => {
                y = input[0] - delayed_sample * self.feedback;
            }
        }
        self.buffer.push_front(y);
        self.buffer_index = (self.buffer_index + 1) % self.delay;
        [y].into()
    }
}

pub fn my_comb(delay: usize, feedback: f64, comb_type: CombType) -> An<CombFilter> {
    An(CombFilter::new(delay, feedback, comb_type, false, 0.0))
}