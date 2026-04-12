#![allow(dead_code)]

pub fn start_profile(_name: &str) {}
pub fn end_profile(_name: &str) {}

pub struct OmniProfiler;
impl OmniProfiler {
    pub fn new() -> Self {
        Self
    }
    pub fn start(&self) {}
    pub fn stop(&self) {}
    pub fn report(&self) -> Option<String> {
        None
    }
    pub fn end(&self) {}
}

pub struct RuntimeProfiler;
impl RuntimeProfiler {
    pub fn start_profiling() {}
    pub fn stop_profiling() -> String {
        String::new()
    }
}
