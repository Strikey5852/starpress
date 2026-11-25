use std::path::Path;

use firmament_hyg::{hyg, pack};

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes(b[i..i + 2].try_into().unwrap())
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}

fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}

#[test]
fn stars_round_trip() {
    let rows =
        hyg::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.csv")).unwrap();
    let (stars, _) = hyg::select(&rows, 10.0);

    let mut buf = Vec::new();
    pack::write_stars(&mut buf, &stars).unwrap();

    assert_eq!(buf.len(), 16 + 16 * stars.len());
    assert_eq!(&buf[0..4], b"FMST");
    assert_eq!(u16_at(&buf, 4), pack::VERSION);
    assert_eq!(u16_at(&buf, 6), pack::RECORD_SIZE);
    assert_eq!(u32_at(&buf, 8), stars.len() as u32);
    assert_eq!(u32_at(&buf, 12), 0);

    for (i, s) in stars.iter().enumerate() {
        let r = 16 + 16 * i;
        let got = [
            f32_at(&buf, r),
            f32_at(&buf, r + 4),
            f32_at(&buf, r + 8),
            f32_at(&buf, r + 12),
        ];
        let want = [s.ra as f32, s.dec as f32, s.mag as f32, s.bv as f32];
        for (g, w) in got.into_iter().zip(want) {
            assert!(
                g == w || (g.is_nan() && w.is_nan()),
                "star {i}: {got:?} != {want:?}"
            );
        }
    }
}

struct Entry {
    index: u32,
    proper: String,
    designation: String,
}

fn read_str(b: &[u8], pos: &mut usize) -> String {
    let len = u16_at(b, *pos) as usize;
    let s = std::str::from_utf8(&b[*pos + 2..*pos + 2 + len])
        .unwrap()
        .to_owned();
    *pos += 2 + len;
    s
}

fn read_names(b: &[u8]) -> Vec<Entry> {
    assert_eq!(&b[0..4], b"FMNM");
    assert_eq!(u16_at(b, 4), pack::VERSION);
    let count = u32_at(b, 8) as usize;
    let mut pos = 12;
    let mut entries = Vec::new();
    for _ in 0..count {
        let index = u32_at(b, pos);
        pos += 4;
        let proper = read_str(b, &mut pos);
        let designation = read_str(b, &mut pos);
        entries.push(Entry {
            index,
            proper,
            designation,
        });
    }
    assert_eq!(pos, b.len());
    entries
}

#[test]
fn names_round_trip() {
    let rows =
        hyg::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.csv")).unwrap();
    let (stars, _) = hyg::select(&rows, 10.0);

    let mut buf = Vec::new();
    pack::write_names(&mut buf, &stars).unwrap();
    let names = read_names(&buf);

    assert_eq!(names.len(), 5);
    let sirius = &names[0];
    assert_eq!(sirius.index, 0);
    assert_eq!(sirius.proper, "Sirius");
    assert_eq!(sirius.designation, "Alp CMa");

    let cyg = names.iter().find(|e| e.designation == "61 Cyg").unwrap();
    assert_eq!(stars[cyg.index as usize].id, 103879);
    assert_eq!(cyg.proper, "");

    assert!(names.iter().all(|e| stars[e.index as usize].id < 900000));
}

#[test]
fn hip_round_trip() {
    let rows =
        hyg::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.csv")).unwrap();
    let (stars, _) = hyg::select(&rows, 10.0);

    let mut buf = Vec::new();
    pack::write_hip(&mut buf, &stars).unwrap();

    assert_eq!(&buf[0..4], b"FMHP");
    assert_eq!(u16_at(&buf, 4), pack::VERSION);
    assert_eq!(u32_at(&buf, 8), stars.len() as u32);
    assert_eq!(buf.len(), 12 + 4 * stars.len());
    for (i, s) in stars.iter().enumerate() {
        assert_eq!(u32_at(&buf, 12 + 4 * i), s.hip.unwrap_or(0));
    }
    assert_eq!(u32_at(&buf, 12), 32349);
}
