use std::path::Path;

use firmament_hyg::hyg::{self, HygRow};

fn rows() -> Vec<HygRow> {
    hyg::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.csv")).unwrap()
}

#[test]
fn reads_every_row() {
    assert_eq!(rows().len(), 10);
}

#[test]
fn parses_sirius() {
    let rows = rows();
    let sirius = rows.iter().find(|r| r.hip == Some(32349)).unwrap();
    assert_eq!(sirius.id, 32263);
    assert_eq!(sirius.proper.as_deref(), Some("Sirius"));
    assert_eq!(sirius.ra, Some(6.752481));
    assert_eq!(sirius.dec, Some(-16.716116));
    assert_eq!(sirius.mag, Some(-1.44));
    assert_eq!(sirius.ci, Some(0.009));
    assert_eq!(sirius.bayer.as_deref(), Some("Alp"));
    assert_eq!(sirius.flam, Some(9));
    assert_eq!(sirius.con.as_deref(), Some("CMa"));
}

#[test]
fn blank_fields_are_none() {
    let rows = rows();
    let faint = rows.iter().find(|r| r.id == 900000).unwrap();
    assert_eq!(faint.hip, None);
    assert_eq!(faint.ci, None);
    assert_eq!(faint.proper, None);
    assert_eq!(faint.bayer, None);
    assert_eq!(faint.flam, None);
    assert_eq!(rows.iter().find(|r| r.id == 900002).unwrap().dec, None);
}
