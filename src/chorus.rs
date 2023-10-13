use fundsp::{audionode::AudioNode, prelude::{An, tan}, buffer, feedback};
use numeric_array::typenum::{*, self};
use rand::Rng;
use circular_buffer::CircularBuffer;

use crate::delay::{Delay, self};

const MAX_DELAY: f64 = 100.0;

#[derive(Clone)]
pub struct Chorus {
    rate: f64,
    delay_ms: f64,
    mix: f64,
    sample_rate: f64,
    delay_samples: usize,
    delay_1: Delay,
    delay_2: Delay,
    delay_3: Delay,
    phase1: f64,
    phase2: f64,
    phase3: f64,
    calculated_depth: f64,
    feedback: f64,
    // 3 second buffer at 44.1khz
    buffer: Box<CircularBuffer::<{3*44100}, f64>>,
    count: usize,
}


impl Chorus {
    //time represented in ms. sr is sample rate
    fn new(sample_rate: f64, depth: f64, rate: f64, delay_ms: f64, mix: f64, feedback: f64) -> Chorus {
        // convert delay time from ms to samples
        let delay_samples: usize = ((delay_ms / 1000.0) * sample_rate) as usize;
        
        // delay lines
        let delay_1 = Delay::new(delay_samples, 0.0);
        let delay_2 = Delay::new(delay_samples, 0.0);
        let delay_3 = Delay::new(delay_samples, 0.0);

        // generate3 random phase offsets in (0, 2pi)
        let mut rng = rand::thread_rng();
        let phase1 = 0.0;//rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let phase2 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let phase3 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        
        let mut calculated_depth = depth * sample_rate / 1000.0;
        if calculated_depth > delay_samples as f64 {
            println!("calculated_depth too high. Setting to delay_samples/2");
            calculated_depth = delay_samples as f64 / 2.0;
        }
        println!("base delay: {} samples", delay_samples);
        println!("calculated depth: {} samples", calculated_depth);
        println!("min depth: {} samples", delay_samples as f64 - calculated_depth);
        println!("max depth: {} samples", delay_samples as f64 + calculated_depth);

        let feedback = feedback.clamp(0.0, 0.9999);

        let mut buffer = CircularBuffer::<{3*44100}, f64>::boxed();
        for _ in 0..(3*44100) {
            buffer.push_back(0.0);
        }

        Chorus {
            rate,
            delay_ms,
            mix,
            sample_rate,
            delay_samples,
            delay_1: delay_1,
            delay_2: delay_2,
            delay_3: delay_3,
            phase1: phase1,
            phase2: phase2,
            phase3: phase3,
            calculated_depth,
            feedback,
            buffer,
            count: 0,
        }
    }
}

impl AudioNode for Chorus {
    const ID: u64 = 9998;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {
    }

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {

        let x = input[0] as f64;

        self.phase1 += 2.0 * std::f64::consts::PI * self.rate / self.sample_rate;  
        if self.phase1 > 2.0 * std::f64::consts::PI {
            self.phase1 -= 2.0 * std::f64::consts::PI;
        }
        // repeat for all other phases
        self.phase2 += 2.0 * std::f64::consts::PI * self.rate / self.sample_rate;
        if self.phase2 > 2.0 * std::f64::consts::PI {
            self.phase2 -= 2.0 * std::f64::consts::PI;
        }

        self.phase3 += 2.0 * std::f64::consts::PI * self.rate / self.sample_rate;
        if self.phase3 > 2.0 * std::f64::consts::PI {
            self.phase3 -= 2.0 * std::f64::consts::PI;
        }
        
        let offset1 = ((self.phase1.sin()) * self.calculated_depth / 2.0).round() as i32;
        let offset2 = ((self.phase2.sin()) * self.calculated_depth / 2.0).round() as i32;
        let offset3 = ((self.phase3.sin()) * self.calculated_depth / 2.0).round() as i32;
        //println!("sample: {}, offset1: {}", self.count, offset1);
        
        let new_x = x + self.feedback * self.buffer.get(self.delay_samples).unwrap();
        //println!("sample: {}, delay: {}", self.count, (self.delay_samples as i32 + offset1));
        // mix * (1/3) * (delay1 + delay2 + delay3) + (1 - mix) * x
        let y = 
        self.mix * 1.0/3.0 * (
            self.delay_1.process_sample(new_x, (self.delay_samples as i32 + offset1) as usize)
            + self.delay_2.process_sample(new_x, (self.delay_samples as i32 + offset2) as usize)
            + self.delay_3.process_sample(new_x, (self.delay_samples as i32 + offset3) as usize)
        ) + new_x;

        self.buffer.push_front(y);
        self.count += 1;
        [y].into()
    }
}

/// created a new chorus effect where:
///- depth is in ms
///- rate is in Hz
///- delay is in ms
///- mix is in [0, 1]
///- feedback is in [0, 0.9999]
pub fn my_chorus(sample_rate: f64, depth: f64, rate: f64, delay_ms: f64, mix: f64, feedback: f64) -> An<Chorus> {
    An(Chorus::new(sample_rate, depth, rate, delay_ms, mix, feedback))
}