use crate::{Sampler, DigiSigGen, SineGen, AnaSigGen};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SigType {
    Digital,
    Analog,
}

pub struct DataStore {
}

impl DataStore {
    pub fn new_test() -> Self {
        Self {
        }
    }

    pub fn get_num_signals(&self) -> usize {
        64
    }

    pub fn get_signal_ypos(&self, sig: usize) -> f64 {
        let mut acc = 0.;
        for n in 0..(sig-1) {
            let (sigtype, _) = self.get_signal_type_idx(sig);
            acc += if sigtype == SigType::Digital { crate::HEIGHT_DIGITAL } else { crate::HEIGHT_ANALOG }
        }
        acc
    }

    pub fn get_signal_type_idx(&self, sig: usize) -> (SigType, usize) {
        use SigType::*;
        if sig < 12 {
            match sig % 12 {
                0 => (Digital, 0),
                1 => (Analog, 0),
                2 => (Digital, 1),
                3 => (Digital, 2),
                4 => (Digital, 3),
                5 => (Analog, 1),
                6 => (Digital, 4),
                7 => (Digital, 5),
                8 => (Digital, 6),
                9 => (Digital, 7),
                10 => (Digital, 8),
                11 | _ => (Digital, 9),
            }
        } else {
            (Digital, sig-12+10)
        }
    }

    pub fn get_dig_sampler(&self, didx: usize, sig: usize) -> Box<dyn Sampler<bool>> {
        let smpl : Box<dyn Sampler<bool>> = match sig % 12 {
            0 => if sig < 12 { Box::new(DigiSigGen::new_clock(sig, 1000000.)) } else { Box::new(DigiSigGen::new_fixed(sig, false)) },
            1 | 5 | 6 | 7 => Box::new(DigiSigGen::new_fixed(sig, true)),
            3 => Box::new(DigiSigGen::new_pulse(sig, 57000000., 58000000., 8000000.)),
            9 => Box::new(DigiSigGen::new_pulse(sig, 10000000., 99000000., 8000000.)),
            _ => Box::new(DigiSigGen::new_fixed(sig, false)),
        };
        smpl
    }

    pub fn get_ana_sampler(&self, aidx: usize, sig: usize) -> Box<dyn Sampler<f32>> {
        let smpl : Box<dyn Sampler<f32>> = match aidx {
            0 => Box::new(SineGen::new(sig, 15., 0., 500000.)),
            1 => Box::new(AnaSigGen::new_pulse(sig, 0., 10., 57000000., 58000000., 8000000.)),
            _ => Box::new(AnaSigGen::new_fixed(sig, 0.5)),
        };
        smpl
    }
}

