use crate::pca::PcaWeights;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::OnceLock;

static TEL_RE: OnceLock<Regex> = OnceLock::new();

pub struct TelemetryProcessor {
    pub pca: PcaWeights,
}

impl TelemetryProcessor {
    pub fn new(pca: PcaWeights) -> Self {
        Self { pca }
    }

    pub fn process_line(&mut self, line: &str) -> Option<Value> {
        let re = TEL_RE.get_or_init(|| {
            Regex::new(r"^\[TELEMETRY\] ts=(\d+) sym=([\w\.\-]+) margin=([0-9\.]+) conv=([0-9\.]+) eq=([0-9\.]+) eff=([0-9\.\-]+) den=([0-9\.\-]+) res=([0-9\.\-]+) comp=([0-9\.]+) range=([0-9\.]+) bias=([0-9\.\-]+)").unwrap()
        });

        let caps = re.captures(line)?;

        let ts: i64 = caps[1].parse().ok()?;
        let sym = caps[2].to_string();
        let margin: f64 = caps[3].parse().ok()?;
        let _conv: f64 = caps[4].parse().ok()?; // Currently unused downstream, but parsed for completeness
        let _eq: f64 = caps[5].parse().ok()?;
        let eff: f64 = caps[6].parse().ok()?;
        let den: f64 = caps[7].parse().ok()?;
        let res: f64 = caps[8].parse().ok()?;
        let comp: f64 = caps[9].parse().ok()?;
        let range: f64 = caps[10].parse().ok()?;
        let bias: f64 = caps[11].parse().ok()?;

        // Calculate synthetic execution properties (simplified mathematical truth)
        let entropy = (1.0 - den.abs()).max(0.0) * (if comp > 1.0 { 1.5 } else { 1.0 });
        let corridor = margin > 1.0 && entropy < 0.3;
        
        // Define instability mathematically
        let precursor_entropy_expansion = if comp > 1.5 { comp - 1.5 } else { 0.0 };
        let precursor_curvature_destabilization = (bias.abs() * 20.0) + (range * 100.0);
        
        let instability_type = if entropy > 0.8 {
            "ENTROPIC_COLLAPSE"
        } else if precursor_entropy_expansion > 0.5 {
            "EXPANSION_STRESS"
        } else if precursor_curvature_destabilization > 10.0 {
            "CURVATURE_FRACTURE"
        } else {
            "STABLE"
        };

        Some(json!({
            "ts": ts,
            "symbol": sym,
            "margin": margin,
            "directional_efficiency": eff,
            "continuation_density": den,
            "resilience": res,
            "compression_ratio": comp,
            "pre_range": range,
            "pre_bias": bias,
            "entropy": entropy,
            "corridor": corridor,
            "precursor_entropy_expansion": precursor_entropy_expansion,
            "precursor_curvature_destabilization": precursor_curvature_destabilization,
            "instability_type": instability_type
        }))
    }
}
