use std::path::Path;

use firmament_hyg::hyg::{self, Skipped, Star};

fn select(max_mag: f64) -> (Vec<Star>, Skipped) {
    let rows =
        hyg::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.csv")).unwrap();
    hyg::select(&rows, max_mag)
}

#[test]
fn skips_by_reason() {
    let (stars, skipped) = select(10.0);
    assert_eq!(stars.len(), 6);
    assert_eq!(
        skipped,
        Skipped {
            sun: 1,
            no_coords: 1,
            no_mag: 1,
            faint: 1
        }
    );
}

#[test]
fn brightest_first() {
    let (stars, _) = select(10.0);
    let ids: Vec<u32> = stars.iter().map(|s| s.id).collect();
    assert_eq!(ids, [32263, 90979, 27919, 11734, 103879, 900000]);
}

#[test]
fn sirius_in_radians() {
    let (stars, _) = select(10.0);
    assert!((stars[0].ra - 1.767_79).abs() < 1e-5);
    assert!((stars[0].dec - -0.291_75).abs() < 1e-5);
}

#[test]
fn missing_color_is_nan() {
    let (stars, _) = select(10.0);
    assert!(stars.iter().find(|s| s.id == 900000).unwrap().bv.is_nan());
}

#[test]
fn cutoff_is_inclusive() {
    let (stars, skipped) = select(0.45);
    assert_eq!(stars.len(), 3);
    assert_eq!(skipped.faint, 4);
}
