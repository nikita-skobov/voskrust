use voskrust::raw::*;

fn main() {
    let level: std::os::raw::c_int = 1;
    unsafe {
        vosk_set_log_level(level);
    }
}
