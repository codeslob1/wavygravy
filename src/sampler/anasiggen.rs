use std::marker::PhantomData;
use crate::{Sampler, Time, TimeUnit, TimeScale};

#[derive(Debug)]
enum SigGenType<T> {
    // Generated signal types
    Fixed(T /*height*/),
    Pulse(T /*off*/, T /*on*/, f64 /*start*/, f64/*end*/, f64 /*repeat (after start)*/)
}

#[derive(Debug)]
pub struct AnaSigGen {
    sig: usize,
    stype: SigGenType<f32>,
    scale: TimeScale, // Sampling scale
}

impl AnaSigGen {
    pub fn new_fixed(idx: usize, height: f32) -> Self {
        AnaSigGen {
            sig: idx,
            stype: SigGenType::Fixed(height),
            scale: TimeScale { time: 1., unit: TimeUnit::Fs },
        }
    }

    pub fn new_pulse(idx: usize, off: f32, on: f32, start: f64, end: f64, repeat: f64) -> Self {
        AnaSigGen {
            sig: idx,
            stype: SigGenType::Pulse(off, on, start, end, repeat),
            scale: TimeScale { time: 1., unit: TimeUnit::Fs },
        }
    }
}

impl Sampler<f32> for AnaSigGen {
    fn get_height(&self) -> f64 { crate::HEIGHT_ANALOG }
    
    /// Return signal y scale (peak-to-peak height)
    fn get_yscale(&self) -> f64 {
        let height : f64 = match self.stype {
            SigGenType::Fixed(_height) => 0.,
            SigGenType::Pulse(off, on, _, _, _) => (on - off) as f64,
        };
        height.max(16.)
    }

    fn get_label(&self) -> String {
        format!("analog_{}", self.sig)
    }

    fn iter_range(&self, range: &[f64; 2]) -> Box<dyn Iterator<Item = (f32, Time)> + '_> {
        let iter = Box::new(AnaSigIter {
            smpl: self,
            pos: range[0]-f64::EPSILON,
            range: *range,
            done: false,
            phantom: PhantomData,
        });
        //println!("iter_range: {:?}", iter);
        iter
    }

    fn get_value_at(&self, t: Time, _s: TimeScale) -> f32 {
        match self.stype {
            SigGenType::Fixed(height) => height,
            SigGenType::Pulse(off, on, start, end, repeat) => {
                if t < start {
                    let delta = 1. - ((start - t) / start) as f32;
                    let val = off + (on - off) * delta;
                    //println!("t < start: {} = {}", t, val);
                    val
                } else {
                    let tdelta = (t - start) % repeat;
                    if tdelta < (end - start) {
                        let delta = (tdelta % (end - start) / (end - start)) as f32;
                        on - (on - off) * (delta as f32)
                    } else {
                        let delta = ((tdelta - (end - start)) % (repeat - (end - start)) / (repeat - (end - start))) as f32;
                        off + (on - off) * delta
                    }
                }
            },
        }
    }

    /// Set iteration scale
    fn set_iter_scale(&mut self, range: &[f64; 2], timescale: &TimeScale, scale_width: f64) {
        let scale = TimeScale {
            time: (range[1] - range[0]) * timescale.time / scale_width,
            unit: timescale.unit,
        };
        self.scale = scale;
    }
}

#[derive(Debug)]
pub struct AnaSigIter<'r, T> {
    smpl: &'r AnaSigGen,
    pos: Time,
    range: [Time; 2],
    done: bool,
    phantom: PhantomData<T>,
}

impl AnaSigIter<'_, f32> {
    fn next_pulse(&mut self, off: f32, on: f32, start: f64, end: f64, repeat: f64) -> Option<(f32, Time)> {
        if self.done { return None }

        if self.pos < start {
            self.pos = start;
            Some((on, start))
        } else {
            let now = self.pos;
            let base = ((now - start) / repeat).floor() * repeat;
            let offs = (now - start) % repeat;
            let (next_val, next_pos) = if offs < (end - start) {
                (off, start + base + (end - start))
            } else {
                (on, start + base + repeat)
            };
            if next_pos < self.range[1] {
                //println!("next_pulse: now {}, base {}, offs {},  {},{}", 
                //         now, base, offs, next_val, next_pos);
                self.pos = next_pos;
                Some((next_val, next_pos))
            } else {
                self.done = true;
                let next_val = self.smpl.get_value_at(self.range[1], /* bs timescale */ TimeScale { time: 1., unit: TimeUnit::Fs });
                Some((next_val, self.range[1]))
            }
        }
    }

}

impl Iterator for AnaSigIter<'_, f32> {
    type Item = (f32, Time);

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
            SigGenType::Pulse(off, on, start, end, repeat) => self.next_pulse(off, on, start, end, repeat),
        }
    }
}
