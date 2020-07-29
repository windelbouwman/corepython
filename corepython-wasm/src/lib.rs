//! Library which can be used in the browser to turn python
//! into webassembly.

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn python_to_wasm(source: &str) -> Vec<u8> {
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Compiling python to WebAssembly");
    let mut output: Vec<u8> = vec![];
    if let Err(err) = corepython::python_to_wasm(source, &mut output) {
        let prefix = match err.location {
            Some(location) => format!("{}: ", location.get_text_for_user()),
            None => "".to_owned(),
        };
        log::error!("{}{}", prefix, err.message);
    }
    output
}
