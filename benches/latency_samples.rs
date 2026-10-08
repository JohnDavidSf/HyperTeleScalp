use std::{hint::black_box, time::Instant};

use hdrhistogram::Histogram;
use hypertele_scalp::{
    model::MessageClassification,
    parser::{SignalParser, SyntheticFixtureParser},
    risk::evaluate_dry_run,
    telegram::MessageView,
};

#[path = "../tests/support/mod.rs"]
mod support;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut histogram = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)?;
    let initial = Instant::now();
    let mut meta = support::signal(initial)?.meta;
    let mut market = support::market(initial)?;
    let limits = support::limits()?;
    let quantity = "1".parse()?;
    let mut accepted = 0;
    for index in 0..22_000 {
        let received = Instant::now();
        meta.received_at = received;
        market.received_at = received;
        let view = MessageView {
            meta: &meta,
            text: support::VALID,
            caption: None,
            has_media: false,
        };
        let MessageClassification::NewSignal(signal) =
            SyntheticFixtureParser.classify(black_box(&view))
        else {
            return Err("sampling fixture did not parse".into());
        };
        black_box(evaluate_dry_run(
            &signal, &market, quantity, &limits, received, received,
        ))?;
        let nanos = u64::try_from(received.elapsed().as_nanos())?;
        if index >= 2000 {
            histogram.record(nanos)?;
            accepted += 1;
        }
    }
    println!(
        "{{\"metric\":\"synthetic_parse_plus_simulation_risk_ns\",\"samples\":{accepted},\"warmup\":2000,\"p50\":{},\"p90\":{},\"p95\":{},\"p99\":{},\"p999\":{},\"max\":{}}}",
        histogram.value_at_quantile(0.5),
        histogram.value_at_quantile(0.9),
        histogram.value_at_quantile(0.95),
        histogram.value_at_quantile(0.99),
        histogram.value_at_quantile(0.999),
        histogram.max()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        panic!("latency sampling failed: {error}");
    }
}
