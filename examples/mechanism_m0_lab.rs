use std::{env, fs, path::PathBuf};

use neuro_engine::mechanism_m0_release;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let output = args
        .iter()
        .position(|arg| arg == "--output")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("app/public/mechanism-m0-v0.1.json"));
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create M0 output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&mechanism_m0_release()).expect("serialize M0 release"),
    )
    .expect("write M0 release");
    println!("wrote {}", output.display());
}
