use std::{io::Write, f64::consts::{PI, SQRT_2}};

use fundsp::{audionode::AudioNode, prelude::{An, tan}};
use numeric_array::typenum::{*, self};


#[derive(Clone, Copy)]
pub struct BiquadCoefficients {
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    c0: f64,
    d0: f64,
}

impl BiquadCoefficients {
    pub fn new(a0: f64, a1: f64, a2: f64, b0: f64, b1: f64, c0: f64, d0: f64) -> Self {
        Self {
            a0,
            a1,
            a2,
            b0,
            b1,
            c0,
            d0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BiquadFilter {
    coeffs: BiquadCoefficients,
    // x represents a sample from the input signal, y represents a sample from the output signal
    // where x1 is the previous sample, x2 is the sample before that, and so on.
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadFilter {
    pub fn new(coeffs: BiquadCoefficients) -> Self {
        Self {
            coeffs,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    // one filtering step, taking current sample as input
    pub fn process_sample(&mut self, x: f64) -> f64 {
        let y = 
            self.coeffs.a0 * x
            + self.coeffs.a1 * self.x1
            + self.coeffs.a2 * self.x2
            - self.coeffs.b0 * self.y1
            - self.coeffs.b1 * self.y2;
              
        self.x2 = self.x1;
        self.x1 = x;
        
        self.y2 = self.y1;
        self.y1 = y;

        let y = self.coeffs.c0 * y + self.coeffs.d0 * x;

        y
    }

    pub fn set_coefficients(&mut self, coeffs: BiquadCoefficients) {
        self.coeffs = coeffs;
    }

    pub fn get_coefficient(&self, i: usize) -> f64 {
        match i {
            0 => self.coeffs.a0,
            1 => self.coeffs.a1,
            2 => self.coeffs.a2,
            3 => self.coeffs.b0,
            4 => self.coeffs.b1,
            5 => self.coeffs.c0,
            6 => self.coeffs.d0,
            _ => panic!("Invalid coefficient index."),
        }
    }

    // for use of 1st order allpass filters in phaser
    pub fn get_s_value(&self) -> f64 {
        // sum all coeficients from a1 to b2
        self.coeffs.a1 + self.coeffs.a2 + self.coeffs.b0 + self.coeffs.b1
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
        let x0 = input[0] as f64;
        let y0 = self.process_sample(x0);
        
        [y0 as f64].into()
    }
}


pub fn first_order_lpf_coefficients(sample_rate: f64, cutoff: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let y = o.cos() / (1.0 + o.sin());
    let a0 = (1.0 - y) / 2.0;
    let a1 = (1.0 - y) / 2.0;
    let a2 = 0.0;
    let b0 = -y;
    let b1 = 0.0;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn first_order_hpf_coefficients(sample_rate: f64, cutoff: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let y = o.cos() / (1.0 + o.sin());
    let a0 = (1.0 + y) / 2.0;
    let a1 = -((1.0 + y) / 2.0);
    let a2 = 0.0;
    let b0 = -y;
    let b1 = 0.0;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn second_order_lpf_coefficients(sample_rate: f64, cutoff: f64, q: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let d = 1.0 / q;
    let b = 
    0.5 
    * ((1.0 - (d / 2.0) * o.sin()) 
    / (1.0 + (d / 2.0) * o.sin()));
    let y = (0.5 + b) * o.cos();
    let a0 = (0.5 + b - y) / 2.0;
    let a1 = 0.5 + b - y;
    let a2 = (0.5 + b - y) / 2.0;
    let b0 = -2.0 * y;
    let b1 = 2.0 * b;
    println!("a0: {}\na1: {}\na2: {}\nb0: {}\nb1: {}", a0, a1, a2, b0, b1);
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn second_order_hpf_coefficients(sample_rate: f64, cutoff: f64, q: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let d = 1.0 / q;
    let b = 
    0.5 
    * ((1.0 - (d / 2.0) * o.sin()) 
    / (1.0 + (d / 2.0) * o.sin()));
    let y = (0.5 + b) * o.cos();
    let a0 = (0.5 + b + y) / 2.0;
    let a1 = -(0.5 + b + y);
    let a2 = (0.5 + b + y) / 2.0;
    let b0 = -2.0 * y;
    let b1 = 2.0 * b;
    println!("a0: {}\na1: {}\na2: {}\nb0: {}\nb1: {}", a0, a1, a2, b0, b1);
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn band_pass_coefficients(sample_rate: f64, cutoff: f64, q: f64) -> BiquadCoefficients {
    let k = (PI * cutoff / sample_rate).tan();
    let d = k * k * q + k + q;
    let a0 = k / d;
    let a1 = 0.0;
    let a2 = -k / d;
    let b0 = 2.0 * q * (k * k - 1.0) / d;
    let b1 = (k * k * q - k + q) / d;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn notch_coefficients(sample_rate: f64, cutoff: f64, q: f64) -> BiquadCoefficients {
    let k = (PI * cutoff / sample_rate).tan();
    let d = k * k * q + k + q;
    let a0 = (q * (k * k + 1.0)) / d;
    let a1 = (2.0 * q * (k * k - 1.0)) / d;
    let a2 = (q * (k * k + 1.0)) / d;
    let b0 = (2.0 * q * (k * k - 1.0)) / d;
    let b1 = (k * k * q - k + q) / d;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn first_order_allpass_coefficients(sample_rate: f64, cutoff: f64) -> BiquadCoefficients {
    let alpha = 
    (tan(PI * cutoff / sample_rate) - 1.0) 
    / (tan(PI * cutoff / sample_rate) + 1.0);
    let a0 = alpha;
    let a1 = 1.0;
    let a2 = 0.0;
    let b0 = alpha;
    let b1 = 0.0;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn second_order_allpass_coefficients(sample_rate: f64, cutoff: f64, q: f64) -> BiquadCoefficients {
    let bw = cutoff / q;
    let alpha = 
    (tan(PI * bw / sample_rate) - 1.0) 
    / (tan(PI * bw / sample_rate) + 1.0);
    let b = -(2.0 * PI * cutoff / sample_rate).cos();
    let a0 = -alpha;
    let a1 = b * (1.0 - alpha);
    let a2 = 1.0;
    let b0 = b * (1.0 - alpha);
    let b1 = -alpha;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
}

pub fn low_shelf_coefficients(sample_rate: f64, cutoff: f64, gain: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let u = 10.0_f64.powf(gain / 20.0);
    let b = 4.0 / (1.0 + u);
    let d = b * (o / 2.0).tan();
    let y = (1.0 - d) / (1.0 + d);

    let a0 = (1.0 - y) / 2.0;
    let a1 = (1.0 - y) / 2.0;
    let a2 = 0.0;
    let b0 = -y;
    let b1 = 0.0;
    let c0 = u - 1.0;
    let d0 = 1.0;
    println!("a0: {}\na1: {}\na2: {}\nb0: {}\nb1: {}", a0, a1, a2, b0, b1);
    BiquadCoefficients::new(a0, a1, a2, b0, b1, c0, d0)
}

pub fn high_shelf_coefficients(sample_rate: f64, cutoff: f64, gain: f64) -> BiquadCoefficients {
    let o = 2.0 * PI * cutoff / sample_rate;
    let u = 10.0_f64.powf(gain / 20.0);
    let b = (1.0 + u) / 4.0;
    let d = b * tan(o / 2.0);
    let y = (1.0 - d) / (1.0 + d);
    let a0 = (1.0 + y) / 2.0;
    let a1 = -(1.0 + y) / 2.0;
    let a2 = 0.0;
    let b0 = -y; 
    let b1 = 0.0; 
    let c0 = u - 1.0;
    let d0 = 1.0;
    BiquadCoefficients::new(a0, a1, a2, b0, b1, c0, d0)
}

pub fn peak_coefficients(sample_rate: f64, cutoff: f64,  q: f64, gain: f64) -> BiquadCoefficients {
    let k = (PI * cutoff / sample_rate).tan();
    let v = 10.0_f64.powf(gain / 20.0);
    let d0 = 1.0 + (1.0 / q) * k + k*k;
    let e = 1.0 + (1.0 / (q * v)) * k + k*k;
    let alpha = 1.0 + (v/q) * k + k*k;
    let beta = 2.0 * (k*k - 1.0);
    let y = 1.0 - (v/q) * k + k*k;
    let d = 1.0 - (1.0 / q) * k + k*k;
    let p = 1.0 - (1.0/(q*v)) * k + k*k;

    if gain >= 0.0 {
        let a0 = alpha / d0;
        let a1 = beta / d0;
        let a2 = y / d0;
        let b0 = beta / d0;
        let b1 = d / d0;
        return BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
    } else {
        let a0 = d0 / e;
        let a1 = beta / e;
        let a2 = d / e;
        let b0 = beta / e;
        let b1 = p / e;
        return BiquadCoefficients::new(a0, a1, a2, b0, b1, 1.0, 0.0)
    }
}

#[allow(dead_code)]
pub fn my_first_order_lpf(sample_rate: f64, cutoff: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(first_order_lpf_coefficients(sample_rate, cutoff)))
}

#[allow(dead_code)]
pub fn my_first_order_hpf(sample_rate: f64, cutoff: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(first_order_hpf_coefficients(sample_rate, cutoff)))
}

#[allow(dead_code)]
pub fn my_second_order_lpf(sample_rate: f64, cutoff: f64, q: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(second_order_lpf_coefficients(sample_rate, cutoff, q)))
}

#[allow(dead_code)]
pub fn my_second_order_hpf(sample_rate: f64, cutoff: f64, q: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(second_order_hpf_coefficients(sample_rate, cutoff, q)))
}

#[allow(dead_code)]
pub fn my_band_pass(sample_rate: f64, cutoff: f64, q: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(band_pass_coefficients(sample_rate, cutoff, q)))
}

#[allow(dead_code)]
pub fn my_notch(sample_rate: f64, cutoff: f64, q: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(notch_coefficients(sample_rate, cutoff, q)))
}

#[allow(dead_code)]
pub fn my_first_order_allpass(sample_rate: f64, cutoff: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(first_order_allpass_coefficients(sample_rate, cutoff)))
}

#[allow(dead_code)]
pub fn my_second_order_allpass(sample_rate: f64, cutoff: f64, q: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(second_order_allpass_coefficients(sample_rate, cutoff, q)))
}

#[allow(dead_code)]
pub fn my_low_shelf(sample_rate: f64, cutoff: f64, gain: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(low_shelf_coefficients(sample_rate, cutoff, gain)))
}

#[allow(dead_code)]
pub fn my_high_shelf(sample_rate: f64, cutoff: f64, gain: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(high_shelf_coefficients(sample_rate, cutoff, gain)))
}

#[allow(dead_code)]
pub fn my_peak(sample_rate: f64, cutoff: f64,  q: f64, gain: f64) -> An<BiquadFilter> {
    An(BiquadFilter::new(peak_coefficients(sample_rate, cutoff, q, gain)))
}