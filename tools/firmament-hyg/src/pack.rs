use std::io::{self, Write};

use crate::hyg::Star;

// stars.bin, little-endian, brightest first:
//   header (16 B)  "FMST" | u16 version | u16 record size | u32 count | u32 reserved
//   record (16 B)  f32 ra | f32 dec | f32 mag | f32 bv
// ra/dec are J2000 radians; bv is NaN when the catalog has no color index.
pub const VERSION: u16 = 1;
pub const RECORD_SIZE: u16 = 16;

pub fn write_stars(w: &mut impl Write, stars: &[Star]) -> io::Result<()> {
    w.write_all(b"FMST")?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&RECORD_SIZE.to_le_bytes())?;
    w.write_all(&(stars.len() as u32).to_le_bytes())?;
    w.write_all(&0u32.to_le_bytes())?;
    for s in stars {
        for v in [s.ra, s.dec, s.mag, s.bv] {
            w.write_all(&(v as f32).to_le_bytes())?;
        }
    }
    Ok(())
}

// names.bin, little-endian, only stars with a proper name or designation:
//   header (12 B)  "FMNM" | u16 version | u16 reserved | u32 count
//   entry          u32 star index | str proper | str designation
//   str            u16 byte length | UTF-8 bytes (length 0 = none)
// Star index points into stars.bin. Designation is Bayer or Flamsteed, e.g. "Alp CMa", "61 Cyg".
pub fn write_names(w: &mut impl Write, stars: &[Star]) -> io::Result<()> {
    let named: Vec<_> = stars
        .iter()
        .enumerate()
        .filter(|(_, s)| s.proper.is_some() || s.designation.is_some())
        .collect();

    w.write_all(b"FMNM")?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&(named.len() as u32).to_le_bytes())?;
    for (i, s) in named {
        w.write_all(&(i as u32).to_le_bytes())?;
        write_str(w, s.proper.as_deref())?;
        write_str(w, s.designation.as_deref())?;
    }
    Ok(())
}

// hip.bin, little-endian, one entry per star in stars.bin order:
//   header (12 B)  "FMHP" | u16 version | u16 reserved | u32 count
//   entry  (4 B)   u32 hip (0 = none)
pub fn write_hip(w: &mut impl Write, stars: &[Star]) -> io::Result<()> {
    w.write_all(b"FMHP")?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&0u16.to_le_bytes())?;
    w.write_all(&(stars.len() as u32).to_le_bytes())?;
    for s in stars {
        w.write_all(&s.hip.unwrap_or(0).to_le_bytes())?;
    }
    Ok(())
}

fn write_str(w: &mut impl Write, s: Option<&str>) -> io::Result<()> {
    let bytes = s.unwrap_or("").as_bytes();
    let len = u16::try_from(bytes.len()).map_err(|_| io::Error::other("name too long"))?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(bytes)
}
