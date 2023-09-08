use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;
use crate::{Result, Sampler, TimeRel, /*TimeUnit,*/ TimeScale};
use wavefmt::{FieldInfo, WaveFile};

pub struct DigiSig<T: WaveFile> {
    wave: Rc<RefCell<T>>,
    sig: usize,
}

pub trait DigiSigIo {
    //fn seek_record(&mut self, idx: usize) -> Result<()>;
    fn read_record_with_time(&mut self, buf: &mut Vec<u8>, sig: usize) -> Result<(bool, TimeRel)>;
}

impl<T: WaveFile> DigiSig<T> {
    pub fn new(wave: Rc<RefCell<T>>, idx: usize) -> Self {
        DigiSig {
            wave,
            sig: idx,
        }
    }
}

impl<T: WaveFile + DigiSigIo> Sampler<bool> for DigiSig<T> {
    fn get_height(&self) -> f64 { crate::HEIGHT_DIGITAL }

    fn get_label(&self) -> String {
        let fi_bind = self.wave.borrow();
        let fi : &FieldInfo = fi_bind.get_field_info(self.sig);
        fi.name.clone()
    }

    #[inline(never)]
    fn iter_range(&self, range: &[f64; 2]) -> Result<Box<dyn Iterator<Item = (bool, TimeRel)> + '_>> {
        let mut wv_bind = self.wave.borrow_mut();
        //println!("iter_range [{:.02},{:.02}]", range[0], range[1]);
        let _sample_bounds : [usize; 2] = wv_bind.prepare_iter_range(range)?;
        //println!("iter_range sample_bounds [{:.02},{:.02}]", sample_bounds[0], sample_bounds[1]);
        let recbuf =
            if let Some(recsize) = wv_bind.get_record_size() {
                let mut recbuf : Vec<u8> = Vec::with_capacity(recsize);
                recbuf.resize(recsize, 0);
                recbuf
            } else { Vec::new() };

        let iter = Box::new(DigiSigIter {
            smpl: self,
            //pos: range[0]-f64::EPSILON,
            range: *range,
            //sample_bounds,
            recbuf,
            cnt: 0,
            phantom: PhantomData,
        });
        Ok(iter)
    }

    fn get_value_at(&self, t: TimeRel, _s: TimeScale) -> bool {
        //let fi : &FieldInfo = self.wave.borrow().get_sig_info(self.sig);
        false
    }
}

/*
pub struct DigiSigDummyIter {
}

impl<W: WaveFile> Iterator for DigiSigDummyIter {
    type Item = (bool, Time);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
*/

pub struct DigiSigIter<'r, B, W: WaveFile + DigiSigIo> {
    smpl: &'r DigiSig<W>,
    //pos: TimeRel,
    range: [TimeRel; 2],
    //sample_bounds: [usize; 2],
    recbuf: Vec<u8>,
    cnt: usize,
    phantom: PhantomData<B>,
}

impl<W: WaveFile + DigiSigIo> Iterator for DigiSigIter<'_, bool, W> {
    type Item = (bool, TimeRel);

    fn next(&mut self) -> Option<Self::Item> {
        let mut wv_bind = self.smpl.wave.borrow_mut();
        let (val, time) = wv_bind.read_record_with_time(&mut self.recbuf, self.smpl.sig).ok()?;
        //println!("rrt #{}: {},{:.02} [{:.02}..{:.02}]", self.cnt, val, time, self.range[0], self.range[1]);
        if time <= self.range[1] {
            self.cnt += 1;
            Some((val, time))
        } else {
            None
        }
    }
}
