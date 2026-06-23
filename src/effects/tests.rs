use super::*;
use crate::particles::ParticleSystem;
use crate::rng::Rng;

// --- burst count ---

#[test]
fn spawn_fireworks_injects_exactly_params_count_particles() {
    let mut sys = ParticleSystem::new();
    let params = FireworksParams::default(); // count == 40
    let mut rng = Rng::new(1);
    spawn(
        EffectKind::Fireworks,
        (10.0, 10.0),
        &params,
        &mut rng,
        &mut sys,
    );
    assert_eq!(
        sys.particles().len(),
        params.count,
        "burst should inject exactly params.count particles"
    );
}

#[test]
fn spawn_fireworks_respects_custom_count() {
    let mut sys = ParticleSystem::new();
    let params = FireworksParams {
        count: 5,
        ..FireworksParams::default()
    };
    let mut rng = Rng::new(2);
    spawn(
        EffectKind::Fireworks,
        (5.0, 5.0),
        &params,
        &mut rng,
        &mut sys,
    );
    assert_eq!(sys.particles().len(), 5);
}

// --- velocity variation ---

#[test]
fn emitted_particles_have_varied_velocities() {
    let mut sys = ParticleSystem::new();
    let params = FireworksParams {
        count: 20,
        ..FireworksParams::default()
    };
    let mut rng = Rng::new(3);
    spawn(
        EffectKind::Fireworks,
        (0.0, 0.0),
        &params,
        &mut rng,
        &mut sys,
    );

    let particles = sys.particles();
    // At least two distinct velocity vectors must exist.
    let first = particles[0].vel;
    let all_same = particles
        .iter()
        .all(|p| (p.vel.0 - first.0).abs() < 1e-6 && (p.vel.1 - first.1).abs() < 1e-6);
    assert!(
        !all_same,
        "all particles had identical velocity — randomization is not applied"
    );
}

// --- seed reproducibility ---

#[test]
fn same_seed_same_origin_same_params_yields_identical_burst() {
    let make_burst = || {
        let mut sys = ParticleSystem::new();
        let params = FireworksParams::default();
        let mut rng = Rng::new(999);
        spawn(
            EffectKind::Fireworks,
            (8.0, 12.0),
            &params,
            &mut rng,
            &mut sys,
        );
        sys
    };

    let sys_a = make_burst();
    let sys_b = make_burst();

    let pa = sys_a.particles();
    let pb = sys_b.particles();
    assert_eq!(pa.len(), pb.len(), "burst sizes must match");

    for (i, (a, b)) in pa.iter().zip(pb.iter()).enumerate() {
        assert!(
            (a.vel.0 - b.vel.0).abs() < 1e-6 && (a.vel.1 - b.vel.1).abs() < 1e-6,
            "particle {i}: velocity mismatch between identical seeds"
        );
        let diff = a.remaining.as_secs_f32() - b.remaining.as_secs_f32();
        assert!(
            diff.abs() < 1e-6,
            "particle {i}: lifetime mismatch between identical seeds"
        );
    }
}

// --- origin placement ---

#[test]
fn emitted_particles_start_at_origin() {
    let mut sys = ParticleSystem::new();
    let params = FireworksParams {
        count: 10,
        ..FireworksParams::default()
    };
    let origin = (15.0_f32, 7.0_f32);
    let mut rng = Rng::new(55);
    spawn(EffectKind::Fireworks, origin, &params, &mut rng, &mut sys);

    for p in sys.particles() {
        assert!(
            (p.pos.0 - origin.0).abs() < 1e-6 && (p.pos.1 - origin.1).abs() < 1e-6,
            "particle does not start at origin: {:?}",
            p.pos
        );
    }
}
