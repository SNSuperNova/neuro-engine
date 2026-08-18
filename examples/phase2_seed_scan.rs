use neuro_engine::{Phase2ProtocolConfig, Phase2StabilityCriteria, screen_phase2_structure_seeds};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limit = std::env::args()
        .nth(1)
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(Phase2ProtocolConfig::default().structure_seed_scan_limit);
    let config = Phase2ProtocolConfig {
        structure_seed_scan_limit: limit,
        ..Phase2ProtocolConfig::default()
    };
    let screens = screen_phase2_structure_seeds(config, Phase2StabilityCriteria::default())?;
    println!("seed,accepted,pass_fraction,rate_hz,active,synchronous,silence_ms,autocorrelation");
    for screen in screens {
        println!(
            "{:016x},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            screen.seed,
            screen.accepted,
            screen.pass_fraction,
            screen.metrics.mean_firing_rate_hz,
            screen.metrics.active_neuron_fraction,
            screen.metrics.synchronous_burst_bin_fraction,
            screen.metrics.maximum_silence_ms,
            screen.metrics.peak_autocorrelation,
        );
    }
    Ok(())
}
