#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum TimeUnit {
    Fs,
    Ps,
    Ns,
    Us,
    Ms,
    S,
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            TimeUnit::Fs => "fs",
            TimeUnit::Ps => "ps",
            TimeUnit::Ns => "ns",
            TimeUnit::Us => "us",
            TimeUnit::Ms => "ms",
            TimeUnit::S  => "s",
        };
        f.write_str(s)
    }
}

/// Relative time (in global time units)
pub type TimeRel = f64;

#[derive(Debug, Clone, Copy)]
pub struct TimeScale {
    pub time: TimeRel,
    pub unit: TimeUnit,
}

impl std::fmt::Display for TimeScale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&fmt_time_unit(*self))
    }
}

impl Default for TimeScale {
    fn default() -> Self {
        Self {
            time: 1.0,
            unit: TimeUnit::Fs,
        }
    }
}

impl TimeScale {
    pub fn scale_factor(&self) -> TimeRel {
        match self.unit {
            TimeUnit::Fs => self.time * 1000_000_000_000_000.0,
            TimeUnit::Ps => self.time * 1000_000_000_000.0,
            TimeUnit::Ns => self.time * 1000_000_000.0,
            TimeUnit::Us => self.time * 1000_000.0,
            TimeUnit::Ms => self.time * 1000.0,
            TimeUnit::S  => self.time * 1.0,
        }
    }
}

/// Format time string
pub fn fmt_time(t: TimeRel) -> String {
    use num_format::{Buffer, Locale/*, WriteFormatted*/};
    // Create a stack-allocated buffer...
    let mut buf = Buffer::default();
    let n = t.round() as i64;
    buf.write_formatted(&n, &Locale::en);
    buf.as_str().to_string()
}

/// Format time string with scale (units)
pub fn fmt_time_unit(ts: TimeScale) -> String {
    use num_format::{Buffer, Locale/*, WriteFormatted*/};
    let mut buf = Buffer::default();
    let n = ts.time.round() as i64;
    buf.write_formatted(&n, &Locale::en);
    format!("{} {}", buf.as_str().to_string(), ts.unit)
}

