use std::{env, fs, path::PathBuf};

use neuro_engine::{GateFExperimentConfig, run_gate_f_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let development = args.iter().any(|arg| arg == "--development");
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = GateFExperimentConfig::default();
    if development {
        config.seed = 0x4741_5445_5f46_0101;
        config.model_seed_count = 4;
        config.pretraining_episodes = 800;
        config.adaptation_episodes = 400;
        config.evaluation_episodes = 100;
        config.checkpoints = [0, 50, 100, 200, 400];
    }
    if quick {
        config.model_seed_count = 2;
        config.pretraining_episodes = 80;
        config.adaptation_episodes = 40;
        config.evaluation_episodes = 20;
        config.checkpoints = [0, 5, 10, 20, 40];
    }
    if let Some(value) =
        argument(&args, "--internal-learning-rate").and_then(|value| value.parse().ok())
    {
        config.internal_learning_rate = value;
    }
    if let Some(value) = argument(&args, "--output") {
        let output = PathBuf::from(value);
        write_result(config, output, development || quick);
    } else {
        let output = if development || quick {
            PathBuf::from("app/public/gate-f-development.json")
        } else {
            PathBuf::from("app/public/gate-f-v1.6.json")
        };
        write_result(config, output, development || quick);
    }
}

fn write_result(config: GateFExperimentConfig, output: PathBuf, non_frozen: bool) {
    let result = run_gate_f_experiment(config).expect("Gate F experiment");
    for report in &result.reports {
        if report.checkpoint_episode == 0 || report.checkpoint_episode == config.adaptation_episodes
        {
            println!(
                "phase={:?} checkpoint={} controller={:?} accuracy={:.3} ci=[{:.3}, {:.3}] drift={:.4}",
                report.phase,
                report.checkpoint_episode,
                report.controller,
                report.metrics.accuracy.mean,
                report.metrics.accuracy.lower95,
                report.metrics.accuracy.upper95,
                report.weights.mean_relative_group_norm_drift.mean,
            );
        }
    }
    for effect in &result.paired_effects {
        println!(
            "{} effect={:.3} ci=[{:.3}, {:.3}]",
            effect.id,
            effect.effect.accuracy.mean,
            effect.effect.accuracy.lower95,
            effect.effect.accuracy.upper95,
        );
    }
    println!("adaptation={:?}", result.adaptation);
    println!("acceptance={:?}", result.acceptance);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create Gate F output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Gate F result"),
    )
    .expect("write Gate F result");
    println!("wrote {}", output.display());
    if !non_frozen && !result.acceptance.passed {
        std::process::exit(2);
    }
}

fn argument<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
