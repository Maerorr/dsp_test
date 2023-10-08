use fundsp::{audionode::AudioNode, prelude::{An, tan}, buffer};
use numeric_array::typenum::{*, self};
use rand::Rng;
use circular_buffer::CircularBuffer;

const MAX_DELAY: f64 = 100.0;

#[derive(Clone)]
pub struct Chorus {
    depth: f64,
    rate: f64,
    delay_ms: f64,
    mix: f64,
    sample_rate: f64,
    delay_samples: usize,
    delay_buffer1: Box<CircularBuffer::<10000, f64>>,
    delay_buffer2: Box<CircularBuffer::<10000, f64>>,
    delay_buffer3: Box<CircularBuffer::<10000, f64>>,
    phase1: f64,
    phase2: f64,
    phase3: f64,
    calculated_depth: f64,
}


impl Chorus {
    //time represented in ms. sr is sample rate
    fn new(sample_rate: f64, depth: f64, rate: f64, delay_ms: f64, mix: f64) -> Chorus {
        // convert delay time from ms to samples
        let delay_samples = (delay_ms * sample_rate / 1000.0) as usize;
        println!("delay_samples: {}", delay_samples);
        // create a new circular buffer and fill it with 0's
        let mut delay_buffer1 = Box::new(CircularBuffer::<10000, f64>::new());
        for _ in 0..10000 {
            delay_buffer1.push_back(0.0);
        }
        let mut delay_buffer2 = Box::new(CircularBuffer::<10000, f64>::new());
        for _ in 0..10000 {
            delay_buffer2.push_back(0.0);
        }
        let mut delay_buffer3 = Box::new(CircularBuffer::<10000, f64>::new());
        for _ in 0..10000 {
            delay_buffer3.push_back(0.0);
        }
        // generate3 random phase offsets in (0, 2pi)
        let mut rng = rand::thread_rng();
        let phase1 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let phase2 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let phase3 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        
        let calculated_depth = depth * sample_rate / 1000.0;
        Chorus {
            depth,
            rate,
            delay_ms,
            mix,
            sample_rate,
            delay_samples,
            delay_buffer1: delay_buffer1,
            delay_buffer2: delay_buffer2,
            delay_buffer3: delay_buffer3,
            phase1: phase1,
            phase2: phase2,
            phase3: phase3,
            calculated_depth,
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
        
        let offset1 = ((self.phase1.sin() + 1.0) * self.calculated_depth).round() as usize;
        let offset2 = ((self.phase2.sin() + 1.0) * self.calculated_depth).round() as usize;
        let offset3 = ((self.phase3.sin() + 1.0) * self.calculated_depth).round() as usize;
        
        // mix * (1/3) * (delay1 + delay2 + delay3) + (1 - mix) * x
        let y = 
        self.mix * 1.0/3.0 * (
            self.delay_buffer1.get(offset1).unwrap() + 
            self.delay_buffer2.get(offset2).unwrap() +
            self.delay_buffer3.get(offset3).unwrap()
        ) + (1.0 - self.mix) * x;

        self.delay_buffer1.push_front(x);
        self.delay_buffer2.push_front(x);
        self.delay_buffer3.push_front(x);

        [y].into()
    }
}

pub fn my_chorus(sample_rate: f64, depth: f64, rate: f64, delay_ms: f64, mix: f64) -> An<Chorus> {
    An(Chorus::new(sample_rate, depth, rate, delay_ms, mix))
}