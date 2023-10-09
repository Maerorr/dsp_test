use fundsp::{audionode::AudioNode, prelude::An};
use numeric_array::typenum::{*, self};
use rand::Rng;

const MAX_INDEX: usize = 64;

#[derive(Clone, Copy)]
pub enum CombType {
    POSITIVE,
    NEGATIVE,
}

#[derive(Clone, Copy)]
pub struct CombFilter {
    buffer: [f64; MAX_INDEX],
    buffer_index: usize,
    delay: usize,
    feedback: f64,
    comb_type: CombType,
}

impl CombFilter {
    pub fn new(delay: usize, feedback: f64, comb_type: CombType) -> Self {
        let feedback = if feedback > 1.0 {
            println!("Feedback for Comb Filter too high. Setting to 1.0");
            1.0
        } else if feedback < 0.0 {
            println!("Feedback for Comb Filter too low. Setting to 0.0");
            0.0
        } else {
            feedback
        };
        Self {
            buffer: [0.0; MAX_INDEX],
            buffer_index: 0,
            delay,
            feedback,
            comb_type,
        }
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
        let delayed_sample = self.buffer[self.buffer_index];
        let mut y = 0.0f64;
        match self.comb_type {
            CombType::POSITIVE => {
                y = input[0] + delayed_sample * self.feedback;
            },
            CombType::NEGATIVE => {
                y = input[0] - delayed_sample * self.feedback;
            }
        }
        self.buffer[self.buffer_index] = y;
        self.buffer_index = (self.buffer_index + 1) % self.delay;
        [y].into()
    }
}

pub fn my_comb(delay: usize, feedback: f64, comb_type: CombType) -> An<CombFilter> {
    An(CombFilter::new(delay, feedback, comb_type))
}