use neuro_engine::{
    Phase2ExperimentConfig, Phase2ProtocolConfig, SimDuration, evaluate_phase2_input_stability,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seeds = [
        0x4e45_5552_4f32_0001,
        0x6c17_0b37_9b6f_61fe,
        0xb109_6f42_857d_6226,
    ];
    println!(
        "magnitude_mv,interval_ms,jitter_ms,pass_fraction,severe,mean_rate_hz,mean_active,mean_synchronous"
    );
    for magnitude_mv in [2.0, 4.0, 6.0, 8.0, 10.0] {
        for (interval_ms, jitter_ms) in [(30, 12), (50, 20), (80, 30)] {
            let config = Phase2ProtocolConfig {
                base: Phase2ExperimentConfig {
                    pattern_magnitude_mv: magnitude_mv,
                    pattern_interval: SimDuration::from_micros(interval_ms * 1_000),
                    ..Phase2ExperimentConfig::default()
                },
                input_jitter: SimDuration::from_micros(jitter_ms * 1_000),
                ..Phase2ProtocolConfig::default()
            };
            let evaluation = evaluate_phase2_input_stability(config, &seeds, 10)?;
            println!(
                "{:.1},{},{},{:.4},{},{:.4},{:.4},{:.4}",
                magnitude_mv,
                interval_ms,
                jitter_ms,
                evaluation.pass_fraction,
                evaluation.severe_instability_count,
                evaluation.mean_firing_rate_hz,
                evaluation.mean_active_fraction,
                evaluation.mean_synchronous_fraction,
            );
        }
    }
    Ok(())
}
