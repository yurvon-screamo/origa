//! JFT-Basic Can-do tracking (issue #760): the Irodori lesson → Can-do
//! mapping and per-band coverage computation.
//!
//! The mapping data lives on the CDN (`well_known_set/irodori_cando.json`),
//! generated from the official JF correspondence list; the catalog here is a
//! parsed, read-only view of it. Band semantics follow the Irodori course
//! progression against the JFT-Basic scoring bands: Nyuumon lessons → A1,
//! Shokyuu 1 → A2.1, Shokyuu 2 → A2 (test pass = 200/250 points, the A2
//! band). Coverage is import-based: a Can-do is covered when its single
//! owning lesson is imported, which keeps the SRS machinery untouched.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::domain::OrigaError;

/// JFT-Basic scoring band, aligned with the Irodori course progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JftBand {
    A1,
    #[serde(rename = "A2.1")]
    A21,
    A2,
}

impl JftBand {
    /// All bands in display (learning-path) order.
    pub const ALL: [JftBand; 3] = [Self::A1, Self::A21, Self::A2];

    pub fn code(&self) -> &'static str {
        match self {
            JftBand::A1 => "A1",
            JftBand::A21 => "A2.1",
            JftBand::A2 => "A2",
        }
    }
}

/// Official per-Can-do level tag from the JF framework list (A1: 158 items,
/// A2: 223). Not used by the v1 UI — its consumer is parse-time validation
/// of the CDN data contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JftCandoLevel {
    A1,
    A2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandoEntry {
    pub id: u32,
    pub level: JftCandoLevel,
    pub band: JftBand,
    pub lesson: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CatalogDocument {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    can_dos: Vec<CandoEntry>,
}

/// Parsed, read-only view of the CDN mapping.
#[derive(Debug, Clone)]
pub struct JftCandoCatalog {
    entries: Vec<CandoEntry>,
}

impl JftCandoCatalog {
    pub fn parse(json: &str) -> Result<Self, OrigaError> {
        let document: CatalogDocument = serde_json::from_str(json).map_err(|e| {
            OrigaError::WellKnownSetParseError {
                reason: format!("irodori_cando.json parse failed: {e}"),
            }
        })?;
        if document.can_dos.is_empty() {
            return Err(OrigaError::WellKnownSetParseError {
                reason: "irodori_cando.json has no can_dos".to_string(),
            });
        }
        Ok(Self {
            entries: document.can_dos,
        })
    }

    /// Per-band coverage over the given imported lesson ids. Bands missing
    /// from the data are reported as zero-coverage with total 0.
    pub fn coverage(&self, imported_lessons: &HashSet<String>) -> JftCoverage {
        let mut covered_per_band: HashMap<JftBand, usize> = HashMap::new();
        let mut total_per_band: HashMap<JftBand, usize> = HashMap::new();
        for entry in &self.entries {
            *total_per_band.entry(entry.band).or_default() += 1;
            if imported_lessons.contains(&entry.lesson) {
                *covered_per_band.entry(entry.band).or_default() += 1;
            }
        }

        let bands = JftBand::ALL
            .into_iter()
            .map(|band| BandCoverage {
                band,
                covered: covered_per_band.get(&band).copied().unwrap_or(0),
                total: total_per_band.get(&band).copied().unwrap_or(0),
            })
            .collect();

        JftCoverage { bands }
    }
}

/// Coverage across all bands, in [`JftBand::ALL`] order.
#[derive(Debug, Clone, PartialEq)]
pub struct JftCoverage {
    bands: Vec<BandCoverage>,
}

impl JftCoverage {
    pub fn bands(&self) -> &[BandCoverage] {
        &self.bands
    }

    /// Can-dos covered in any band; > 0 marks a user as an Irodori/JFT
    /// track learner (the home card's visibility gate).
    pub fn total_covered(&self) -> usize {
        self.bands.iter().map(|b| b.covered).sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BandCoverage {
    pub band: JftBand,
    pub covered: usize,
    pub total: usize,
}

impl BandCoverage {
    /// Covered fraction as percent. Zero when the band has no Can-dos
    /// (mirrors `CategoryProgress::percentage`), never NaN.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.covered as f64 / self.total as f64) * 100.0
    }
}

#[cfg(test)]
#[path = "jft_cando_tests.rs"]
mod tests;
