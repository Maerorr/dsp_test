use std::string;

use fundsp::hacker::*;

use filter::*;
use chorus::*;
use comb::*;
use allpass::*;
mod allpass;
mod filter;
mod chorus;
mod comb;

const INPUT_DIR: &str= "res/input/";
const OUTPUT_DIR: &str = "res/output/";

fn main() {
    let saw_wave = Wave64::load(INPUT_DIR.to_string() + "drum_loop_tail.wav").expect("Could not load wave file.");
    println!("{}", saw_wave.len() as f64 / saw_wave.sample_rate());

    // test: 550Hz LPF, 15.707Q, gain 1
    let coeffs: BiquadCoefficients = BiquadCoefficients::new(
        -1.62155873, 
        0.62835853, 
        0.81417927, 
        -1.62155873, 
        0.81417927);


    // let output = saw_wave.filter(
    //     saw_wave.len() as f64 / saw_wave.sample_rate(), 
    //     &mut (
    //         my_comb(40, 0.8, CombType::POSITIVE) 
    //         >> 
    //         lpf(saw_wave.sample_rate() as f32, 550.0)
    //         >>
    //         notch600()));

    // let output = saw_wave.filter(
    //     saw_wave.len() as f64 / saw_wave.sample_rate(),
    //      &mut (my_chorus(saw_wave.sample_rate(), 5.0, 0.2, 5.0, 0.7)
    //     >>
    //     my_comb(40, 0.6, CombType::NEGATIVE)));

    let output = saw_wave.filter(
        saw_wave.len() as f64 / saw_wave.sample_rate(),
         &mut ( 
                (pass() &
                    my_allpass(2000, 0.7)
                    >> my_allpass(1000, 0.7)
                    >> my_allpass(5000, 0.7)
                    >> my_chorus(saw_wave.sample_rate(), 5.0, 0.2, 3.0, 0.7)
                    >>
                        (
                            my_comb(1687, 0.83, CombType::POSITIVE)
                            &
                            my_comb(1601, 0.71, CombType::POSITIVE)
                            &
                            my_comb(2053, 0.73, CombType::POSITIVE)
                            &
                            my_comb(2251, 0.75, CombType::POSITIVE)
                        )
                        >>
                    my_allpass(500, 0.7)
                    >> my_allpass(113, 0.7)
                    >> my_allpass(1000, 0.7) * 0.4
                ) 
            //((pass() | lfo(|t| (xerp11(110.0, 880.0, spline_noise(0, t * 5.0)), 1.0))) >> bandpass())
            )
        );
    //  ((pass() | lfo(|t| (xerp11(110.0, 880.0, spline_noise(0, t * 5.0)), 1.0))) >> bandpass());
    // let output = saw_wave.filter(saw_wave.len() as f64 / saw_wave.sample_rate(), &mut shape(Shape::Tanh(10.0)));
    //saw_wave.save(OUTPUT_DIR.to_string() + "saw_filtered.wav").expect("Could not save wave file.");
    Wave64::save_wav32(&output, OUTPUT_DIR.to_string() + "saw_filtered.wav").expect("Could not save wave file.");
}
