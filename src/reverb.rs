use circular_buffer::CircularBuffer;
use fundsp::{audionode::AudioNode, prelude::An, Num};
use numeric_array::typenum::{*, self};
use rand::Rng;

use crate::{comb::{CombFilter, CombType}, allpass::AllPass, delay};

#[derive(Clone)]
pub enum ReverbType {
    CombReverb,
    Schroeder,
    LpfComb,
    Moorer,
}

#[derive(Clone)]
pub struct Reverb {
    combs: Vec<CombFilter>,
    allpasses: Vec<AllPass>,
    decay: f64,
    reverb_type: ReverbType,
}

impl Reverb {
    pub fn new(sample_rate: f64, decay: f64, reverb_type: ReverbType, damp: f64) -> Self {
        let mut combs = Vec::new();
        let mut allpasses = Vec::new();

        let mut rng = rand::thread_rng();

        match reverb_type {
            ReverbType::CombReverb => {
                let delays_ms = [21.0, 26.0, 31.0, 37.0];
                for i in 0..4 {
                    // random delay between 3 and 50 ms
                    let delay_ms = delays_ms[i];
                    let delay_seconds = delay_ms / 1000.0;
                    let delay_samples = (delay_seconds * sample_rate).floor() as usize;

                    let power = -(3.0 * delay_seconds as f64) / (decay) ;

                    let g = 10f64.powf(power);

                    //println!("\n## COMB {} ## \ndelay_ms: {}, \ndelay_seconds: {}, \ndelay_samples: {},\npower: {:.2}\ng: {:.5}", i, delay_ms, delay_seconds, delay_samples, power, g);
                    let comb_type = CombType::POSITIVE;
                    combs.push(CombFilter::new_comb(delay_samples as usize, g, comb_type));
                }
            },
            ReverbType::Schroeder => {
                let mut delay_ms = 15.0;
                for i in 0..4 {
                    // random delay between 3 and 50 ms
                    delay_ms = delay_ms * 1.5;
                    let delay_seconds = delay_ms / 1000.0;
                    let delay_samples = (delay_seconds * sample_rate).floor() as usize;
                    let decay_samples = (decay * sample_rate).floor() as usize;

                    let power = -(3.0 * delay_samples as f64) / (decay_samples as f64) ;

                    let g = 10f64.powf(power);

                    //println!("\n## COMB {} ## \ndelay_ms: {}, \ndelay_seconds: {}, \ndelay_samples: {},\npower: {:.2}\ng: {:.5}", i, delay_ms, delay_seconds, delay_samples, power, g);
                    let comb_type = CombType::POSITIVE;
                    combs.push(CombFilter::new_comb(delay_samples as usize, g, comb_type));
                }

                for _ in 0..4 {
                    // random delay between 1 and 5 ms
                    let delay = ((rng.gen_range(2.0..8.0) / 1000.0) * sample_rate).floor() as usize;
                    allpasses.push(AllPass::new(delay, 0.707));
                }
            },
            ReverbType::LpfComb => {
                let mut delay_ms = 15.0;
                for i in 0..6 {
                    // random delay between 3 and 50 ms
                    delay_ms = delay_ms * 1.5;
                    let delay_seconds = delay_ms / 1000.0;
                    let delay_samples = (delay_seconds * sample_rate).floor() as usize;
                    let decay_samples = (decay * sample_rate).floor() as usize;

                    let power = -(3.0 * delay_samples as f64) / (decay_samples as f64) ;

                    let g = 10f64.powf(power);
                    
                    let damp = damp.clamp(0.0, 0.9999);

                    let new_g = g * (1.0 - damp);

                    println!("\n## COMB {} ## \ndelay_ms: {}, \ndelay_seconds: {}, \ndelay_samples: {},\npower: {:.2}\ng: {:.5}", i, delay_ms, delay_seconds, delay_samples, power, new_g);
                    combs.push(CombFilter::new_lpf_comb(delay_samples as usize, new_g, damp));
                }
            },
            ReverbType::Moorer => {
                let mut delay_ms = 15.0;
                for i in 0..6 {
                    // random delay between 3 and 50 ms
                    delay_ms = delay_ms * 1.5;
                    let delay_seconds = delay_ms / 1000.0;
                    let delay_samples = (delay_seconds * sample_rate).floor() as usize;
                    let decay_samples = (decay * sample_rate).floor() as usize;

                    let power = -(3.0 * delay_samples as f64) / (decay_samples as f64) ;

                    let g = 10f64.powf(power);
                    
                    let damp = damp.clamp(0.0, 0.9999);

                    let new_g = g * (1.0 - damp);

                    println!("\n## COMB {} ## \ndelay_ms: {}, \ndelay_seconds: {}, \ndelay_samples: {},\npower: {:.2}\ng: {:.5}", i, delay_ms, delay_seconds, delay_samples, power, new_g);
                    combs.push(CombFilter::new_lpf_comb(delay_samples as usize, new_g, damp));
                }

                for _ in 0..1 {
                    let delay = ((rng.gen_range(2.0..8.0) / 1000.0) * sample_rate).floor() as usize;
                    allpasses.push(AllPass::new(delay, 0.707));
                }
            },
        }

        Self {
            combs,
            allpasses,
            decay,
            reverb_type,
        }
    }

    pub fn new_comb_reverb(sample_rate: f64, decay: f64) -> Self {
        Self::new(sample_rate, decay, ReverbType::CombReverb, 0.0)
    }

    pub fn new_schroeder_reverb(sample_rate: f64, decay: f64) -> Self {
        Self::new(sample_rate, decay, ReverbType::Schroeder, 0.0)
    }

    pub fn new_lpf_comb(sample_rate: f64, decay: f64, damp: f64) -> Self {
        Self::new(sample_rate, decay, ReverbType::LpfComb, damp)
    }

    pub fn new_moorer_reverb(sample_rate: f64, decay: f64, damp: f64) -> Self {
        Self::new(sample_rate, decay, ReverbType::Moorer, damp)
    }

    pub fn process_sample(&mut self, x: f64) -> f64 {
        let mut y = 0.0;
        match self.reverb_type {
            ReverbType::CombReverb => {
                for (i, comb) in self.combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_sample(x);
                    } else {
                        y -= comb.process_sample(x);
                    }
                }
                y *= 0.25;
            },
            ReverbType::Schroeder => {
                let mut after_allpasses: f64;
                after_allpasses = self.allpasses[0].process_sample(x);
                after_allpasses = self.allpasses[1].process_sample(after_allpasses);

                for (i, comb) in self.combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_sample(x);
                    } else {
                        y -= comb.process_sample(x);
                    }
                }

                y = self.allpasses[2].process_sample(y);
                y = self.allpasses[3].process_sample(y);
            },
            ReverbType::LpfComb => {
                for (i, comb) in self.combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_sample(x);
                    } else {
                        y -= comb.process_sample(x);
                    }
                }
                y *= 0.8;
            },
            ReverbType::Moorer => {
                for (i, comb) in self.combs.iter_mut().enumerate() {
                    if i % 2 == 0 {
                        y += comb.process_sample(x);
                    } else {
                        y -= comb.process_sample(x);
                    }
                }

                y *= 0.75;

                y = self.allpasses[0].process_sample(y);
            },
        }
        y
    }
}

impl AudioNode for Reverb {
    const ID: u64 = 9992;
    type Sample = f64;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = f64;

    fn reset(&mut self) {
    }

    fn tick(
        &mut self,
        input: &fundsp::prelude::Frame<Self::Sample, Self::Inputs>,
    ) -> fundsp::prelude::Frame<Self::Sample, Self::Outputs> {
        let x = input[0] as f64;
        let y = self.process_sample(x);
        [y].into()
    }
}

pub fn comb_reverb(sample_rate: f64, decay: f64) -> An<Reverb> {
    An(Reverb::new_comb_reverb(sample_rate, decay))
}

pub fn schroeder_reverb(sample_rate: f64, decay: f64) -> An<Reverb> {
    An(Reverb::new_schroeder_reverb(sample_rate, decay))
}

pub fn lpf_comb_reverb(sample_rate: f64, decay: f64, damp: f64) -> An<Reverb> {
    An(Reverb::new_lpf_comb(sample_rate, decay, damp))
}

pub fn moorer_reverb(sample_rate: f64, decay: f64, damp: f64) -> An<Reverb> {
    An(Reverb::new_moorer_reverb(sample_rate, decay, damp))
}