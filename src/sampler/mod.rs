
mod digisiggen;
pub use digisiggen::DigiSigGen;

mod sinegen;
pub use sinegen::SineGen;

mod anasiggen;
pub use anasiggen::AnaSigGen;

use super::{Time, TimeScale};

pub trait Sampler<T> {
    /// Height to display this signals data (in pixels)
    fn get_height(&self) -> f64;

    /// Return signal y scale (peak-to-peak height)
    fn get_yscale(&self) -> f64 { 1. }

    fn get_label(&self) -> String;

    //fn iter_range(&self, range: &[f64; 2]) -> impl Iterator<Item = T>;
    fn iter_range(&self, range: &[f64; 2]) -> Box<dyn Iterator<Item = (T, Time)> + '_>;

    fn get_value_at(&self, t: Time, s: TimeScale) -> T;

    /// Set iteration scale
    fn set_iter_scale(&mut self, range: &[f64; 2], timescale: &TimeScale, scale_width: f64) { }
}

