use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use crate::{AnaSigGen, DigiSig, DigiSigGen, Result, Sampler, SineGen, /*TimeRel,*/ TimeScale};
use wavefmt::{FieldType, WaveFile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    #[allow(dead_code)]
    TryAny, // Not yet
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SigType {
    Digital,
    Analog,
}

pub struct DataStore {
    timescale: TimeScale,
    sigs   : Vec<(SigType, usize)>,
    digsam : Vec<Rc<RefCell<dyn Sampler<bool>>>>,
    anasam : Vec<Rc<RefCell<dyn Sampler<f32>>>>,
}

impl Default for DataStore {
    fn default() -> Self {
        Self {
            timescale: Default::default(),
            sigs   : Vec::new(),
            digsam : Vec::new(),
            anasam : Vec::new(),
        }
    }
}

impl DataStore {
    pub fn new(timescale: TimeScale) -> Self {
        Self {
            timescale,
            sigs   : Vec::new(),
            digsam : Vec::new(),
            anasam : Vec::new(),
        }
    }

    pub fn load_wave(&mut self, path: PathBuf, ftype: FileType) -> Result<()> {
        match ftype {
            _ => {
                let msg = format!("File type: {:?}", ftype);
                return Err(Box::new(Error::new(ErrorKind::Unsupported, msg)));
            }
        }
    }

    pub fn new_test(timescale: TimeScale) -> Self {
        use SigType::*;
        let mut sigs   : Vec<(SigType, usize)> = Vec::new();
        let mut digsam : Vec<Rc<RefCell<dyn Sampler<bool>>>> = Vec::new();
        let mut anasam : Vec<Rc<RefCell<dyn Sampler<f32>>>> = Vec::new();
        let sigtypes : [SigType; 12] = [
            Digital, Analog, Digital, Digital, Digital, Analog, Digital, Digital, Digital, Digital, Digital, Digital,
        ];
        for _n in 0..5 {
            for (sig,sigtype) in sigtypes.into_iter().enumerate() {
                let samidx = if sigtype == Digital {
                    let cur = digsam.len();
                    let smpl : Rc<RefCell<dyn Sampler<bool>>> = match sig % 12 {
                        0 => if sig < 12 { Rc::new(RefCell::new(DigiSigGen::new_clock(sig, 1000000.))) }
                             else { Rc::new(RefCell::new(DigiSigGen::new_fixed(sig, false))) },
                        1 | 5 | 6 | 7 => Rc::new(RefCell::new(DigiSigGen::new_fixed(sig, true))),
                        3 => Rc::new(RefCell::new(DigiSigGen::new_pulse(sig, 57000000., 58000000., 8000000.))),
                        9 => Rc::new(RefCell::new(DigiSigGen::new_pulse(sig, 10000000., 99000000., 8000000.))),
                        _ => Rc::new(RefCell::new(DigiSigGen::new_fixed(sig, false))),
                    };
                    digsam.push(smpl);
                    cur
                } else {
                    let cur = anasam.len();
                    let smpl : Rc<RefCell<dyn Sampler<f32>>> = match anasam.len() {
                        0 => Rc::new(RefCell::new(SineGen::new(sig, 15., 0., 500000.))),
                        1 => Rc::new(RefCell::new(AnaSigGen::new_pulse(sig, 0., 10., 57000000., 58000000., 8000000.))),
                        _ => Rc::new(RefCell::new(AnaSigGen::new_fixed(sig, 0.5))),
                    };
                    anasam.push(smpl);
                    cur
                };
                sigs.push((sigtype, samidx));
            }
        }
        Self {
            timescale,
            sigs,
            digsam,
            anasam,
        }
    }

    /// Get maximum start, end time of all waveforms
    pub fn get_range(&self) -> (f64, f64) {
        let mut start = 0.0f64;
        let mut end = 10000000000.0f64;
        /*
        for w in self.ws_XXX.iter() {
            let (tstart, tend) = w.borrow().get_range();
            if start > tstart { start = tstart; }
            if end < tend { end = tend; }
        }
        */
        (start, end)
    }

    pub fn get_num_signals(&self) -> usize {
        self.sigs.len()
    }

    #[allow(dead_code)]
    pub fn get_signal_ypos(&self, sig: usize) -> f64 {
        let mut acc = 0.;
        for _n in 0..(sig-1) {
            let (sigtype, _) = self.get_signal_type_idx(sig);
            acc += if sigtype == SigType::Digital { crate::HEIGHT_DIGITAL } else { crate::HEIGHT_ANALOG }
        }
        acc
    }

    pub fn get_signal_type_idx(&self, sig: usize) -> (SigType, usize) {
        self.sigs[sig]
    }

    pub fn get_dig_sampler(&mut self, didx: usize) -> Option<&mut RefCell<dyn Sampler<bool>>> {
        Rc::get_mut(&mut self.digsam[didx])
    }

    pub fn get_ana_sampler(&mut self, aidx: usize) -> Option<&mut RefCell<dyn Sampler<f32>>> {
        Rc::get_mut(&mut self.anasam[aidx])
    }
}

