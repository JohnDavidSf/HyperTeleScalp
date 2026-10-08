use std::{hint::black_box, time::Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use hypertele_scalp::{
    model::{EntryPolicy, MessageClassification},
    parser::{ChannelParser, SignalParser, SyntheticFixtureParser},
    risk::{evaluate_dry_run, validate_signal},
    telegram::MessageView,
};

#[path = "../tests/support/mod.rs"]
mod support;

fn pipeline(c: &mut Criterion) {
    let now = Instant::now();
    let meta = support::meta(now);
    let view = MessageView {
        meta: &meta,
        text: support::VALID,
        caption: None,
        has_media: false,
    };
    let (Ok(signal), Ok(market), Ok(mut limits), Ok(quantity)) = (
        support::signal(now),
        support::market(now),
        support::limits(),
        "1".parse(),
    ) else {
        panic!("benchmark fixture setup failed");
    };
    // Bench fixed input with a fixed evaluation time: no wall clock, network, or stale-data drift.
    limits.max_slippage_bps = 10;
    c.bench_function("text_classify_channel_stub", |b| {
        b.iter(|| ChannelParser { channel_id: -1001 }.classify(black_box(&view)))
    });
    c.bench_function("text_parse_synthetic", |b| {
        b.iter(|| SyntheticFixtureParser.classify(black_box(&view)))
    });
    let Some(entry) = market.ask else {
        panic!("benchmark fixture missing ask");
    };
    c.bench_function("signal_validation", |b| {
        b.iter(|| validate_signal(black_box(&signal), black_box(entry)))
    });
    if let (EntryPolicy::Range(range), Some(ask)) = (&signal.entry, market.ask) {
        c.bench_function("price_range_check", |b| {
            b.iter(|| black_box(range).contains(black_box(ask)))
        });
    }
    c.bench_function("risk_decision", |b| {
        b.iter(|| {
            evaluate_dry_run(
                black_box(&signal),
                black_box(&market),
                quantity,
                &limits,
                now,
                now,
            )
        })
    });
    c.bench_function("signal_to_prepared_order_synthetic", |b| {
        b.iter(|| match SyntheticFixtureParser.classify(black_box(&view)) {
            MessageClassification::NewSignal(parsed) => {
                evaluate_dry_run(&parsed, &market, quantity, &limits, now, now)
            }
            _ => Err(hypertele_scalp::risk::Rejection::InvalidIdentity),
        })
    });
}

criterion_group! { name = benches; config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(1)).measurement_time(std::time::Duration::from_secs(2)); targets = pipeline }
criterion_main!(benches);
