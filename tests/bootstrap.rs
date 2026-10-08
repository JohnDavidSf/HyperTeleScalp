mod support;

use std::{
    io::Cursor,
    time::{Duration, Instant},
};

use hypertele_scalp::{
    config::Config,
    execution::{BackendMode, ExecutionBackend, ExecutionError, ExecutionQueue, NoopBackend},
    model::{
        Decimal, DecimalError, Direction, EntryPolicy, MessageClassification, Price, PriceRange,
    },
    parser::{ChannelParser, SignalParser, SyntheticFixtureParser},
    recovery::{IntentError, IntentLedger},
    replay::{ReplayError, replay},
    risk::{Rejection, evaluate_dry_run},
    telegram::MessageView,
    telemetry::{LatencyEvent, LatencyStages, Stage, bounded_telemetry},
};
use proptest::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn chatter_and_incomplete_or_malformed_messages_are_ignored() {
    let meta = support::meta(Instant::now());
    for text in [
        "BTC to the moon 🚀",
        "watch ETH",
        "BTC LONG entry 100 target 110",
        "HTS_FIXTURE_V1|BTC|LONG|100",
        "HTS_FIXTURE_V1|BTC|LONG|bad|101|110|90",
        "HTS_FIXTURE_V1|BTC|LONG|100|101|110|90|EXTRA",
        "HTS_FIXTURE_V1|BTC|LONG|101|100|110|90",
        include_str!("fixtures/arbitrary-utf8.txt"),
    ] {
        let view = MessageView {
            meta: &meta,
            text,
            caption: None,
            has_media: false,
        };
        assert!(matches!(
            SyntheticFixtureParser.classify(&view),
            MessageClassification::Ignore(_)
        ));
    }
    // Real channel stubs must ignore even the complete artificial fixture.
    let view = MessageView {
        meta: &meta,
        text: support::VALID,
        caption: None,
        has_media: false,
    };
    assert!(matches!(
        ChannelParser { channel_id: -1001 }.classify(&view),
        MessageClassification::Ignore(_)
    ));
}

#[test]
fn complete_caption_is_parsed_without_media_work() {
    let meta = support::meta(Instant::now());
    let view = MessageView {
        meta: &meta,
        text: "hype",
        caption: Some(support::VALID),
        has_media: true,
    };
    assert!(matches!(
        SyntheticFixtureParser.classify(&view),
        MessageClassification::NewSignal(_)
    ));
}

#[test]
fn decimal_syntax_precision_and_tick_rules_are_exact() -> TestResult {
    for text in [
        "NaN",
        "inf",
        "1e3",
        "-1",
        "+1",
        "1,2",
        "1_000",
        "1.",
        ".1",
        "１",
        "1.0000000000001",
        "340282366920938463463374607431768211455",
    ] {
        assert!(text.parse::<Decimal>().is_err(), "accepted {text}");
    }
    assert_eq!("7.0600".parse::<Decimal>()?.to_string(), "7.06");
    assert_eq!("0.000000000001".parse::<Decimal>()?.atoms(), 1);
    assert!("0".parse::<Price>().is_err());
    assert_eq!("7.06".parse::<Price>()?.exact_ticks("0.01".parse()?)?, 706);
    assert_eq!(
        "7.061".parse::<Price>()?.exact_ticks("0.01".parse()?),
        Err(DecimalError::OffTick)
    );
    Ok(())
}

#[test]
fn duplicate_intents_and_parser_version_changes_are_rejected_without_eviction() -> TestResult {
    let mut id = support::signal(Instant::now())?.id;
    let mut ledger = IntentLedger::new(1);
    ledger.claim(id)?;
    assert_eq!(ledger.claim(id), Err(IntentError::Duplicate));
    id.template_version += 1;
    assert_eq!(ledger.claim(id), Err(IntentError::Duplicate));
    id.source.message_id += 1;
    assert_eq!(ledger.claim(id), Err(IntentError::Capacity));
    Ok(())
}

#[test]
fn stale_signal_queue_and_market_are_rejected() -> TestResult {
    let now = Instant::now();
    let mut signal = support::signal(now)?;
    let mut market = support::market(now)?;
    let limits = support::limits()?;
    let qty = "1".parse()?;
    signal.meta.source_age_at_receive = Some(Duration::from_secs(4));
    assert_eq!(
        evaluate_dry_run(&signal, &market, qty, &limits, now, now).err(),
        Some(Rejection::StaleSignal)
    );
    signal.meta.source_age_at_receive = Some(Duration::ZERO);
    let old = now
        .checked_sub(Duration::from_millis(300))
        .ok_or("clock range")?;
    assert_eq!(
        evaluate_dry_run(&signal, &market, qty, &limits, old, now).err(),
        Some(Rejection::StaleQueue)
    );
    market.received_at = old;
    assert_eq!(
        evaluate_dry_run(&signal, &market, qty, &limits, now, now).err(),
        Some(Rejection::StaleMarket)
    );
    signal.meta.source_age_at_receive = None;
    assert_eq!(
        evaluate_dry_run(&signal, &market, qty, &limits, now, now).err(),
        Some(Rejection::UnknownSourceAge)
    );
    Ok(())
}

#[test]
fn price_outside_entry_is_rejected_even_if_book_is_fresh() -> TestResult {
    let now = Instant::now();
    let signal = support::signal(now)?;
    let mut market = support::market(now)?;
    market.bid = Some("105".parse()?);
    market.ask = Some("105.01".parse()?);
    assert_eq!(
        evaluate_dry_run(
            &signal,
            &market,
            "1".parse()?,
            &support::limits()?,
            now,
            now
        )
        .err(),
        Some(Rejection::OutsideEntry)
    );
    Ok(())
}

#[test]
fn directionally_invalid_or_missing_protection_is_rejected() -> TestResult {
    let now = Instant::now();
    let market = support::market(now)?;
    let limits = support::limits()?;
    for (direction, stop, tp) in [
        (Direction::Long, Some("110"), "120"),
        (Direction::Long, Some("90"), "99"),
        (Direction::Short, Some("90"), "80"),
        (Direction::Short, Some("110"), "120"),
        (Direction::Long, None, "120"),
    ] {
        let mut signal = support::signal(now)?;
        signal.direction = direction;
        signal.stop_loss = stop.map(str::parse).transpose()?;
        signal.take_profits = vec![tp.parse()?];
        assert_eq!(
            evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now).err(),
            Some(Rejection::InvalidProtection)
        );
    }
    Ok(())
}

#[test]
fn missing_book_thin_depth_unsupported_symbol_and_spread_are_rejected() -> TestResult {
    let now = Instant::now();
    let signal = support::signal(now)?;
    let limits = support::limits()?;
    for (mut market, expected) in [
        (support::market(now)?, Rejection::InvalidBook),
        (support::market(now)?, Rejection::ThinBook),
        (support::market(now)?, Rejection::ExchangeNotReady),
        (support::market(now)?, Rejection::Spread),
    ] {
        match expected {
            Rejection::InvalidBook => market.ask = None,
            Rejection::ThinBook => market.ask_size = Some("0.1".parse()?),
            Rejection::ExchangeNotReady => market.supported_native_perp = false,
            Rejection::Spread => market.ask = Some("101".parse()?),
            _ => return Err("unexpected case".into()),
        }
        assert_eq!(
            evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now).err(),
            Some(expected)
        );
    }
    Ok(())
}

#[test]
fn caps_and_budgets_apply_at_the_worst_permitted_price() -> TestResult {
    let now = Instant::now();
    let signal = support::signal(now)?;
    let market = support::market(now)?;
    let mut limits = support::limits()?;
    let order = evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now)?;
    assert!(order.limit_price() >= market.ask.ok_or("ask")?);
    assert!(order.limit_price() <= "101".parse()?);
    limits.max_notional = "100.02".parse()?;
    assert_eq!(
        evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now).err(),
        Some(Rejection::Notional)
    );
    limits.max_notional = "1000".parse()?;
    limits.max_loss = "10.02".parse()?;
    assert_eq!(
        evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now).err(),
        Some(Rejection::Loss)
    );
    Ok(())
}

#[test]
fn short_ioc_cap_and_cmp_bound_are_directional() -> TestResult {
    let now = Instant::now();
    let mut signal = support::signal(now)?;
    signal.direction = Direction::Short;
    signal.entry = EntryPolicy::CurrentPrice {
        reference: "101".parse()?,
        lower: Some("99".parse()?),
        upper: None,
    };
    signal.take_profits = vec!["90".parse()?];
    signal.stop_loss = Some("110".parse()?);
    let market = support::market(now)?;
    let limits = support::limits()?;
    let order = evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now)?;
    assert!(order.limit_price() <= market.bid.ok_or("bid")?);
    assert!(order.limit_price() >= "99".parse()?);
    signal.entry = EntryPolicy::CurrentPrice {
        reference: "101".parse()?,
        lower: None,
        upper: None,
    };
    assert_eq!(
        evaluate_dry_run(&signal, &market, "1".parse()?, &limits, now, now).err(),
        Some(Rejection::OutsideEntry)
    );
    Ok(())
}

#[test]
fn backend_cannot_submit_live_or_reuse_or_execute_expired_intents() -> TestResult {
    let now = Instant::now();
    let order = evaluate_dry_run(
        &support::signal(now)?,
        &support::market(now)?,
        "1".parse()?,
        &support::limits()?,
        now,
        now,
    )?;
    for mode in [BackendMode::DryRun, BackendMode::Replay] {
        let mut backend = NoopBackend::new(mode, 8);
        assert!(!backend.can_submit_live());
        assert_eq!(backend.submit(&order, now)?.mode, mode);
        assert_eq!(
            backend.submit(&order, now),
            Err(ExecutionError::Intent(IntentError::Duplicate))
        );
        let late = now
            .checked_add(Duration::from_secs(1))
            .ok_or("clock range")?;
        assert_eq!(backend.submit(&order, late), Err(ExecutionError::Expired));
    }
    Ok(())
}

#[test]
fn configuration_rejects_live_testnet_unknown_fields_and_unsafe_values() -> TestResult {
    let example = include_str!("../config/config.example.toml");
    Config::from_toml(example)?;
    for text in [
        example.replace("trading_enabled = false", "trading_enabled = true"),
        example.replace("backend = \"dry_run\"", "backend = \"live\""),
        example.replace("backend = \"dry_run\"", "backend = \"testnet\""),
        example.replace("dedupe_capacity = 4096", "dedupe_capacity = 0"),
        example.replace("schema_version = 1", "schema_version = 2"),
        format!("typo = true\n{example}"),
    ] {
        assert!(Config::from_toml(&text).is_err());
    }
    Ok(())
}

#[test]
fn queue_and_telemetry_are_bounded_and_nonblocking() -> TestResult {
    let mut queue = ExecutionQueue::new(1);
    assert_eq!(queue.try_push(1), Ok(()));
    assert_eq!(queue.try_push(2), Err(2));
    assert_eq!(queue.pop(), Some(1));
    let (sink, mut collector) = bounded_telemetry(1)?;
    sink.record(LatencyEvent {
        stage: Stage::Transport,
        nanos: 100,
    });
    sink.record(LatencyEvent {
        stage: Stage::Transport,
        nanos: 200,
    });
    collector.drain()?;
    let summary = collector.summary(Stage::Transport);
    assert_eq!(summary.count, 1);
    assert_eq!(summary.dropped, 1);
    assert_eq!(summary.p99_ns, 100);
    Ok(())
}

#[test]
fn local_latency_uses_monotonic_stages_without_fabricating_network_events() -> TestResult {
    let now = Instant::now();
    let mut stages = LatencyStages::new(now, None);
    assert!(stages.event(Stage::Transport).is_none());
    assert!(stages.mark(Stage::Risk, now).is_err());
    stages.mark(Stage::Parse, now)?;
    stages.mark(Stage::Risk, now)?;
    stages.mark(Stage::Sign, now)?;
    let end = now
        .checked_add(Duration::from_micros(100))
        .ok_or("clock range")?;
    stages.mark(Stage::Transport, end)?;
    assert_eq!(
        stages.event(Stage::Transport).ok_or("missing event")?.nanos,
        100_000
    );
    assert!(stages.mark(Stage::Transport, end).is_err());
    assert_eq!(
        stages.between(Stage::Receive, Stage::Transport),
        Some(Duration::from_micros(100))
    );
    // Independent subscriptions can report the fill before the action acknowledgement.
    stages.mark(Stage::Fill, end)?;
    stages.mark(Stage::Protection, end)?;
    stages.mark(Stage::Ack, end)?;
    Ok(())
}

#[test]
fn multiple_entry_zones_reject_malformed_or_ambiguous_policies() -> TestResult {
    let valid = PriceRange {
        lower: "100".parse()?,
        upper: "101".parse()?,
    };
    let invalid = PriceRange {
        lower: valid.upper,
        upper: valid.lower,
    };
    let entry = "100.5".parse()?;
    assert_eq!(
        EntryPolicy::MultipleRanges(vec![valid, invalid]).allowed_range(entry, Direction::Long),
        None
    );
    assert_eq!(
        EntryPolicy::MultipleRanges(vec![valid, valid]).allowed_range(entry, Direction::Long),
        None
    );
    assert_eq!(
        EntryPolicy::MultipleRanges(vec![valid]).allowed_range(entry, Direction::Long),
        Some(valid)
    );
    Ok(())
}

#[test]
fn replay_classifies_bounded_records_and_never_executes() -> TestResult {
    let result = replay(
        Cursor::new(format!(
            "\n{}\n\n",
            include_str!("fixtures/synthetic.ndjson")
        )),
        &SyntheticFixtureParser,
    )?;
    assert_eq!(result.messages, 3);
    assert_eq!(result.signals, 1);
    assert_eq!(result.ignored, 2);
    assert!(matches!(
        replay(Cursor::new(vec![b'x'; 65_537]), &SyntheticFixtureParser),
        Err(ReplayError::Oversized)
    ));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn parsers_never_panic_on_arbitrary_utf8(text in any::<String>()) {
        let meta = support::meta(Instant::now());
        let view = MessageView { meta: &meta, text: &text, caption: None, has_media: false };
        let _ = SyntheticFixtureParser.classify(&view);
        let prefixed = format!("HTS_FIXTURE_V1|{text}");
        let _ = SyntheticFixtureParser.classify(&MessageView { meta: &meta, text: &prefixed, caption: None, has_media: false });
        prop_assert!(matches!(ChannelParser { channel_id: -1001 }.classify(&view), MessageClassification::Ignore(_)), "channel stub must ignore arbitrary UTF-8");
    }

    #[test]
    fn decimal_roundtrip_is_exact(whole in 0_u64..1_000_000_000, fraction in 0_u64..1_000_000_000_000) {
        let text = format!("{whole}.{fraction:012}");
        let parsed = text.parse::<Decimal>();
        prop_assert!(parsed.is_ok());
        if let Ok(value) = parsed {
            prop_assert_eq!(value.atoms(), u128::from(whole) * Decimal::SCALE + u128::from(fraction));
            prop_assert_eq!(value.to_string().parse::<Decimal>(), Ok(value));
        }
    }
}
