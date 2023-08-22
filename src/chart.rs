use vello::{
    kurbo::{Affine, /*Line,*/ PathEl, Rect},
    peniko::{BlendMode, Brush, Color, Fill, Stroke},
    /*BumpAllocators,*/ SceneBuilder,
};
use crate::{
    Time, TimeUnit, TimeScale, Vec2,
    Sampler, DataStore, SimpleText,
};

const BG_COL : Brush = Brush::Solid(Color::rgba8(0, 0, 0, 255));
const WVLO_COL : Brush = Brush::Solid(Color::rgba8(0, 0, 200, 255));
const WVHI_COL : Brush = Brush::Solid(Color::rgba8(0, 100, 200, 255));
const WVDN_COL : Brush = Brush::Solid(Color::rgba8(100, 0, 200, 255));
const WVUP_COL : Brush = Brush::Solid(Color::rgba8(100, 100, 200, 255));
const WVANA_COL : Brush = Brush::Solid(Color::rgba8(255, 100, 0, 255));
const YSCRLBOX_COL : Brush = Brush::Solid(Color::rgba8(200, 0, 200, 255));
const YSCRLARR_COL : Brush = Brush::Solid(Color::rgba8(0, 0, 0, 255));
const XSCRLLOC_COL : Brush = Brush::Solid(Color::rgba8(0, 180, 0, 180));
const CURS_COL : Brush = Brush::Solid(Color::YELLOW);
const XSCRLCURS_COL : Brush = Brush::Solid(Color::RED);

const RULE_HEIGHT : f64 = 16.;
const SCROLL_WIDTH : f64 = 16.;

/// How close (in pixels) we have to be to grab column header adjustment
const COLHDR_REACH : f64 = 10.;
const COLWIDTH_MIN : f64 = 16.;
const SIGWIDTH_MIN : f64 = 32.;

#[derive(Debug)]
enum MouseRegion {
    None,
    Waveform,
    YScrollBar,
    XScrollRuler,
    ColSignameHdr,
    ColValueHdr,
}

#[derive(Debug)]
pub enum MouseCursor {
    Normal,
    Column,
}

#[derive(Debug)]
pub struct Chart {
    pub time_range: [Time; 2],
    pub time_scale: TimeScale,
    pub max_range: [Time; 2],
    pub col_signame : f64,
    pub col_value : f64,
    pub cursor: Option<Time>,
    mregion: MouseRegion,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            time_range: [0.0, 0.0],
            time_scale: TimeScale { time: 1.0, unit: TimeUnit::Ps },
            max_range: [0.0, 0.0],
            col_signame: 0.2,
            col_value: 0.05,
            cursor: None,
            mregion: MouseRegion::None,
        }
    }

    pub fn set_range(&mut self, range: &[f64; 2], scale: &TimeScale) {
        self.time_range = *range;
        self.time_scale = *scale;
    }

    pub fn set_max_range(&mut self, range: &[f64; 2], _scale: &TimeScale) {
        self.max_range = *range;
        //self.max_scale = *scale;
    }

    pub fn set_cursor(&mut self, t: f64) {
        self.cursor = Some(t);
    }

    #[allow(dead_code)]
    pub fn clear_cursor(&mut self) {
        self.cursor = None;
    }

    /// Convert time to screen x position
    pub fn time_to_xpos(&self, t: Time, range: &[Time; 2], sig_xoffs: f64, sig_width: f64) -> f64 {
        sig_xoffs + sig_width * (t - range[0]) / (range[1] - range[0])
    }

    /// Convert screen x position to time, clipping to range bounds
    pub fn xpos_to_time(&self, x: f64, range: &[Time; 2], sig_xoffs: f64, sig_width: f64) -> Time {
        let t = (x - sig_xoffs) * (range[1] - range[0]) / sig_width + range[0];
        if t < range[0] {
            range[0]
        } else if t > range[1] {
            range[1]
        } else {
            t
        }
    }

    fn is_signame_region(&self, pos: &Vec2, width: f64, height: f64) -> bool {
        if pos.y <= RULE_HEIGHT {
            let col_signame_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame);
            let dist_signame = (pos.x - col_signame_x).abs();
            dist_signame < COLHDR_REACH
        } else { false }
    }

    fn is_value_region(&self, pos: &Vec2, width: f64, height: f64) -> bool {
        if pos.y <= RULE_HEIGHT {
            let col_value_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
            let dist_value = (pos.x - col_value_x).abs();
            dist_value < COLHDR_REACH
        } else { false }
    }

    /// Calculate mouse region from current position
    fn get_mouse_region(&self, pos: &Vec2, width: f64, height: f64) -> MouseRegion {
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;

        // Scroll bar region
        if pos.x >= width - SCROLL_WIDTH {
            MouseRegion::YScrollBar

        // Column header (top ruler): signame
        } else if self.is_signame_region(pos, width, height) {
            MouseRegion::ColSignameHdr

        // Column header (top ruler): value
        } else if self.is_value_region(pos, width, height) {
            MouseRegion::ColValueHdr

        // Bottom ruler (scroll) region
        } else if pos.y >= height - RULE_HEIGHT {
            MouseRegion::XScrollRuler

        // Waveform region
        } else if pos.y > RULE_HEIGHT && pos.y < height - RULE_HEIGHT 
            && pos.x >= sig_xoffs && pos.x < sig_xoffs + sig_width 
        {
            MouseRegion::Waveform

        } else {
            MouseRegion::None
        }
    }

    fn handle_wave_click(&mut self, pos: &Vec2, width: f64, _height: f64) -> bool {
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;
        let t = self.xpos_to_time(pos.x, &self.time_range, sig_xoffs, sig_width);
        self.set_cursor(t);
        true
    }

    fn handle_xscroll_click(&mut self, pos: &Vec2, width: f64, _height: f64) -> bool {
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;
        // Adjust zoom around waveform midpoint
        let zoom_range = self.time_range[1] - self.time_range[0];
        let mut t = self.xpos_to_time(pos.x, &self.max_range, sig_xoffs, sig_width);
        if t < self.max_range[0] + 0.5 * zoom_range { t = self.max_range[0] + 0.5 * zoom_range; }
        if t > self.max_range[1] - 0.5 * zoom_range { t = self.max_range[1] - 0.5 * zoom_range; }
        self.time_range[0] = t - 0.5 * zoom_range;
        self.time_range[1] = t + 0.5 * zoom_range;
        // Bounds check
        if self.time_range[0] < self.max_range[0] { self.time_range[0] = self.max_range[0] }
        if self.time_range[1] > self.max_range[1] { self.time_range[1] = self.max_range[1] }
        true
    }

    fn handle_colhdr_click(&mut self, pos: &Vec2, width: f64, _height: f64) -> bool {
        let frac = pos.x / width;
        match self.mregion {
            MouseRegion::ColSignameHdr => {
                let new_colsigname = if frac * width > COLWIDTH_MIN {
                    if (1. - (frac + self.col_value)) * width > SIGWIDTH_MIN {
                        frac
                    } else { 1. - (self.col_value + SIGWIDTH_MIN / width) }
                } else { COLWIDTH_MIN / width };
                //println!("signame frac {:.2}, col_signame {:.2}, col_value {:.2}, new {:.2}", frac, self.col_signame, self.col_value, new_colsigname);
                self.col_signame = new_colsigname;
                true
            }
            MouseRegion::ColValueHdr => {
                let new_colvalue = if (frac - self.col_signame) * width > COLWIDTH_MIN {
                    if (1. - frac) * width > SIGWIDTH_MIN {
                        frac - self.col_signame
                    } else {
                        1. - (self.col_signame + SIGWIDTH_MIN / width)
                    }
                } else {
                    COLWIDTH_MIN / width
                };
                //println!("value frac {:.2}, col_signame {:.2}, col_value {:.2}, new {:.2}", frac, self.col_signame, self.col_value, new_colvalue);
                self.col_value = new_colvalue;
                true
            }
            _ => false,
        }
    }

    fn handle_click(&mut self, pos: &Vec2, width: f64, height: f64) -> bool {
        match self.mregion {
            MouseRegion::ColSignameHdr => self.handle_colhdr_click(pos, width, height),
            MouseRegion::ColValueHdr => self.handle_colhdr_click(pos, width, height),
            MouseRegion::YScrollBar => false,
            MouseRegion::XScrollRuler => self.handle_xscroll_click(pos, width, height),
            MouseRegion::Waveform => self.handle_wave_click(pos, width, height),
            MouseRegion::None => false,
        }
    }

    /// Handle mouse down event, return true if handled
    pub fn handle_mousedown(&mut self, prior: &Option<Vec2>, width: f64, height: f64) -> bool {
        if let Some(pos) = prior {
            let col_signame_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame);
            let col_value_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
            let sig_xoffs : f64 = col_value_x;
            let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;

            self.mregion = self.get_mouse_region(pos, width, height);
            println!("mousedown {:.0},{:.0} - {:.0},{:.0}, region {:?}", pos.x, pos.y, sig_xoffs, sig_xoffs+sig_width, self.mregion);

            let handled = self.handle_click(&pos, width, height); 
            handled
        } else { false }
    }

    /// Handle mouse up event, return true if handled
    pub fn handle_mouseup(&self, _prior: &Option<Vec2>, _width: f64, _height: f64) -> bool {
        true
    }

    /// Handle mouse move event, return true if handled
    pub fn handle_mousemove(&mut self, pos: &Vec2, prior: &Option<Vec2>, width: f64, height: f64, mouse_down: bool) 
        -> (bool, MouseCursor)
    {
        //println!("mousemove {},{} - region {:?}", pos.x, pos.y, self.mregion);
        let col_signame_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame);
        let col_value_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_xoffs : f64 = col_value_x;
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;

        let mcurs : MouseCursor = if pos.y <= RULE_HEIGHT && 
            (self.is_signame_region(pos, width, height) || self.is_value_region(pos, width, height))
        {
            MouseCursor::Column
        } else { MouseCursor::Normal };

        let handled = if mouse_down { self.handle_click(&pos, width, height) } else { false };
        (handled, mcurs)
    }

    /// Change zoom (time window size) by `ratio`
    pub fn do_zoom(&mut self, ratio: f64) {
        let zoom_range = ratio * (self.time_range[1] - self.time_range[0]);

        if let Some(curs) = self.cursor {
            // Adjust zoom around cursor position
            self.time_range[0] = curs - 0.5 * zoom_range;
            self.time_range[1] = curs + 0.5 * zoom_range;
        } else {
            // Adjust zoom around waveform midpoint
            let midpt = self.time_range[0] + 0.5 * (self.time_range[1] - self.time_range[0]);
            self.time_range[0] = midpt - 0.5 * zoom_range;
            self.time_range[1] = midpt + 0.5 * zoom_range;
        }
        //println!("range {},{} max {},{}", self.time_range[0], self.time_range[1], 
        //         self.max_range[0], self.max_range[1]);
        // Bounds check
        if self.time_range[0] < self.max_range[0] { self.time_range[0] = self.max_range[0] }
        if self.time_range[1] > self.max_range[1] { self.time_range[1] = self.max_range[1] }
    }

    /// Handle mouse wheel event, return true if handled
    pub fn handle_mousewheel(&mut self, exponent: f64, _prior: &Vec2, _width: f64, _height: f64) -> bool {
        //let mregion = self.get_mouse_region(prior, width, height);
        //println!("mousewheel {} {},{} - region {:?}", exponent, prior.x, prior.y, mregion);
        let zoom_ratio = if exponent > 0. {
            0.5
        } else {
            2.0
        };
        self.do_zoom(zoom_ratio);
        true
    }

    /// Draw horizontal ruler, at top/bottom of waveform window
    pub fn draw_ruler(
        &self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        width: f64,
        _height: f64,
        region_offset: Affine,
        label_height: f32,
        rule_height: f64,
        x_offs: f64,
        y_offs: f64,
        range: &[Time; 2],
        scale: &TimeScale
    )
    {
        use PathEl::*;
        let offset = region_offset * Affine::translate((x_offs, y_offs));

        let time_range = (range[1] - range[0]) * scale.time; 
        let min_pixels = 4.;
        let range_per_min = time_range * min_pixels / width;
        let range_min_rnd_pow10 = range_per_min.log10();
        let min_scale = range_min_rnd_pow10.ceil();
        let min_step = (10.0f64).powf(min_scale);
        //println!("width {}, range {} {:?}", width, time_range, self.time_scale.scale);
        //println!("range per 10 pixel {} rnd {}, rounded {}, {}", range_per_min, min_step,
        //         range_min_rnd_pow10, min_scale);

        // Draw top ruler
        let marker_long = [
            MoveTo((0., 0.).into()),
            LineTo((0., rule_height).into()),
        ];
        let marker_medium = [
            MoveTo((0., rule_height * 0.5).into()),
            LineTo((0., rule_height).into()),
        ];
        let marker_short = [
            MoveTo((0., rule_height * 0.8).into()),
            LineTo((0., rule_height).into()),
        ];
        sb.fill(
            Fill::NonZero,
            offset,
            /*Color::DIM_GRAY,*/ &Brush::Solid(Color::rgba8(80, 80, 80, 255)),
            None,
            &Rect::new(0., 0., width, RULE_HEIGHT),
        );

        // Render ticks and time text
        let epsilon = 16. * f64::EPSILON;
        let range_per_pix = time_range / width;
        let mut xv = range[0] * scale.time;
        let mut xv_prev = xv - range_per_pix;
        
        let textwidth = 64; // Estimate - calculate properly from font
        let mut last_str_xpos : i32 = 0 - textwidth;
        for xpos in 0..(width.round() as i32) {
            let xv_next = xv + range_per_pix;
            let xv_step = (xv / min_step).floor();
            let xv_step_round = xv_step * min_step;
            let xv_prev_step = (xv_prev / min_step).floor();
            let xv_active = xv_step - xv_prev_step > epsilon;
            let xv_major = if xv_active {
                ((xv_step / 10.).floor() - (xv_prev_step / 10.).floor()) > epsilon
            } else { false };
            let xv_semi = if !xv_major {
                ((xv_step / 5.).floor() - (xv_prev_step / 5.).floor()) > epsilon
            } else { false };
            //println!("xv_step {}, xv_prev_step {}, xv_active {}, xv_major {}, xv_semi {}", xv_step, xv_prev_step,
            //         xv_active, xv_major, xv_semi);
            if xv_active {
                let xoffs : f64 = xpos as f64;
                let yoffs : f64 = 0.;
                let marker_ref = if xv_major {
                    if xpos - last_str_xpos > textwidth {
                        let label = crate::fmt_time(xv_step_round); //self.time_range[0] * self.time_scale.time);
                        text.add(
                            sb,
                            None,
                            label_height,
                            Some(&Brush::Solid(Color::YELLOW)),
                            offset * Affine::translate((xoffs + 1., yoffs + RULE_HEIGHT - 5.)),
                            &label,
                        );
                        last_str_xpos = xpos;
                    }
                    &marker_long
                } else {
                    if xv_semi { &marker_medium } else { &marker_short }
                };
                sb.stroke(
                    &Stroke::new((1.0) as f32),
                    offset * Affine::translate((xoffs, yoffs)),
                    Color::YELLOW,
                    None,
                    marker_ref,
                );
            }

            xv_prev = xv;
            xv = xv_next;
        }
        // Top/bottom borders
        let hline = [
            MoveTo((0., 0.).into()),
            LineTo((width, 0.).into()),
        ];
        sb.stroke(
            &Stroke::new((1.0) as f32),
            offset,
            Color::LIGHT_GRAY,
            None,
            &hline,
        );
        sb.stroke(
            &Stroke::new((1.0) as f32),
            offset * Affine::translate((0., RULE_HEIGHT)),
            Color::LIGHT_GRAY,
            None,
            &hline,
        );
    }

    /// Draw column headers at top of waveform window
    pub fn draw_colhdr(
        &self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        width: f64,
        _height: f64,
        offset: Affine,
    )
    {
        use PathEl::*;
        // Column header bars
        let marker_long = [
            MoveTo((0., 0.).into()),
            LineTo((0., RULE_HEIGHT).into()),
        ];
        let col_signame_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame);
        sb.stroke(
            &Stroke::new((2.0) as f32),
            offset * Affine::translate((col_signame_x, 0.)),
            Color::YELLOW,
            None,
            &marker_long,
        );
        let col_value_x : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        sb.stroke(
            &Stroke::new((2.0) as f32),
            offset * Affine::translate((col_value_x, 0.)),
            Color::YELLOW,
            None,
            &marker_long,
        );
    }

    /// Draw individual digital waveform at a specific location
    pub fn draw_digital(
        &self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        width: f64,
        _height: f64,
        region_offset: Affine,
        label_height: f32,
        signal_height: f64,
        y_offs : f64,
        smpl: Box<dyn Sampler<bool>>,
    )
    {
        use PathEl::*;
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;

        let label = smpl.get_label();
        text.add(
            sb,
            None,
            label_height,
            Some(&Brush::Solid(Color::WHITE)),
            region_offset * Affine::translate((0., y_offs + signal_height - 2.0)),
            &label,
        );

        let meas_pos = if let Some(curs) = self.cursor { curs } else { self.time_range[0] };
        let value_bool = smpl.get_value_at(meas_pos, self.time_scale);
        let value = if value_bool { "1" } else { "0" };
        text.add(
            sb,
            None,
            label_height,
            Some(&Brush::Solid(Color::WHITE)),
            region_offset * Affine::translate((self.col_signame * width, y_offs + signal_height - 2.0)),
            &value,
        );
        
        let mut curval = smpl.get_value_at(self.time_range[0], self.time_scale);
        let mut curtime = self.time_range[0];
        let y_hi : f64 = y_offs + 2.;
        let y_lo : f64 = y_offs + signal_height - 1.;
        for (nxval, nxtime) in smpl.iter_range(&self.time_range) {
            let x_cur = self.time_to_xpos(curtime, &self.time_range, sig_xoffs, sig_width);
            let x_nxt = self.time_to_xpos(nxtime, &self.time_range, sig_xoffs, sig_width);
            //print!("{} ({},{},{}) -> {},{} ", curtime, curval, nxval, nxtime, x_cur, x_nxt);
            match (curval, nxval) {
                (false, true) => {
                    //println!("RISE");
                    let line0 = [
                        MoveTo((x_cur, y_lo).into()),
                        LineTo((x_nxt, y_lo).into()),
                    ];
                    sb.stroke(
                        &Stroke::new((1.0) as f32),
                        region_offset,
                        &WVLO_COL,
                        None,
                        &line0,
                    );
                    let line1 = [
                        MoveTo((x_nxt, y_lo).into()),
                        LineTo((x_nxt, y_hi).into()),
                    ];
                    sb.stroke(
                        &Stroke::new((1.0) as f32),
                        region_offset,
                        &WVUP_COL,
                        None,
                        &line1,
                    );
                }
                (true, false) => {
                    //println!("FALL");
                    let line0 = [
                        MoveTo((x_cur, y_hi).into()),
                        LineTo((x_nxt, y_hi).into()),
                    ];
                    sb.stroke(
                        &Stroke::new((1.0) as f32),
                        region_offset,
                        &WVHI_COL,
                        None,
                        &line0,
                    );
                    let line1 = [
                        MoveTo((x_nxt, y_lo).into()),
                        LineTo((x_nxt, y_hi).into()),
                    ];
                    sb.stroke(
                        &Stroke::new((1.0) as f32),
                        region_offset,
                        &WVDN_COL,
                        None,
                        &line1,
                    );
                }
                _ => {
                    //println!("LEVEL");
                    let (y_val, col) = if nxval { (y_hi, WVHI_COL) } else { (y_lo, WVLO_COL) };
                    let line0 = [
                        MoveTo((x_cur, y_val).into()),
                        LineTo((x_nxt, y_val).into()),
                    ];
                    sb.stroke(
                        &Stroke::new((1.0) as f32),
                        region_offset,
                        &col,
                        None,
                        &line0,
                    );
                }
            }
            curval = nxval;
            curtime = nxtime;
        }
        // Extend signal level to waveform edge
        if curtime < self.time_range[1] {
            let x_cur = self.time_to_xpos(curtime, &self.time_range, sig_xoffs, sig_width);
            //println!("LEVEL");
            let (y_val, col) = if curval { (y_hi, WVHI_COL) } else { (y_lo, WVLO_COL) };
            let line0 = [
                MoveTo((x_cur, y_val).into()),
                LineTo((width - SCROLL_WIDTH, y_val).into()),
            ];
            sb.stroke(
                &Stroke::new((1.0) as f32),
                region_offset,
                &col,
                None,
                &line0,
            );
        }
    }

    /// Draw individual analog waveform at a specific location
    pub fn draw_analog(
        &self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        width: f64,
        _height: f64,
        region_offset: Affine,
        label_height: f32,
        signal_height: f64,
        y_offs : f64,
        smpl: Box<dyn Sampler<f32>>,
    )
    {
        use PathEl::*;
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;
        let label_yoffs : f64 = 0.5 * (signal_height as f64) + (label_height as f64)/2.;
        let yscale = smpl.get_yscale();

        fn value_to_ypos(val: f32, yscale: f64, yoffs: f64, height: f64) -> f64 {
            height / yscale * (-val as f64) / 2. + yoffs + height/2.
        }

        let label = smpl.get_label();
        text.add(
            sb,
            None,
            label_height,
            Some(&Brush::Solid(Color::WHITE)),
            region_offset * Affine::translate((0., y_offs + label_yoffs)),
            &label,
        );

        let meas_pos = if let Some(curs) = self.cursor { curs } else { self.time_range[0] };
        let sigval = smpl.get_value_at(meas_pos, self.time_scale);
        let value = format!("{:.2}", sigval);
        //let value = value_to_ypos(sigval, yscale, y_offs, );
        text.add(
            sb,
            None,
            label_height,
            Some(&Brush::Solid(Color::WHITE)),
            region_offset * Affine::translate((self.col_signame * width, y_offs + label_yoffs)),
            &value,
        );
        
        let mut curval = smpl.get_value_at(self.time_range[0], self.time_scale);
        let mut curtime = self.time_range[0];
        for (nxval, nxtime) in smpl.iter_range(&self.time_range) {
            let x_cur = self.time_to_xpos(curtime, &self.time_range, sig_xoffs, sig_width);
            let x_nxt = self.time_to_xpos(nxtime, &self.time_range, sig_xoffs, sig_width);
            let y_cur = value_to_ypos(curval, yscale, y_offs, signal_height);
            let y_nxt = value_to_ypos(nxval, yscale, y_offs, signal_height);
            //println!("{:.2} ({:.2},{:.2},{:.2}) -> {:.2},{:.2} {:.2},{:.2} ", curtime, curval, nxval, nxtime, x_cur, y_cur, x_nxt, y_nxt);
            let line0 = [
                MoveTo((x_cur, y_cur).into()),
                LineTo((x_nxt, y_nxt).into()),
            ];
            sb.stroke(
                &Stroke::new((1.0) as f32),
                region_offset,
                &WVANA_COL,
                None,
                &line0,
            );
            curval = nxval;
            curtime = nxtime;
        }
    }

    /// Draw y scrollbar
    pub fn draw_yscroll(
        &self,
        sb: &mut SceneBuilder,
        _text: &mut SimpleText,
        width: f64,
        height: f64,
        region_offset: Affine,
    )
    {
        use PathEl::*;
        let sb_x = width - SCROLL_WIDTH;
        let line0 = [
            MoveTo((sb_x, 0.).into()),
            MoveTo((width, 0.).into()),
            LineTo((width, height).into()),
            LineTo((sb_x, height).into()),
            LineTo((sb_x, 0.).into()),
        ];
        sb.stroke(
            &Stroke::new((1.0) as f32),
            region_offset,
            &YSCRLBOX_COL,
            None,
            &line0,
        );
        let y = 0.;
        sb.fill(
            Fill::NonZero,
            region_offset,
            &YSCRLBOX_COL,
            None,
            &Rect::new(sb_x+1., y+1., width-1., y+RULE_HEIGHT-1.),
        );
        let y = height - RULE_HEIGHT;
        sb.fill(
            Fill::NonZero,
            region_offset,
            &YSCRLBOX_COL,
            None,
            &Rect::new(sb_x+1., y+1., width-1., y+RULE_HEIGHT-1.),
        );
        let arr_w = width - sb_x;
        let mid_x = width-SCROLL_WIDTH + arr_w/2.;
        let arrow = [
            MoveTo((mid_x, 1.).into()),
            LineTo((width-1., RULE_HEIGHT/3.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*2./3., RULE_HEIGHT/3.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*2./3., RULE_HEIGHT-2.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*1./3., RULE_HEIGHT-2.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*1./3., RULE_HEIGHT/3.).into()),
            LineTo((sb_x+1., RULE_HEIGHT/3.).into()),
            LineTo((mid_x, 1.).into()),
        ];
        sb.stroke(
            &Stroke::new((1.0) as f32),
            region_offset,
            &YSCRLARR_COL,
            None,
            &arrow,
        );
        let arrow = [
            MoveTo((mid_x, height-1.).into()),
            LineTo((width-1., height-RULE_HEIGHT/3.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*2./3., height-RULE_HEIGHT/3.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*2./3., height-RULE_HEIGHT-2.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*1./3., height-RULE_HEIGHT-2.).into()),
            LineTo((width-SCROLL_WIDTH + arr_w*1./3., height-RULE_HEIGHT/3.).into()),
            LineTo((sb_x+1., height-RULE_HEIGHT/3.).into()),
            LineTo((mid_x, height-1.).into()),
        ];
        sb.stroke(
            &Stroke::new((1.0) as f32),
            region_offset,
            &YSCRLARR_COL,
            None,
            &arrow,
        );
    }

    /// Draw chart layer
    #[allow(clippy::too_many_arguments)]
    pub fn draw_layer<'a, T>(
        &self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        datas: &mut DataStore,
        viewport_width: f64,
        viewport_height: f64,
    )
    {
        use PathEl::*;
        let width = viewport_width; //.max(100.);//(viewport_width * 0.49).max(200.).min(2000.);
        let height = viewport_height;
        let x_offset = 0.; //viewport_width - width;
        let y_offset = 0.; //viewport_height - height;
        let offset = Affine::translate((x_offset, y_offset));
        let label_height = (RULE_HEIGHT - 4.) as f32;

        // Draw the background
        sb.fill(
            Fill::NonZero,
            offset,
            &BG_COL,
            None,
            &Rect::new(0., 0., width, height),
        );

        let rule_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let rule_width : f64 = (width - SCROLL_WIDTH) - rule_xoffs;
        // Top ruler
        self.draw_ruler(sb, text, rule_width, height, offset, label_height, RULE_HEIGHT, rule_xoffs, 0., &self.time_range, &self.time_scale);
        // Bottom ruler
        self.draw_ruler(sb, text, rule_width, height, offset, label_height, RULE_HEIGHT, rule_xoffs, height - RULE_HEIGHT, &self.max_range, &self.time_scale);
        // Draw locator box for current zoom level
        let sig_xoffs : f64 = (width - SCROLL_WIDTH) * (self.col_signame + self.col_value);
        let sig_width : f64 = (width - SCROLL_WIDTH) - sig_xoffs;
        let x0 = self.time_to_xpos(self.time_range[0], &self.max_range, sig_xoffs, sig_width);
        let x1 = self.time_to_xpos(self.time_range[1], &self.max_range, sig_xoffs, sig_width);
        let (x0, x1) = if x1 - x0 < 2.0 {
            let avg = (x1 + x0) / 2.; let x1 = avg + 1.; let x0 = avg - 1.;
            (x0, x1)
        } else { (x0, x1) };

        sb.fill(
            Fill::NonZero,
            offset,
            &XSCRLLOC_COL,
            None,
            &Rect::new(x0, height - RULE_HEIGHT, x1, height),
        );
    
        // Set clip region to waveform signals and labels
        let blend : BlendMode = Default::default();
        sb.push_layer(
            blend,        // blend
            1.,           // alpha
            offset,       // transform
            &Rect::new(0., RULE_HEIGHT, viewport_width, viewport_height - RULE_HEIGHT),  // shape
        );

        //const SIGNAL_HEIGHT : f64 = 16.;
        let mut height_acc : f64 = 0.0;
        let num_signals = datas.get_num_signals();
        for sig in 0..num_signals {
            use crate::datastore::SigType;
            let (sigtype, idx) = datas.get_signal_type_idx(sig);

            // Digital signal(s)
            let signal_height = if sigtype == SigType::Digital {
                let mut smpl = datas.get_dig_sampler(idx, sig);
                let signal_height = smpl.get_height();
                let y_pos = RULE_HEIGHT + height_acc;
                self.draw_digital(sb, text, width, height, offset, label_height, signal_height, y_pos, smpl);
                signal_height

            // Analog signal(s)
            } else {
                let mut smpl = datas.get_ana_sampler(idx, sig);
                smpl.set_iter_scale(&self.time_range, &self.time_scale, sig_width);
                let signal_height = smpl.get_height();
                let y_pos = RULE_HEIGHT + height_acc;
                self.draw_analog(sb, text, width, height, offset, label_height, signal_height, y_pos, smpl);
                signal_height
            };
            height_acc += signal_height;
        }

        sb.pop_layer();

        // Cursor
        if let Some(curs) = self.cursor {
            // Draw main cursor across top scrollbar and waveform window
            if curs >= self.time_range[0] && curs <= self.time_range[1] {
                let cursor_x = (curs - self.time_range[0]) * rule_width / (self.time_range[1] - self.time_range[0]) + rule_xoffs;
                //println!("Cursor {} -> {} [{}]", self.cursor, cursor_x, width);
                let vline = [
                    MoveTo((cursor_x, 0.).into()),
                    LineTo((cursor_x, height - RULE_HEIGHT).into()),
                ];
                sb.stroke(
                    &Stroke::new((0.6) as f32),
                    offset,
                    &CURS_COL,
                    None,
                    &vline,
                );
            }
            // Cursor time is displayed at top left
            let curs_unit = TimeScale {
                time: curs,
                unit: self.time_scale.unit,
            };
            let label = crate::fmt_time_unit(curs_unit);
            text.add(
                sb,
                None,
                label_height,
                Some(&Brush::Solid(Color::YELLOW)),
                offset * Affine::translate((0., RULE_HEIGHT - 5.)),
                &label,
            );
            // Draw global cursor (bottom scrollbar)
            let cursor_x = (curs - self.max_range[0]) * rule_width / (self.max_range[1] - self.max_range[0]) + rule_xoffs;
            //println!("Cursor {} -> {} [{}]", self.cursor, cursor_x, width);
            let vline = [
                MoveTo((cursor_x, height - RULE_HEIGHT).into()),
                LineTo((cursor_x, height).into()),
            ];
            sb.stroke(
                &Stroke::new((1.0) as f32),
                offset,
                &XSCRLCURS_COL,
                None,
                &vline,
            );
        }

        // Column headers
        self.draw_colhdr(sb, text, width, height, offset);

        // Y scrollbar
        self.draw_yscroll(sb, text, width, height, offset);
    }
}

