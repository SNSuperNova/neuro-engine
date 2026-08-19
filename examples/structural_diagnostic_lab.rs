use std::{env, fs, path::PathBuf};

use neuro_engine::{StructuralDiagnosticConfig, run_structural_diagnostic};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = StructuralDiagnosticConfig::default();
    if quick {
        config.m2c_protocol.m1_protocol.development_seed_count = 1;
        config.m2c_protocol.m1_protocol.confirmation_seed_count = 2;
        config.forward_training_trials = 8;
        config.evaluation_trial_count = 16;
        config.shuffled_ranking_count = 4;
        config
            .m2c_protocol
            .m1_protocol
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .map1_protocol
            .probe_protocol
            .pretraining_episodes = 80;
    }
    let output = argument(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if quick {
                PathBuf::from("app/public/structural-diagnostic-development.json")
            } else {
                PathBuf::from("app/public/structural-diagnostic-v0.4.json")
            }
        });
    let result = run_structural_diagnostic(config).expect("structural diagnostic");
    for conclusion in &result.conclusions {
        println!("{conclusion}");
    }
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create diagnostic output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize structural diagnostic"),
    )
    .expect("write structural diagnostic");
    println!("wrote {}", output.display());
    if !result.acceptance.passed {
        std::process::exit(2);
    }
}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
