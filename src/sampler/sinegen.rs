use std::marker::PhantomData;
use std::f64::consts::PI;
use crate::{Result, Sampler, TimeRel, TimeUnit, TimeScale};

#[derive(Debug)]
pub struct SineGen {
    sig: usize,
    height: f32,
    yoffs: f32,
    period: TimeRel,
    scale: TimeScale, // Sampling scale
    phi_delta: f64,
    time_delta: TimeRel,
}

impl SineGen {
    pub fn new(idx: usize, height: f32, yoffs: f32, period: f64) -> Self {
        SineGen {
            sig: idx,
            height,
            yoffs,
            period,
            scale: TimeScale { time: 1., unit: TimeUnit::Fs },
            phi_delta: 0.,
            time_delta: 0.,
        }
    }
}

impl Sampler<f32> for SineGen {
    fn get_height(&self) -> f64 { crate::HEIGHT_ANALOG }
    
    /// Return signal y scale (peak-to-peak height)
    fn get_yscale(&self) -> f64 {
        let height : f64 = self.height as f64 + f64::abs(self.yoffs as f64/2.);
        height.max(16.)
    }

    fn get_label(&self) -> String {
        format!("analog_{}", self.sig)
    }

    fn iter_range(&self, range: &[f64; 2]) -> Result<Box<dyn Iterator<Item = (f32, TimeRel)> + '_>> {
        let phi = range[0] / (2. * PI * self.period);
        let iter = Box::new(SineIter {
            smpl: self,
            phi_delta: self.phi_delta,
            time_delta: self.time_delta,
            pos: range[0]-self.time_delta,
            phi: phi - self.phi_delta,
            range: *range,
            phantom: PhantomData,
        });
        //println!("iter_range: {:?}", iter);
        Ok(iter)
    }

    fn get_value_at(&self, t: TimeRel, _s: TimeScale) -> f32 {
        let phi = t / (2. * PI * self.period);
        let val = ((self.yoffs as f64) + (self.height as f64) * f64::sin(phi)) as f32;
        //println!("get_value_at[sine] {:.2} = ..*sin({:.2}) = {:.2}", t, phi, val);
        val
    }

    /// Set iteration scale
    fn set_iter_scale(&mut self, range: &[f64; 2], timescale: &TimeScale, scale_width: f64) {
        let scale = TimeScale {
            time: (range[1] - range[0]) * timescale.time / scale_width,
            unit: timescale.unit,
        };
        //println!("set_iter_scale range:{}-{} timescale:{} scale_width:{}", range[0], range[1], timescale, scale_width);
        self.scale = scale;
        self.time_delta = scale.time;
        self.phi_delta = self.time_delta / (2. * PI * self.period as f64);
    }
}

#[derive(Debug)]
pub struct SineIter<'r, T> {
    smpl: &'r SineGen,
    phi_delta: f64,
    time_delta: TimeRel,
    phi: f64,
    pos: TimeRel,
    range: [TimeRel; 2],
    phantom: PhantomData<T>,
}

impl Iterator for SineIter<'_, f32> {
    type Item = (f32, TimeRel);

    fn next(&mut self) -> Option<Self::Item> {
        let next_pos = self.pos + self.time_delta;
        if next_pos <= self.range[1] {
            let next_phi = self.phi + self.phi_delta;
            //print!("next_sine: {:.2},{:.2} = ", next_pos, next_phi);
            self.pos = next_pos;
            self.phi = next_phi;
            let next_val = (self.smpl.yoffs as f64 + self.smpl.height as f64 * f64::sin(next_phi)) as f32;
            let time_scale = TimeScale { time: 1., unit: TimeUnit::Ps };
            //println!("{:.2} [{:.2}]", next_val, self.smpl.get_value_at(next_pos, time_scale));
            Some((next_val, next_pos))
        } else {
            None
        }
    }
}
