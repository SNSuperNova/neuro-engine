use neuro_engine::{Gate2ExperimentConfig, run_gate2_experiment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inhibitory_out_degrees = [32];
    let inhibitory_weights = [13.0, 13.2, 13.4, 14.0];

    println!(
        "inh_degree\tinh_mv\tdriven_hz\tactive\tsync_bins\tmax_silence_ms\tperiodicity\twithdraw_hz\twithdraw_last_ms\ttrajectory"
    );
    for inhibitory_out_degree in inhibitory_out_degrees {
        for inhibitory_weight_mv in inhibitory_weights {
            let mut config = Gate2ExperimentConfig::default();
            config.network.inhibitory_out_degree = inhibitory_out_degree;
            config.network.inhibitory_weight_mv = inhibitory_weight_mv;
            let result = run_gate2_experiment(config)?;
            let summary = result.summary;
            println!(
                "{}\t{:.1}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}\t{:.5}",
                inhibitory_out_degree,
                inhibitory_weight_mv,
                summary.driven.mean_firing_rate_hz,
                summary.driven.active_neuron_fraction,
                summary.driven.synchronous_burst_bin_fraction,
                summary.driven.maximum_silence_ms,
                summary.driven.peak_autocorrelation,
                summary.withdrawal.mean_firing_rate_hz,
                summary
                    .withdrawal_last_spike_after_stop_ms
                    .map(|value| format!("{value:.3}"))
                    .unwrap_or_else(|| "none".to_owned()),
                summary.stimulus_trajectory.mean_rms_spike_difference,
            );
        }
    }
    Ok(())
}
