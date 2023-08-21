use nanorand::{Rng, WyRand};
use vello::{
    kurbo::{Affine, BezPath, /*Line, PathEl,*/ Point, Rect, /*Shape,*/ Vec2},
    peniko::{BlendMode, Brush, Color, Fill, Stroke},
    /*BumpAllocators,*/ SceneBuilder,
};
use crate::{
    SimpleText,
};
use std::f64::consts::PI;

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

const NUM_LINES : usize = 5000;

#[derive(Clone, Copy, Debug, Default)]
struct Ray {
    pt: Point,
    len: f64,
    phi: f64,
}

impl Ray {
    fn spawn(rng: &mut WyRand, star_points: usize, width: f64, height: f64, frame_rotate: f64) -> Self {
        let mid_pt = Point { x: width / 2., y: height / 2. };
        let phiquant = 2. * PI / (star_points as f64);
        let rt_angle = 2. * PI / 4.;

        let ptidx = rng.generate_range(0u64..star_points as u64) as f64;

        const U32M : u64 = u32::MAX as u64;
        const U32MF64 : f64 = u32::MAX as f64;
        let n = rng.generate::<u64>();
        let frac = (rng.generate_range(0u64..U32M) as f64) / U32MF64;
        let phi = 2. * PI * frac;
        let radius = (width / 2.).min(height / 2.);
        let r : f64 = radius * (n % U32M) as f64 / U32MF64;
        let len : f64 = 10. * (n / U32M) as f64 / U32MF64;
        let pt = Point {
            x : mid_pt.x + r * f64::cos(ptidx * phiquant + rt_angle),
            y : mid_pt.y - r * f64::sin(ptidx * phiquant + rt_angle),
        };
        let rotate = Affine::rotate_about(frame_rotate, mid_pt); 
        let pt = rotate * pt;
        //println!("Spawn: pt {}, len {}, phi {}", pt, len, phi);
        Self {
            pt,
            len,
            phi,
        }
    }

    fn update(&mut self, rng: &mut WyRand, star_points: usize, width: f64, height: f64, frame_rotate: f64) {
        const OMEGA_SCALE : f64 = 8.;
        if self.len < 100. {
            let len = self.len.max(1.0);
            let omega = OMEGA_SCALE / (len * len);
            //let omega = OMEGA_SCALE / len;
            self.len += 3.;
            self.phi = (self.phi + omega) % (2. * PI);
        } else {
            *self = Self::spawn(rng, star_points, width, height, frame_rotate);
        }
    }
}

#[derive(Debug)]
pub struct Face {
    rng: WyRand,
    xscale: f64,
    yscale: f64,
    star_points: usize,
    rotate: f64,
    omega: f64,
    rays: [Ray; NUM_LINES],
    rays_init: usize,
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn n_pointed_star(num_points: usize, width: f64, height: f64) -> BezPath {
    let x_mid = width / 2.;
    let y_mid = height / 2.;
    let phi = 2. * PI / (num_points as f64);
    let rt_angle = 2. * PI / 4.;
    let radius = (width / 2.).min(height / 2.);
    let mut bez = BezPath::new();
    for idx in 0..num_points {
        let ptidx = if num_points > 4 { (idx * 2 % num_points) as f64 } else { idx as f64 };
        let p = Point {
            x : x_mid + radius * f64::cos(ptidx * phi + rt_angle),
            y : y_mid - radius * f64::sin(ptidx * phi + rt_angle),
        };
        if idx == 0 { bez.move_to(p); }
        else { bez.line_to(p) }
    }
    bez.close_path();
    bez
}

impl Face {
    pub fn new(star_points: usize, width: f64, height: f64) -> Self {
        let mut rng = WyRand::new();
        Self {
            xscale : 2./3.,
            yscale : 2./3.,
            star_points,
            rotate : 0.,
            omega : (2. * PI) * (1. / 360.), // 1° per frame
            rng,
            rays : [Default::default(); NUM_LINES],
            rays_init : 0,
        }
    }

    pub fn set_num_star_points(&mut self, num_points: usize) {
        self.star_points = num_points;
    }

    pub fn incr_star_points(&mut self) {
        self.star_points += 1;
    }

    pub fn decr_star_points(&mut self) {
        if self.star_points > 3 {
            self.star_points -= 1;
        }
    }


    fn draw_ray(&mut self,
        sb: &mut SceneBuilder,
        width: f64,
        height: f64,
        offset: Affine,
        idx: usize,
    )
    {
        let ray : &mut Ray = &mut self.rays[idx];
        let v = Vec2::from_angle(ray.phi);
        let p0 = ray.pt + ray.len * v;
        let p1 = ray.pt - ray.len * v;
        //if idx == 0 { println!("D Ray 0 : {:?}, p0 {}, p1 {}", ray, p0, p1); }
        let mut bez = BezPath::new();
        bez.move_to(p0);
        bez.line_to(p1);
        bez.close_path();

        //let rotate = Affine::rotate_about(self.rotate, ray.pt); 

        sb.stroke(
            &Stroke::new((1.0) as f32),
            offset,
            Color::hlc(v.x * 360., 50. + v.y * 50., 100.),
            None,
            &bez,
        );
    }

    /// Draw face
    #[allow(clippy::too_many_arguments)]
    pub fn draw_layer<'a, T>(
        &mut self,
        sb: &mut SceneBuilder,
        text: &mut SimpleText,
        viewport_width: f64,
        viewport_height: f64,
    )
    {
        //use PathEl::*;
        let width = self.xscale * viewport_width; //.max(100.);//(viewport_width * 0.49).max(200.).min(2000.);
        let height = self.yscale * viewport_height;
        
        let mut cnt : usize = 0;
        let idx = self.rays_init;
        while idx+cnt < NUM_LINES && cnt < 10 {
            self.rays[idx+cnt] = Ray::spawn(&mut self.rng, self.star_points, width, height, self.rotate + self.omega);
            cnt += 1;
        }
        self.rays_init = idx+cnt;

        let x_offset = (viewport_width - width) / 2.;
        let y_offset = (viewport_height - height) / 2.;
        let offset = Affine::translate((x_offset, y_offset));

        let clip_shape = n_pointed_star(self.star_points, width, height);
        let blend : BlendMode = Default::default();

        let (xsh, ysh) = (width / 2., height / 2.);
        //let rotate = 
        //    Affine::translate((xsh, ysh))
        //    * Affine::rotate(self.rotate) 
        //    * Affine::translate((-xsh, -ysh));
        let rotate = Affine::rotate_about(self.rotate, Point {x:xsh, y:ysh}); 

        // Set clip region
        sb.push_layer(
            blend,        // blend
            1.,           // alpha
            offset * rotate,       // transform
            &clip_shape,  // shape
        );

        const BGCOL : Brush = Brush::Solid(Color::rgba8(0, 0, 0, 255));
        // Draw the background
        sb.fill(
            Fill::NonZero,
            offset,
            &BG_COL,
            None,
            &Rect::new(0., 0., width, height),
        );

        for idx in 0..NUM_LINES { self.draw_ray(sb, width, height, offset, idx); }

        sb.pop_layer();

        // Update dynamic vars
        self.rotate += self.omega;
        for idx in 0..NUM_LINES {
            let ray : &mut Ray = &mut self.rays[idx];
            ray.update(&mut self.rng, self.star_points, width, height, self.rotate + 12. * self.omega);
            //if idx == 0 { println!("U Ray 0 : {:?}", ray); }
        }
    }
}

