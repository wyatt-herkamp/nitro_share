use std::path::PathBuf;

use clap::Parser;
use nitro_share::open_api;
use utoipa::OpenApi;
#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    output: Option<PathBuf>,
}
fn main() {
    let openapi = open_api::ApiDoc::openapi();

    let args: Args = Args::parse();

    if let Some(output) = args.output {
        if output.exists() {
            std::fs::remove_file(&output).unwrap();
        }
        let mut file = std::fs::File::create(output).unwrap();
        serde_json::to_writer_pretty(&mut file, &openapi).unwrap();
    } else {
        println!("{}", serde_json::to_string_pretty(&openapi).unwrap());
    }
}
