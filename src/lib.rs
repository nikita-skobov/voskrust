#[link(name = "vosk")]
extern {
    pub fn vosk_set_log_level(log_level: ::std::os::raw::c_int);
}
