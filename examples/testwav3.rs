use voskrust::api::{Model, Recognizer, set_log_level};
use voskrust::sound::*;

fn main() {
    set_log_level(-1);
    let model = Model::new("model").unwrap();
    let recognizer = Recognizer::new(&model, 16000 as f32);
    // can optionally use a predetermined vocab:
    let mut recognizer = Recognizer::with_grammar(
        &model,
        16000 as f32,
        &["hello", "world"],
    );
    let mut audioreader = voskrust::sound::ParecStream::init().unwrap();
    loop {
        let buf = audioreader.read_n_milliseconds(100.0).unwrap();
        let completed = recognizer.accept_waveform(&buf[..]);
        if completed {
            let result = recognizer.final_result();
            println!("Result: {:?}", result);
            // break;
        } else {
            let result = recognizer.partial_result();
            println!("Partial: {:?}", result);
        }
    }
}
