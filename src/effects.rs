use fundsp::{prelude::{AudioNode, An}, hacker::U1};

use crate::{waveshaper::{my_waveshaper, ShapeType}, filter::{my_second_order_lpf, my_second_order_hpf, my_low_shelf, my_high_shelf}};


pub fn triode_class_a(sample_rate: f64, gain: f64, saturation: f64, low_shelf_gain: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    my_waveshaper(ShapeType::TANH, gain, 0.9, Some(saturation))
    >> my_waveshaper(ShapeType::TANH, gain, 0.9, Some(saturation))
    >> my_waveshaper(ShapeType::TANH, gain, 0.9, Some(saturation))
    >> my_waveshaper(ShapeType::TANH, gain, 0.9, Some
    (saturation)) * -1.0
    >> my_second_order_hpf(sample_rate, 100.0, 1.0)
    >> my_low_shelf(sample_rate, 500.0, low_shelf_gain) * 0.7
}

pub fn class_a_tube_pre(sample_rate: f64, gain: f64, saturation: f64, low_shelf_gain: f64, high_shelf_gain: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    triode_class_a(sample_rate, gain, saturation, 0.0)
    >> triode_class_a(sample_rate, gain, saturation, 0.0)
    >> triode_class_a(sample_rate, gain, saturation, 0.0)
    >> triode_class_a(sample_rate, gain, saturation, 0.0)
    >> my_low_shelf(sample_rate, 500.0, low_shelf_gain)
    >> my_high_shelf(sample_rate, 6000.0, high_shelf_gain)
}