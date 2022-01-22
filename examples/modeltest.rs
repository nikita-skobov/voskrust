use voskrust::api::*;

fn main() {
    let result = Model::new("not_existing");
    assert!(result.is_none());
}
