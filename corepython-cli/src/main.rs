#[macro_use]
extern crate log;

use corepython::python_to_wasm;

fn main() {
    simple_logger::init().unwrap();

    let filename = "demo.py";
    info!("Reading {}", filename);
    let source = std::fs::read_to_string(filename).unwrap();

    let output_filename = "demo.wasm";
    info!("Writing WebAssembly to {}", output_filename);
    let mut file = std::fs::File::create(output_filename).unwrap();

    python_to_wasm(&source, &mut file);
}
