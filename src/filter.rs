use std::{io::Write, f32::consts::{PI, SQRT_2}};

use fundsp::{audionode::AudioNode, prelude::{An, tan}};
use numeric_array::typenum::{*, self};


#[derive(Clone, Copy)]
pub struct BiquadCoefficients {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
}

impl BiquadCoefficients {
    pub fn new(a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) -> Self {
        Self {
            a1,
            a2,
            b0,
            b1,
            b2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BiquadFilter {
    coeffs: BiquadCoefficients,
    // x represents a sample from the input signal, y represents a sample from the output signal
    // where x1 is the previous sample, x2 is the sample before that, and so on.
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    print: bool,
    print_count: u32,
}

impl BiquadFilter {
    fn new(coeffs: BiquadCoefficients) -> Self {
        Self {
            coeffs,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            print: true,
            print_count: 0,
        }
    }

    // one filtering step, taking current sample as input
    fn filter_step(&mut self, x: f32) -> f32 {
        let y = 
              self.coeffs.b0 * x
            + self.coeffs.b1 * self.x1
            + self.coeffs.b2 * self.x2
            - self.coeffs.a1 * self.y1
            - self.coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;

        y
    }
}

impl AudioNode for BiquadFilter {
    const ID: u64 = 9999;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = (f32, f32, f32, f32, f32);

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x0 = input[0] as f32;
        let y0 = 
              self.coeffs.b0 * x0
            + self.coeffs.b1 * self.x1
            + self.coeffs.b2 * self.x2
            - self.coeffs.a1 * self.y1
            - self.coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        
        [y0 as f64].into()
    }
}

pub fn lpf(sr: f32, cutoff: f32) -> An<BiquadFilter> {
    let coeffs = butter_lowpass(sr, cutoff);
    An(BiquadFilter::new(coeffs))
}

pub fn butter_lowpass(sample_rate: f32, cutoff: f32) -> BiquadCoefficients {
    let f = tan(cutoff * PI / sample_rate);
    let a0r = (1.0) / ((1.0) + (SQRT_2) * f + f * f);
    let a1 = ((2.0) * f * f - (2.0)) * a0r;
    let a2 = ((1.0) - (SQRT_2) * f + f * f) * a0r;
    let b0 = f * f * a0r;
    let b1 = (2.0) * b0;
    let b2 = b0;
    println!("a1: {}, a2: {}, b0: {}, b1: {}, b2: {}", a1, a2, b0, b1, b2);
    BiquadCoefficients { a1, a2, b0, b1, b2 }
}

pub fn notch600() -> An<BiquadFilter> {
    let coeffs = BiquadCoefficients::new(
        -1.62155873, 
        0.62835853, 
        0.81417927, 
        -1.62155873, 
        0.81417927);
    An(BiquadFilter::new(coeffs))
}

pub fn lpf550() -> An<BiquadFilter> {
    let coeffs = BiquadCoefficients::new(
        -1.9427603,
        0.99498366,
        0.01305582,
        0.02611163,
        0.01305582);
    An(BiquadFilter::new(coeffs))
}