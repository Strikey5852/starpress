use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HygRow {
    pub id: u32,
    pub hip: Option<u32>,
    pub proper: Option<String>,
    pub ra: Option<f64>,
    pub dec: Option<f64>,
    pub mag: Option<f64>,
    pub ci: Option<f64>,
    pub bayer: Option<String>,
    pub flam: Option<u32>,
    pub con: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Star {
    pub id: u32,
    pub ra: f64,
    pub dec: f64,
    pub mag: f64,
    pub bv: f64,
    pub hip: Option<u32>,
    pub proper: Option<String>,
    pub designation: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Skipped {
    pub sun: usize,
    pub no_coords: usize,
    pub no_mag: usize,
    pub faint: usize,
}

pub fn read(path: &Path) -> Result<Vec<HygRow>> {
    csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .with_context(|| format!("opening {}", path.display()))?
        .into_deserialize()
        .map(|r| r.context("bad HYG row"))
        .collect()
}

pub fn select(rows: &[HygRow], max_mag: f64) -> (Vec<Star>, Skipped) {
    let mut stars = Vec::new();
    let mut skipped = Skipped::default();

    for row in rows {
        if row.id == 0 {
            skipped.sun += 1;
            continue;
        }
        let (Some(ra), Some(dec)) = (row.ra, row.dec) else {
            skipped.no_coords += 1;
            continue;
        };
        let Some(mag) = row.mag else {
            skipped.no_mag += 1;
            continue;
        };
        if mag > max_mag {
            skipped.faint += 1;
            continue;
        }
        stars.push(Star {
            id: row.id,
            // HYG gives RA in hours (15 degrees each), Dec in degrees.
            ra: (ra * 15.0).to_radians(),
            dec: dec.to_radians(),
            mag,
            bv: row.ci.unwrap_or(f64::NAN),
            hip: row.hip,
            proper: row.proper.clone(),
            designation: designation(row),
        });
    }

    stars.sort_by(|a, b| a.mag.total_cmp(&b.mag).then(a.id.cmp(&b.id)));
    (stars, skipped)
}

fn designation(row: &HygRow) -> Option<String> {
    let con = row.con.as_deref()?;
    match (&row.bayer, row.flam) {
        (Some(bayer), _) => Some(format!("{bayer} {con}")),
        (None, Some(flam)) => Some(format!("{flam} {con}")),
        (None, None) => None,
    }
}
