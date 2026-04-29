use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: org2mdx <input.org> [output.mdx]");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let input = fs::read_to_string(input_path).expect("failed to read input file");
    let output = match org2mdx::convert(&input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Conversion failed: {}", e);
            std::process::exit(1);
        }
    };

    if args.len() >= 3 {
        fs::write(&args[2], output).expect("failed to write output file");
    } else {
        print!("{}", output);
    }
}
