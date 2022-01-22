use std::ffi::{CStr, CString};
use std::os::raw;
use crate::extract_json::extract_json;

use crate::raw::vosk_set_log_level;
use crate::raw::vosk_model_new;
use crate::raw::vosk_recognizer_new;
use crate::raw::vosk_recognizer_new_grm;
use crate::raw::vosk_recognizer_accept_waveform_s;
use crate::raw::vosk_recognizer_partial_result;
use crate::raw::vosk_recognizer_final_result;
use crate::raw::VoskModel;
use crate::raw::VoskRecognizer;


pub struct Model {
    inner: ModelInner,
}

pub struct Recognizer {
    ptr: *mut VoskRecognizer,
}

/// use -1 for no logs, 0 for default, 1 for verbose
pub fn set_log_level(level: raw::c_int) {
    unsafe { vosk_set_log_level(level) }
}

struct ModelInner {
    ptr: *mut VoskModel,
}

impl Model {
    /// provide a path to an existing model on disk. will error if the
    /// path doesnt exist or doesnt point to a valid model
    pub fn new(path: &str) -> Option<Model> {
        let path = CString::new(path).ok()?;
        let model = unsafe { vosk_model_new(path.as_ptr()) };
        if model.is_null() {
            return None;
        }
        let inner = ModelInner { ptr: model };
        Some(Model { inner })
    }

    fn ptr(&self) -> *mut VoskModel {
        self.inner.ptr
    }
}

impl Recognizer {
    /// use `16000 as f32` for sample rate
    pub fn new(model: &Model, sample_rate: f32) -> Recognizer {
        let recognizer = unsafe { vosk_recognizer_new(model.ptr(), sample_rate) };
        Recognizer { ptr: recognizer }
    }

    /// can optionally provide a `grammar` as an array of string phrases.
    /// can be either single words, or phrases.
    /// eg, grammar can be: `&["hello world", "a", "b", "c"]`
    pub fn with_grammar(model: &Model, sample_rate: f32, grammar: &[&str]) -> Recognizer {
        let mut json_grammar: String = "[".into();
        for st in grammar {
            json_grammar.push('"');
            json_grammar.push_str(st);
            json_grammar.push_str("\",");
        }
        // remove trailing comma
        json_grammar.pop();
        json_grammar.push(']');
        let cstr = CString::new(json_grammar).unwrap();
        let recognizer = unsafe {
            vosk_recognizer_new_grm(model.ptr(), sample_rate, cstr.as_ptr())
        };
        Recognizer { ptr: recognizer }
    }

    /// enter more data for vosk to process.
    /// vosk will apped this to its internal buffer and keep track
    /// of it internally. ie: the buffer you provide should be a
    /// continuation of the last buffer you provided. this is
    /// effectively a low level way of streaming in data.
    /// remember that the audio being sent in must be single channel (mono)
    /// 16 bit samples.
    /// returns true if vosk has detected silence and is ready to report
    /// its final guess of the speech entered
    pub fn accept_waveform(&mut self, buf: &[i16]) -> bool {
        let completed = unsafe {
            vosk_recognizer_accept_waveform_s(self.ptr, buf.as_ptr(), buf.len() as i32)
        };
        completed != 0
    }

    /// call this if `accept_waveform` returned false
    /// it will give you a partial result of what vosk thinks
    /// the speech is at the current time. This is useful for instant feedback
    /// but otherwise, if you want to wait until the response is most "correct",
    /// then keep calling accept_waveform unil it returns true,
    /// and then use `final_result instead
    pub fn partial_result<'a>(&'a mut self) -> &'a str {
        let c_str = unsafe {
            let ptr = vosk_recognizer_partial_result(self.ptr);
            CStr::from_ptr(ptr)
        };
        let s = c_str.to_str().expect("failed to convert cstr to string");
        extract_json(s, "partial").unwrap_or("JSONERR")
    }

    /// call this if `accept_waveform` returned true.
    /// Note: `vosk-api` also has
    /// `result()`, but I am choosing to only include `final_result` because
    /// it does some extra cleanup
    pub fn final_result<'a>(&'a mut self) -> &'a str {
        let c_str = unsafe {
            let ptr = vosk_recognizer_final_result(self.ptr);
            CStr::from_ptr(ptr)
        };
        let s = c_str.to_str().expect("failed to convert cstr to string");
        extract_json(s, "text").unwrap_or("JSONERR")
    }
}