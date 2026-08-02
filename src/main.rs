use std::io::Read;

fn main() {
    let input = match std::env::args_os().nth(1) {
        Some(path) => std::fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("could not read input: {error}");
            std::process::exit(2);
        }),
        None => read_stdin(),
    };

    match brotaparser::compile_source(&input) {
        Ok(contract) => match brotaparser::serialize_json(&contract) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("could not serialize contract: {error}");
                std::process::exit(1);
            }
        },
        Err(diagnostics) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&diagnostics).expect("diagnostics are serializable")
            );
            std::process::exit(1);
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("could not read stdin: {error}");
        std::process::exit(2);
    }
    input
}
