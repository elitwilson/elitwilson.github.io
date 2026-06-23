use super::*;

// --- determinism ---

#[test]
fn same_seed_produces_same_u32_sequence() {
    let mut a = Rng::new(42);
    let mut b = Rng::new(42);
    let seq_a: Vec<u32> = (0..10).map(|_| a.next_u32()).collect();
    let seq_b: Vec<u32> = (0..10).map(|_| b.next_u32()).collect();
    assert_eq!(seq_a, seq_b, "same seed must yield identical sequences");
}

#[test]
fn different_seeds_produce_different_sequences() {
    let mut a = Rng::new(1);
    let mut b = Rng::new(2);
    let seq_a: Vec<u32> = (0..10).map(|_| a.next_u32()).collect();
    let seq_b: Vec<u32> = (0..10).map(|_| b.next_u32()).collect();
    assert_ne!(seq_a, seq_b, "different seeds must diverge");
}

#[test]
fn zero_seed_does_not_panic_or_return_all_zeros() {
    // xorshift32 state must not be 0; Rng::new(0) must handle this.
    let mut rng = Rng::new(0);
    let v = rng.next_u32();
    assert_ne!(v, 0, "zero seed must not produce a stuck-at-zero stream");
}

// --- next_f32 bounds ---

#[test]
fn next_f32_stays_in_zero_to_one_exclusive() {
    let mut rng = Rng::new(12345);
    for _ in 0..10_000 {
        let v = rng.next_f32();
        assert!(v >= 0.0 && v < 1.0, "next_f32 out of [0,1): {v}");
    }
}

// --- range_f32 bounds ---

#[test]
fn range_f32_stays_within_min_max() {
    let mut rng = Rng::new(99);
    let (min, max) = (2.5_f32, 7.3_f32);
    for _ in 0..10_000 {
        let v = rng.range_f32(min, max);
        assert!(
            v >= min && v < max,
            "range_f32({min}, {max}) out of range: {v}"
        );
    }
}

#[test]
fn range_f32_negative_bounds() {
    let mut rng = Rng::new(77);
    let (min, max) = (-5.0_f32, -1.0_f32);
    for _ in 0..1_000 {
        let v = rng.range_f32(min, max);
        assert!(
            v >= min && v < max,
            "range_f32({min}, {max}) out of range: {v}"
        );
    }
}
