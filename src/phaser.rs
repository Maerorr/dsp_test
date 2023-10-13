use std::ops::Deref;

use circular_buffer::CircularBuffer;
use fundsp::prelude::{AudioNode, An};
use fundsp::hacker::*;
use numeric_array::typenum::{self, Pow};

use crate::allpass::AllPass;
use crate::filter::{BiquadFilter, second_order_allpass_coefficients, first_order_allpass_coefficients};

const PHASER_DELAYS: [f64; 12] = [
    16.0, 1600.0, 
    33.0, 3300.0, 
    48.0, 4800.0,
    98.0, 9800.0,
    160.0, 16000.0,
    260.0, 20480.0,
    ];

#[derive(Clone)]
pub struct Phaser {
    feedback_buffer: Box<CircularBuffer::<20550, f64>>,
    allpasses: Vec<BiquadFilter>,
    feedback: f64,
    rate: f64,
    sample_rate: f64,
    lfo: f64,
    depth: f64,
    stages: usize,
    offset: f64,
    intensity: f64,
}

impl Phaser {

    pub fn new(sample_rate: f64, feedback: f64, rate: f64, depth: f64, offset: f64, intensity: f64, stages: usize) -> Self {
        let mut buffer1 = CircularBuffer::<20550, f64>::boxed();
        for _ in 0..20550 {
            buffer1.push_back(0.0);
        }

        let mut allpasses = Vec::new();
        for i in 0..6 {
            let allpass = BiquadFilter::new(first_order_allpass_coefficients(sample_rate, PHASER_DELAYS[2 * i] as f64));
            allpasses.push(allpass);
        }

        let feedback = if feedback > 1.0 {
            1.0
        } else if feedback < 0.0 {
            0.0
        } else {
            feedback
        };
        let rate = if rate > 50.0 {
            50.0
        } else if rate < 0.0 {
            0.0
        } else {
            rate
        };

        let depth = depth.clamp(0.0, 1.0);

        let stages = stages.clamp(1, 3);

        let offset = offset.clamp(-1.0, 1.0);

        let intensity = intensity.clamp(0.0, 1.0);

        Self {
            feedback_buffer: buffer1,
            feedback,
            rate: rate,
            sample_rate,
            lfo: 0.0,
            allpasses,
            depth,
            offset: offset,
            intensity: intensity,
            stages,
        }
    }
}

impl AudioNode for Phaser {
    const ID: u64 = 9995;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {
        self.feedback_buffer.clear();
    }

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x = input[0] as f64;
        let mut y = 0.0;

        let mut phased_signal = x + self.feedback * self.feedback_buffer.get(0).unwrap();

        // 1, 2 or 3 noth phaser
        for i in 0..(self.stages * 2) {
            self.allpasses[i].set_coefficients(
                first_order_allpass_coefficients(
                    self.sample_rate, 
                    lerp11(
                        PHASER_DELAYS[2 * i] as f64,
                        PHASER_DELAYS[2 * i + 1] as f64, 
                        (self.lfo.sin() * self.depth + self.offset).clamp(-1.0, 1.0)
                    )));
            phased_signal = self.allpasses[i].process_sample(phased_signal);
        }

        self.lfo += 2.0 * std::f64::consts::PI * self.rate / self.sample_rate;
        if self.lfo > 2.0 * std::f64::consts::PI {
            self.lfo -= 2.0 * std::f64::consts::PI;
        }

        self.feedback_buffer.push_front(phased_signal);

        y = (x + self.intensity * phased_signal);
        [y].into()
    }
}

pub fn my_phaser(sample_rate: f64, feedback: f64, rate: f64, depth: f64, offset: f64, intensity: f64, stages: usize) -> An<Phaser> {
    An(Phaser::new(sample_rate, feedback, rate, depth, offset, intensity, stages))
}