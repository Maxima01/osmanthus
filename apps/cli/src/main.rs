use anyhow::{Result, bail};
use osmanthus_application::{Backend, default_database, sample_bundle};
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("Osmanthus: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" {
        println!(
            "Osmanthus preliminary build\n\n  demo                  Replay the labeled synthetic recording\n  replay <bundle.json>  Validate, store, and analyze a recording\n  inspect <file>        Read-only SHA-256, PE imports, embedded URLs\n  history               List locally saved recordings and inspections\n\nData: ./data/evidence.db (override with OSMANTHUS_DATA_DIR)\nNo live monitoring, execution, or containment is implemented."
        );
        return Ok(());
    }
    let mut backend = Backend::open(&default_database())?;
    match args.as_slice() {
        [command] if command == "demo" => {
            let bundle = sample_bundle()?;
            println!("SYNTHETIC SAMPLE - no host activity was collected");
            println!(
                "{}",
                serde_json::to_string_pretty(&backend.replay(&bundle)?)?
            );
        }
        [command, path] if command == "replay" => {
            let (bundle, analysis) = backend.import(Path::new(path))?;
            println!(
                "Recording source: {:?} (unverified import)",
                bundle.session.source
            );
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        [command, path] if command == "inspect" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&backend.inspect(Path::new(path))?)?
            );
        }
        [command] if command == "history" => {
            for item in backend.history()? {
                println!("{}\t{}\t{}", item.kind, item.id, item.label);
            }
        }
        _ => bail!("Unknown command or wrong arguments; use --help"),
    }
    Ok(())
}
