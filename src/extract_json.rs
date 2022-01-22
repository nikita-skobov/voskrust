//! vosk api returns results as a json formatted string.
//! This string is well formatted, so using serde seems a bit overkill
//! instead we can parse it very simply using
//! `extract_json(source_json_str, "key")`


pub fn extract_json<'a>(
    source_json_str: &'a str,
    key: &str
) -> Option<&'a str> {
    // check if it at least looks like valid json
    if !source_json_str.starts_with('{') {
        return None;
    }
    // from vosk, we know the json will always start on
    // the 5th character
    let key_range = 5..(5 + key.len());
    let potential_key = source_json_str.get(key_range)?;
    if potential_key != key {
        // println!("KEY DOESNT MATCH!");
        return None;
    }

    // add 5 to the end for a quote char, a space,
    // colon, another space, and another quote.
    let value_start_index = 5 + key.len() + 5;
    let value_piece = source_json_str.get(value_start_index..)?;
    // we assume that the json does not contain any quotes:
    let (value, _) = value_piece.split_once('"')?;
    Some(value)
}
