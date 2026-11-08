//! Duplicate-spend regression cases and a focused allocation-path benchmark.

use std::{collections::HashSet, hint::black_box, time::Instant};

use zebra_chain::{transaction, transparent::OutPoint};

use super::{check_for_duplicates, TransactionError};

fn outpoint(index: u32) -> OutPoint {
    OutPoint {
        hash: transaction::Hash([0; 32]),
        index,
    }
}

#[test]
fn duplicate_spend_iterator_preserves_the_first_conflict() {
    for (indices, duplicate) in [
        (vec![], None),
        (vec![0], None),
        (vec![0, 1], None),
        (vec![0, 0], Some(0)),
        (vec![0, 1, 2, 1, 0], Some(1)),
    ] {
        let result = check_for_duplicates(
            indices.into_iter().map(outpoint),
            TransactionError::DuplicateTransparentSpend,
        );
        assert_eq!(
            result,
            duplicate.map_or(Ok(()), |index| {
                Err(TransactionError::DuplicateTransparentSpend(outpoint(index)))
            }),
        );
    }
}

#[test]
#[ignore = "indicative local duplicate-spend microbenchmark"]
fn benchmark_single_spend_duplicate_check() {
    const ITERATIONS: u32 = 1_000_000;
    let _init_guard = zebra_test::init();
    for legacy in [true, false] {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let items = [outpoint(0)];
            let items = black_box(items.as_slice());
            if legacy {
                // Previous spend_conflicts path: collect the domain, then hash each item.
                let items = items.to_vec();
                let mut seen = HashSet::new();
                for item in items {
                    assert!(seen.replace(item).is_none());
                }
                black_box(seen);
            } else {
                check_for_duplicates(
                    items.iter().copied(),
                    TransactionError::DuplicateTransparentSpend,
                )
                .unwrap();
            }
        }
        tracing::info!(
            "single-spend legacy={legacy}: {:.1} ns/check ({ITERATIONS} iterations)",
            started.elapsed().as_secs_f64() * 1e9 / f64::from(ITERATIONS),
        );
    }
}
