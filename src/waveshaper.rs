use std::f64::consts::E;

use circular_buffer::CircularBuffer;
use fundsp::{audionode::AudioNode, prelude::An, shape::Shape};
use numeric_array::typenum::{*, self};

#[derive(Clone)]
pub enum ShapeType {
    ARRY,
    SIG, // USES SATURATION
    SIG2,
    TANH,// USES SATURATION
    ATAN,// USES SATURATION
    FEXP1,// USES SATURATION
    FEXP2,
    EXP,
    ATSR,
    SQS,
    CUBE,
    HCLIP,
    HWR,
    FWR,
    ASQRT,
}

// implement all of these as functions
pub fn sgn(x: f64) -> f64 {
    if x >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

pub fn arry(x: f64) -> f64 {
    (3.0 * x / 2.0) 
    * (1.0 - x*x/3.0)
}

pub fn sig(x: f64, saturation: f64) -> f64 {
    2.0 / (1.0 + (-saturation * x).exp()) - 1.0
}

pub fn sig2(x: f64) -> f64 {
    ((x.exp() - 1.0) * (E + 1.0))
    / 
    ((x.exp() + 1.0) * (E - 1.0))
}

pub fn tanh(x: f64, saturation: f64) -> f64 {
    (saturation * x).tanh() / saturation.tanh()
}

pub fn atan(x: f64, saturation: f64) -> f64 {
    (saturation * x).atan() / (saturation).atan()
}

pub fn fexp1(x: f64, saturation: f64) -> f64 {
    sgn(x) * (
        (1.0 - (-(saturation * x).abs()).exp())
        /
        (1.0 - (-saturation).exp())
    )
}

pub fn fexp2(x: f64) -> f64 {
    sgn(x) * (
        (1.0 - (x.abs()).exp())
        /
        (E - 1.0)
    )
}

pub fn exp(x: f64) -> f64 {
    (E - (1.0 - x).exp()) / (E - 1.0)
}

pub fn atsr(x: f64) -> f64 {
    2.5 * (0.9*x).atan() + 2.5 * (1.0 - (0.9*x)*(0.9*x)).sqrt() -2.5
}

pub fn sqs(x: f64) -> f64 {
    x*x*sgn(x)
}

pub fn cube(x: f64) -> f64 {
    x*x*x
}

pub fn hclip(x: f64) -> f64 {
    if x.abs() > 0.5 {
        0.5 * sgn(x)
    } else {
        x
    }
}

pub fn hwr(x: f64) -> f64 {
    0.5*(x + x.abs())
}

pub fn fwr(x: f64) -> f64 {
    x.abs()
}

pub fn asqrt(x: f64) -> f64 {
    sgn(x) * x.abs().sqrt()
}

#[derive(Clone)]
pub struct Waveshaper {
    shape_type: ShapeType,
    saturation: f64,
    pre_gain: f64,
    post_gain: f64,
}

impl Waveshaper {
    pub fn new(shape_type: ShapeType, saturation: f64, pre_gain: f64, post_gain: f64) -> Self {
        let saturation = if saturation < 0.0 {
            0.0
        } else {
            saturation
        };

        let pre_gain = pre_gain.max(0.0);
        let post_gain = post_gain.max(0.0);

        Self {
            shape_type,
            saturation,
            pre_gain,
            post_gain,
        }
    }

    pub fn process_sample(&mut self, x: f64) -> f64 {
        // initial gain and had clipping at |x|=1.0

        let x = x * self.pre_gain;

        let y = match self.shape_type {
            ShapeType::ARRY => arry(x),
            ShapeType::SIG => sig(x, self.saturation),
            ShapeType::SIG2 => sig2(x),
            ShapeType::TANH => tanh(x, self.saturation),
            ShapeType::ATAN => atan(x, self.saturation),
            ShapeType::FEXP1 => fexp1(x, self.saturation),
            ShapeType::FEXP2 => fexp2(x),
            ShapeType::EXP => exp(x),
            ShapeType::ATSR => atsr(x),
            ShapeType::SQS => sqs(x),
            ShapeType::CUBE => cube(x),
            ShapeType::HCLIP => hclip(x),
            ShapeType::HWR => hwr(x),
            ShapeType::FWR => fwr(x),
            ShapeType::ASQRT => asqrt(x),
        };

        y * self.post_gain
    }
}

impl AudioNode for Waveshaper {
    const ID: u64 = 9992;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {}

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x = input[0] as f64;
        let y = self.process_sample(x);

        [y].into()
    }
}

pub fn my_waveshaper(shape_type: ShapeType, pre_gain: f64, post_gain: f64, saturation: Option<f64> ) -> An<Waveshaper> {
    if saturation.is_some() {
        return An(Waveshaper::new(shape_type, saturation.unwrap(), pre_gain, post_gain));
    }
    An(Waveshaper::new(shape_type, 1.0, pre_gain, post_gain))
}

#[derive(Clone)]
pub struct AssymetricWaveshaper {
    up_shaper: Waveshaper,
    down_shaper: Waveshaper,
    saturation: f64,
    pre_gain: f64,
    post_gain: f64,
}

impl AssymetricWaveshaper {
    pub fn new(up_shape: ShapeType, down_shape: ShapeType, saturation: f64, pre_gain: f64, post_gain: f64) -> Self {
        let saturation = if saturation < 0.0 {
            0.0
        } else {
            saturation
        };

        let pre_gain = pre_gain.max(0.0);
        let post_gain = post_gain.max(0.0);

        Self {
            up_shaper: Waveshaper::new(up_shape, saturation, pre_gain, post_gain),
            down_shaper: Waveshaper::new(down_shape, saturation, pre_gain, post_gain),
            saturation,
            pre_gain,
            post_gain,
        }
    }

    pub fn process_sample(&mut self, x: f64) -> f64 {
        // initial gain and had clipping at |x|=1.0

        let x = x * self.pre_gain;

        self.up_shaper.pre_gain = self.pre_gain;
        self.up_shaper.post_gain = self.post_gain;
        self.up_shaper.saturation = self.saturation;
        self.down_shaper.pre_gain = self.pre_gain; 
        self.down_shaper.post_gain = self.post_gain;
        self.down_shaper.saturation = self.saturation;

        let y = if x >= 0.0 {
            self.up_shaper.process_sample(x)
        } else {
            self.down_shaper.process_sample(x)
        };

        y * self.post_gain
    }
}

impl AudioNode for AssymetricWaveshaper {
    const ID: u64 = 9992;
    type Sample = f64;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = f64;

    fn reset(&mut self) {}

    fn tick(
            &mut self,
            input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
        ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x = input[0] as f64;
        let y = self.process_sample(x);

        [y].into()
    }
}

pub fn my_assymetric_waveshaper(up_shape: ShapeType, down_shape: ShapeType, pre_gain: f64, post_gain: f64, saturation: Option<f64> ) -> An<AssymetricWaveshaper> {
    if saturation.is_some() {
        return An(AssymetricWaveshaper::new(up_shape, down_shape, saturation.unwrap(), pre_gain, post_gain));
    }
    An(AssymetricWaveshaper::new(up_shape, down_shape, 1.0, pre_gain, post_gain))
}