use corepython::python_to_wasm;

fn main() {
    let matches = clap::App::new("corepython compiler")
        .version("0")
        .author("Windel Bouwman")
        .about("Compile python to webassembly")
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            clap::Arg::with_name("source")
                .required(true)
                .help("Source file to compile"),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("output file name"),
        )
        .get_matches();

    let log_level = match matches.occurrences_of("v") {
        0 => log::Level::Warn,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    let filename = std::path::Path::new(matches.value_of("source").unwrap());
    let default_output = filename.with_extension("wasm");
    let output_filename = matches
        .value_of("output")
        .map(std::path::Path::new)
        .unwrap_or_else(|| default_output.as_path());

    simple_logger::init_with_level(log_level).unwrap();

    log::info!("Reading {}", filename.to_string_lossy());
    let source = std::fs::read_to_string(filename).unwrap();

    log::info!(
        "Writing WebAssembly to {}",
        output_filename.to_string_lossy()
    );
    let mut file = std::fs::File::create(output_filename).unwrap();

    if let Err(err) = python_to_wasm(&source, &mut file) {
        let prefix = match err.location {
            Some(location) => format!(
                "{}:{}",
                filename.to_string_lossy(),
                location.get_text_for_user()
            ),
            None => format!("{}", filename.to_string_lossy()),
        };
        log::error!("{}: {}", prefix, err.message);
    }
}
