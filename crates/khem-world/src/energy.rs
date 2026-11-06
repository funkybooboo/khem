//! Energy sources as declared world content: which vents, which
//! sunlight (the data a .kem world file declares in phase 4).
//! The system that APPLIES them (the Energy trait impl) lives in
//! khem-physics.

/// The three source kinds the world vocabulary allows (language
/// spec section 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Hydrothermal,
    SolarUv,
    Radiation,
}

/// One energy source (runtime spec 8.1, 8.2).
#[derive(Debug, Clone, Copy)]
pub struct EnergySource {
    pub kind: SourceKind,
    /// Position, angstroms; meaningful for point sources.
    pub position: (f32, f32, f32),
    /// Normalized intensity, 0.0 - 1.0.
    pub intensity: f32,
    /// Radius, angstroms; point sources.
    pub radius: f32,
    /// Solar UV only; defaults to true (language spec V-WORLD-08).
    pub surface_only: bool,
}

impl EnergySource {
    pub fn hydrothermal(position: (f32, f32, f32), intensity: f32, radius: f32) -> Self {
        Self {
            kind: SourceKind::Hydrothermal,
            position,
            intensity,
            radius,
            surface_only: false,
        }
    }

    pub fn solar_uv(intensity: f32, surface_only: bool) -> Self {
        Self {
            kind: SourceKind::SolarUv,
            position: (0.0, 0.0, 0.0),
            intensity,
            radius: 0.0,
            surface_only,
        }
    }
}
