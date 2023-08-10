use std::marker::PhantomData;
use crate::{Sampler, Time, /*TimeUnit,*/ TimeScale};

#[derive(Debug)]
enum SigGenType<T> {
    //Boolean,
    //Numeric,
    // Generated signal types
    Fixed(T /*value*/),
    Clock(f64 /*period*/),
    Pulse(f64 /*start*/, f64/*end*/, f64 /*repeat (after start)*/)
}

#[derive(Debug)]
pub struct DigiSigGen {
    sig: usize,
    stype: SigGenType<bool>,
}

impl DigiSigGen {
    pub fn new_fixed(idx: usize, val: bool) -> Self {
        DigiSigGen {
            sig: idx,
            stype: SigGenType::Fixed(val),
        }
    }

    pub fn new_clock(idx: usize, period: f64) -> Self {
        DigiSigGen {
            sig: idx,
            stype: SigGenType::Clock(period),
        }
    }

    pub fn new_pulse(idx: usize, start: f64, end: f64, repeat: f64) -> Self {
        DigiSigGen {
            sig: idx,
            stype: SigGenType::Pulse(start, end, repeat),
        }
    }
}

impl Sampler<bool> for DigiSigGen {
    fn get_height(&self) -> f64 { crate::HEIGHT_DIGITAL }

    fn get_label(&self) -> String {
        format!("signal_{}", self.sig)
    }

    fn iter_range(&self, range: &[f64; 2]) -> Box<dyn Iterator<Item = (bool, Time)> + '_> {
        Box::new(DigiSigIter {
            smpl: self,
            pos: range[0]-f64::EPSILON,
            range: *range,
            phantom: PhantomData,
        })
    }

    fn get_value_at(&self, t: Time, _s: TimeScale) -> bool {
        match self.stype {
            SigGenType::Fixed(val) => val,
            SigGenType::Clock(period) => t % period < period / 2.,
            SigGenType::Pulse(start, end, repeat) => {
                if t < start { false }
                else { (t - start) % repeat < (end - start) }
            },
        }
    }
}

pub struct DigiSigIter<'r, T> {
    smpl: &'r DigiSigGen,
    pos: Time,
    range: [Time; 2],
    phantom: PhantomData<T>,
}

impl DigiSigIter<'_, bool> {
    fn next_clock(&mut self, period: f64) -> Option<(bool, Time)> {
        let half_period = period / 2.;
        let next_pos = self.pos + (half_period - self.pos % half_period);
        if next_pos <= self.range[1] {
            let epsilon = 16. * f64::EPSILON;
            self.pos = next_pos;
            let next_val = next_pos % period < epsilon; // Rising edge on full period
            Some((next_val, next_pos))
        } else {
            None
        }
    }

    fn next_pulse(&mut self, start: f64, end: f64, repeat: f64) -> Option<(bool, Time)> {
        if self.pos < start {
            self.pos = start;
            Some((true, start))
        } else {
            let now = self.pos;
            let base = ((now - start) / repeat).floor() * repeat;
            let offs = (now - start) % repeat;
            let (next_val, next_pos) = if offs < (end - start) {
                (false, start + base + (end - start))
            } else {
                (true, start + base + repeat)
            };
            if next_pos < self.range[1] {
                //println!("next_pulse: now {}, base {}, offs {},  {},{}", 
                //         now, base, offs, next_val, next_pos);
                self.pos = next_pos;
                Some((next_val, next_pos))
            } else { None }
        }
    }

}

impl Iterator for DigiSigIter<'_, bool> {
    type Item = (bool, Time);

    fn next(&mut self) -> Option<Self::Item> {
        match self.smpl.stype {
            SigGenType::Fixed(val) => {
                if self.pos < self.range[0] {
                    self.pos = self.range[0];
                    Some((val, self.range[0]))
                } else if self.pos < self.range[1] {
                    self.pos = self.range[1];
                    Some((val, self.range[1]))
                } else {
                    None
                }
            },
            SigGenType::Clock(period) => self.next_clock(period),
            SigGenType::Pulse(start, end, repeat) => self.next_pulse(start, end, repeat),
        }
    }
}
