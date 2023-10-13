use circular_buffer::CircularBuffer;
use fundsp::prelude::{AudioNode, An};
use numeric_array::typenum;

const MAX_DELAY: usize = 3 * 44100; // 3 seconds at 44100Hz

#[derive(Clone)]
pub struct Delay {
    x_buffer: Box<CircularBuffer::<MAX_DELAY, f64>>,
    y_buffer: Box<CircularBuffer::<MAX_DELAY, f64>>,
    delay: usize,
    feedback: f64,
}

impl Delay {
    pub fn new(delay: usize, feedback: f64) -> Self {

        let mut buffer1 = CircularBuffer::<MAX_DELAY, f64>::boxed();
        for _ in 0..MAX_DELAY {
            buffer1.push_back(0.0);
        }

        let mut buffer2 = CircularBuffer::<MAX_DELAY, f64>::boxed();
        for _ in 0..MAX_DELAY {
            buffer2.push_back(0.0);
        }


        let feedback = if feedback > 1.0 {
            1.0
        } else if feedback < 0.0 {
            0.0
        } else {
            feedback
        };

        Self {
            x_buffer: buffer1,
            y_buffer: buffer2,
            delay,
            feedback: feedback,
        }
    }

    // y(n) = x(n - delay) + fb * y(n - delay)
    pub fn process_sample(&mut self, x: f64, delay: usize) -> f64 {
        let y = 
        self.x_buffer.get(delay).unwrap() 
        + self.feedback * self.y_buffer.get(delay).unwrap();

        self.x_buffer.push_front(x);
        self.y_buffer.push_front(y);

        y
    }
}

impl AudioNode for Delay {
    const ID: u64 = 9993;
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
        
        let y = self.process_sample(input[0] as f64, self.delay);

        [y].into()
    }
}

pub fn my_delay(sr: f64, delay_ms: f64, feedback: f64) -> An<Delay> {
    let delay_samples = ((delay_ms as f64 / 1000.0) * sr) as usize;
    println!("delay samples: {}", delay_samples);
    An(Delay::new(delay_samples, feedback))
}