//! Kind 1's original frequency/buckling wire layout. Runtime buckling has its own types;
//! keeping this private representation preserves the current encoding's kind-1 payload.
//! Neither the kind code nor any field order/presence byte changes. Array payloads are borrowed
//! on write and moved on read, without cloning a second set of mode shapes.

use fieldcraft_solver::{BucklingMode, BucklingResult, FrequencyResult, Mode, Stats, StepResult};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

use crate::codec::{self, CodecError};

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct LegacyFrequency {
    step: String,
    nodes: Vec<u32>,
    reference_points: Vec<(String, u32)>,
    modes: Vec<Mode>,
    total_mass: f64,
    stats: Stats,
    warnings: Vec<String>,
    buckling: Option<LegacySpectrum>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct LegacySpectrum {
    negative: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shift: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    below_shift: Option<usize>,
}

#[derive(Serialize)]
struct FrequencyRef<'a> {
    step: &'a str,
    nodes: &'a [u32],
    reference_points: &'a [(String, u32)],
    modes: Modes<'a>,
    total_mass: f64,
    stats: &'a Stats,
    warnings: &'a [String],
    buckling: Option<SpectrumRef<'a>>,
}

#[derive(Serialize)]
struct SpectrumRef<'a> {
    negative: &'a [f64],
    #[serde(skip_serializing_if = "Option::is_none")]
    shift: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    below_shift: Option<usize>,
}

#[derive(Serialize)]
struct ModeRef<'a> {
    eigenvalue: f64,
    frequency: f64,
    shape: &'a [[f64; 3]],
    rotation: &'a [[f64; 3]],
    participation: [f64; 3],
    effective_mass: [f64; 3],
    point_shape: &'a [[f64; 6]],
}

enum Modes<'a> {
    Frequency(&'a [Mode]),
    Buckling(&'a [BucklingMode]),
}

impl Serialize for Modes<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Frequency(modes) => modes.serialize(serializer),
            Self::Buckling(modes) => {
                let mut sequence = serializer.serialize_seq(Some(modes.len()))?;
                for m in *modes {
                    sequence.serialize_element(&ModeRef {
                        eigenvalue: m.load_factor,
                        frequency: 0.0,
                        shape: &m.shape,
                        rotation: &m.rotation,
                        participation: [0.0; 3],
                        effective_mass: [0.0; 3],
                        point_shape: &m.point_shape,
                    })?;
                }
                sequence.end()
            }
        }
    }
}

pub(super) fn encode_frequency(r: &FrequencyResult, out: &mut Vec<u8>) -> std::result::Result<(), CodecError> {
    codec::encode_into(
        &FrequencyRef {
            step: &r.step,
            nodes: &r.nodes,
            reference_points: &r.reference_points,
            modes: Modes::Frequency(&r.modes),
            total_mass: r.total_mass,
            stats: &r.stats,
            warnings: &r.warnings,
            buckling: None,
        },
        out,
    )
}

pub(super) fn encode_buckling(r: &BucklingResult, out: &mut Vec<u8>) -> std::result::Result<(), CodecError> {
    codec::encode_into(
        &FrequencyRef {
            step: &r.step,
            nodes: &r.nodes,
            reference_points: &r.reference_points,
            modes: Modes::Buckling(&r.modes),
            total_mass: 0.0,
            stats: &r.stats,
            warnings: &r.warnings,
            buckling: Some(SpectrumRef { negative: &r.negative, shift: r.shift, below_shift: r.below_shift }),
        },
        out,
    )
}

pub(super) fn decode(payload: &[u8]) -> std::result::Result<StepResult, CodecError> {
    let r: LegacyFrequency = codec::decode(payload)?;
    Ok(match r.buckling {
        None => StepResult::Frequency(FrequencyResult {
            step: r.step,
            nodes: r.nodes,
            reference_points: r.reference_points,
            modes: r.modes,
            total_mass: r.total_mass,
            stats: r.stats,
            warnings: r.warnings,
        }),
        Some(b) => {
            // Do not silently discard contradictory data in a damaged or hand-made result.
            // Original buckling writers always stored these frequency-only quantities as zero.
            if r.total_mass != 0.0 || r.modes.iter().any(|m| m.frequency != 0.0 || m.participation.iter().chain(&m.effective_mass).any(|v| *v != 0.0))
            {
                return Err(CodecError("a buckling result has nonzero or nonfinite frequency-only values".into()));
            }
            StepResult::Buckling(BucklingResult {
                step: r.step,
                nodes: r.nodes,
                reference_points: r.reference_points,
                modes: r
                    .modes
                    .into_iter()
                    .map(|m| BucklingMode { load_factor: m.eigenvalue, shape: m.shape, rotation: m.rotation, point_shape: m.point_shape })
                    .collect(),
                stats: r.stats,
                warnings: r.warnings,
                negative: b.negative,
                shift: b.shift,
                below_shift: b.below_shift,
            })
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Original small numeric records in the pre-#174 declaration order, including signed
    // zero and reference-point rotations. No external solver or native binary fixture.
    fn legacy(buckling: bool, shifted: bool) -> LegacyFrequency {
        LegacyFrequency {
            step: "Original".into(),
            nodes: vec![1, 2],
            reference_points: vec![("Fixture".into(), 9)],
            modes: vec![Mode {
                eigenvalue: 2.5,
                frequency: if buckling { 0.0 } else { 0.25 },
                shape: vec![[1.0, -0.0, 0.5], [-0.25, 0.0, 0.0]],
                rotation: vec![[0.0; 3]; 2],
                participation: if buckling { [0.0; 3] } else { [0.5, 0.0, -0.1] },
                effective_mass: if buckling { [0.0; 3] } else { [0.25, 0.0, 0.01] },
                point_shape: vec![[1.0, 0.0, 0.0, 0.1, -0.0, 0.2]],
            }],
            total_mass: if buckling { 0.0 } else { 8.0 },
            stats: Stats { dofs: 12, equations: 6, factor_entries: 21, lanczos_vectors: 6 },
            warnings: vec!["Original spectrum note".into()],
            buckling: buckling.then_some(LegacySpectrum {
                negative: if shifted { vec![] } else { vec![-4.0] },
                shift: shifted.then_some(1.0),
                below_shift: shifted.then_some(2),
            }),
        }
    }

    #[test]
    fn original_kind_one_records_keep_their_bytes_and_distinct_runtime_types() {
        for (buckling, shifted) in [(false, false), (true, false), (true, true)] {
            let old = legacy(buckling, shifted);
            let bytes = codec::encode(&old).unwrap();
            let result = decode(&bytes).unwrap();
            let mut rewritten = Vec::new();
            match &result {
                StepResult::Frequency(f) => {
                    assert!(!buckling);
                    assert_eq!(f.total_mass, old.total_mass);
                    assert_eq!(f.modes, old.modes);
                    encode_frequency(f, &mut rewritten).unwrap();
                }
                StepResult::Buckling(b) => {
                    assert!(buckling);
                    assert_eq!(b.modes[0].load_factor, old.modes[0].eigenvalue);
                    assert_eq!(b.modes[0].shape, old.modes[0].shape);
                    assert_eq!(b.modes[0].point_shape, old.modes[0].point_shape);
                    assert_eq!(b.shift, shifted.then_some(1.0));
                    assert_eq!(b.below_shift, shifted.then_some(2));
                    encode_buckling(b, &mut rewritten).unwrap();
                }
                _ => panic!("a modal result"),
            }
            assert_eq!(rewritten, bytes, "backward-compatible field order and float bits");
            let back: LegacyFrequency = codec::decode(&rewritten).unwrap();
            assert_eq!(back.modes, old.modes, "an old reader receives its unchanged shape/mass data");
            assert_eq!(back.buckling.is_some(), buckling);
            assert_eq!(back.stats, old.stats);
            assert_eq!(back.warnings, old.warnings);
        }
    }

    #[test]
    fn contradictory_or_damaged_legacy_records_are_not_silently_converted() {
        for field in 0..4 {
            let mut old = legacy(true, false);
            match field {
                0 => old.total_mass = 1.0,
                1 => old.modes[0].frequency = 1.0,
                2 => old.modes[0].participation[1] = f64::NAN,
                _ => old.modes[0].effective_mass[0] = 1.0,
            }
            assert!(decode(&codec::encode(&old).unwrap()).unwrap_err().to_string().contains("frequency-only"));
        }
        let bytes = codec::encode(&legacy(true, true)).unwrap();
        for len in 0..bytes.len() {
            assert!(decode(&bytes[..len]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode(&trailing).is_err());
    }

    #[test]
    fn buckling_vectors_and_spectrum_are_checked_before_display() {
        let StepResult::Buckling(original) = decode(&codec::encode(&legacy(true, true)).unwrap()).unwrap() else { panic!() };
        assert!(crate::results::check(&crate::results::Solved::Implicit(StepResult::Buckling(original.clone()))).is_ok());
        for bad in 0..9 {
            let mut b = original.clone();
            match bad {
                0 => {
                    b.modes[0].shape.pop();
                }
                1 => {
                    b.modes[0].rotation.pop();
                }
                2 => {
                    b.modes[0].point_shape.clear();
                }
                3 => b.modes[0].load_factor = f64::NAN,
                4 => b.modes[0].shape[0][0] = f64::INFINITY,
                5 => b.modes[0].point_shape[0][5] = f64::NAN,
                6 => b.shift = Some(f64::INFINITY),
                7 => b.negative.push(f64::NAN),
                _ => b.nodes.reverse(),
            }
            assert!(crate::results::check(&crate::results::Solved::Implicit(StepResult::Buckling(b))).is_err(), "case {bad}");
        }
    }
}
