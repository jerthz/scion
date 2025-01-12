//! Utilities provided by `Scion` to help to do some basic stuff.
pub mod file;
pub mod logger;
pub mod maths;
pub mod frame_limiter;
pub mod premade;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ScionError {
    details: String
}

impl ScionError {
    pub fn new(msg: &str) -> Self {
        Self{details: msg.to_string()}
    }
}
