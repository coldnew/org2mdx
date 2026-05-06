use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "org2mdx")]
#[command(about = "Convert between Org and MDX", long_about = None)]
struct Cli {
    input: String,
    output: Option<String>,

    #[arg(long, default_value = "org")]
    from: String,

    #[arg(long, default_value = "mdx")]
    to: String,
}

fn main() {
    let cli = Cli::parse();
    let input_content = fs::read_to_string(&cli.input).expect("failed to read input file");

    let output_content = match (cli.from.as_str(), cli.to.as_str()) {
        ("org", "mdx") => org2mdx::org_to_mdx::convert(&input_content),
        ("mdx", "org") => org2mdx::mdx_to_org::convert(&input_content),
        ("org", "ast") => org2mdx::org_to_ast::parse(&input_content).and_then(|root| {
            serde_json::to_string_pretty(&root)
                .map_err(|e| org2mdx::Error::InvalidInput(e.to_string()))
        }),
        ("mdx", "ast") => org2mdx::mdx_to_ast::parse(&input_content).and_then(|root| {
            serde_json::to_string_pretty(&root)
                .map_err(|e| org2mdx::Error::InvalidInput(e.to_string()))
        }),
        (from, to) => {
            eprintln!("Unsupported conversion: {} -> {}", from, to);
            std::process::exit(1);
        }
    }
    .unwrap_or_else(|e| {
        eprintln!("Conversion failed: {}", e);
        std::process::exit(1);
    });

    if let Some(output_path) = cli.output {
        fs::write(output_path, output_content).expect("failed to write output file");
    } else {
        print!("{}", output_content);
    }
}
