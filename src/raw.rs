//! Contains *a subset of* the raw bindings for `vosk_api`
//! If you want more vosk functionality supported, see:
//! 
//! https://github.com/alphacep/vosk-api/blob/master/src/vosk_api.h
//! and also:
//! 
//! https://github.com/wzhd/vosk-sys/blob/main/src/lib.rs

use std::os::raw;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoskModel {
    _dummy: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct VoskRecognizer {
    _unused: [u8; 0],
}

#[link(name = "vosk")]
extern {
    /// Loads model data from the path and returns the model object
    pub fn vosk_model_new(model_path: *const raw::c_char) -> *mut VoskModel;

    /// Set log level for Kaldi messages
    pub fn vosk_set_log_level(log_level: raw::c_int);

    /// creates the recognizer object. takes a model, and a sample rate (should be 16000 as f32)
    pub fn vosk_recognizer_new(model: *mut VoskModel, sample_rate: f32) -> *mut VoskRecognizer;

    /// Creates the recognizer object with the phrase list
    pub fn vosk_recognizer_new_grm(model: *mut VoskModel, sample_rate: f32, grammar: *const raw::c_char) -> *mut VoskRecognizer;

    /// Same as above but the version with the short data for language bindings where you have
    /// audio as array of shorts
    pub fn vosk_recognizer_accept_waveform_s(recognizer: *mut VoskRecognizer, data: *const raw::c_short, length: raw::c_int) -> raw::c_int;

    /// Returns partial speech recognition
    pub fn vosk_recognizer_partial_result(recognizer: *mut VoskRecognizer) -> *const raw::c_char;

    /// Returns speech recognition result.
    pub fn vosk_recognizer_final_result(recognizer: *mut VoskRecognizer) -> *const raw::c_char;
}
