use std::{env, fs, path::PathBuf, time::Instant};

use neuro_engine::{GateDExperimentConfig, run_gate_d_experiment};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let development = args.iter().any(|arg| arg == "--development");
    let quick = args.iter().any(|arg| arg == "--quick");
    let mut config = GateDExperimentConfig::default();
    if development {
        config.seed = 0x4741_5445_5f44_0101;
        if let Some(radius) = args
            .iter()
            .position(|arg| arg == "--radius")
            .and_then(|index| args.get(index + 1))
            .and_then(|value| value.parse::<f64>().ok())
        {
            config.recurrent_spectral_radius = radius;
        }
    }
    if quick {
        config.model_seed_count = 2;
        config.training_episodes = 80;
        config.evaluation_episodes = 16;
        config.curve_window = 20;
    }
    let output = args
        .iter()
        .position(|arg| arg == "--output")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if development || quick {
                PathBuf::from("app/public/gate-d-development.json")
            } else {
                PathBuf::from("app/public/gate-d-v1.4.json")
            }
        });
    let started = Instant::now();
    let result = run_gate_d_experiment(config).expect("Gate D experiment");
    for report in &result.reports {
        let accuracy = report.metrics.correct_choice_fraction;
        println!(
            "delay={} controller={:?} accuracy={:.3} ci=[{:.3}, {:.3}] stable_abs={:.3} saturation={:.3} silent={:.3} synchrony={:.3}",
            report.delay_steps,
            report.controller,
            accuracy.mean,
            accuracy.lower95,
            accuracy.upper95,
            report.stability.mean_absolute_activity.mean,
            report.stability.saturated_unit_fraction.mean,
            report.stability.silent_unit_fraction.mean,
            report.stability.population_synchrony.mean,
        );
    }
    for effect in &result.paired_effects {
        let value = effect.effect.correct_choice_fraction;
        println!(
            "{} effect={:.3} ci=[{:.3}, {:.3}]",
            effect.id, value.mean, value.lower95, value.upper95
        );
    }
    println!("boundaries={:?}", result.reliable_memory_boundary_steps);
    println!("budgets={:?}", result.controller_budgets);
    println!("acceptance={:?}", result.acceptance);
    println!(
        "benchmark_elapsed={:?} (not part of deterministic report)",
        started.elapsed()
    );
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&result).expect("serialize Gate D result"),
    )
    .expect("write Gate D result");
    println!("wrote {}", output.display());
    if !development && !quick && !result.acceptance.passed {
        std::process::exit(2);
    }
}
