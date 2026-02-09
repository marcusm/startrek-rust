pub struct Args {
    pub seed: Option<u64>,
}

pub fn parse() -> Args {
    let mut args = Args { seed: None };
    let mut iter = std::env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--seed" | "-s" => {
                if let Some(val) = iter.next() {
                    args.seed = Some(
                        val.parse::<u64>()
                            .expect("seed must be a valid integer"),
                    );
                } else {
                    eprintln!("Error: --seed requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Usage: startrek [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -s, --seed <INT>  Seed for the random number generator");
                println!("  -h, --help        Print help");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
    }

    args
}
