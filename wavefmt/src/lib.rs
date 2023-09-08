use std::error;

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FieldType {
    Timestamp,
    Digital,
    DigiBus(usize),
    Analog,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NumType {
    Unknown,
    Float,
    UnsignedInteger,
    Integer,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub ftype: FieldType,
}

pub trait WaveFile {
    /// Check the wave file matches the expected format
    fn check_format(&mut self) -> Result<bool> { Ok(false) }

    /// Size (in bytes) of wave record
    fn get_record_size(&self) -> Option<usize>;

    /// Return time range covered by this waveform
    fn get_range(&self) -> (f64, f64);

    /// Return number of fields, requires check_format is called first to read file header
    fn get_num_fields(&self) -> usize;

    /// Return field details, requires check_format is called first to read file header
    fn get_field_info(&self, field: usize) -> &FieldInfo;

    /// Return number of rows (data points) if available, requires check_format is called first to read file header
    fn get_num_rows(&self) -> Option<usize>;

    /// Prepare to display a range of waveform data, return (start, end) record number for this
    /// time range
    fn prepare_iter_range(&mut self, range: &[f64; 2]) -> Result<[usize; 2]>;
}

