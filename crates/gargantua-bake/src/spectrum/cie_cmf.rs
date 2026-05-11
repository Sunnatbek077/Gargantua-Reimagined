// ============================================================
// FILE: crates/gargantua-bake/src/spectrum/cie_cmf.rs
// LINES: ~180
// CATEGORY: Bake — CIE 1931 Color Matching Functions data
// PLATFORM: cross-platform (Mac + Windows)
// ============================================================
//
// PURPOSE:
//   Provides the CIE 1931 2° standard observer color matching
//   functions (CMFs) as a static data table. Used by blackbody.rs
//   and doppler_lut.rs to perform spectral → XYZ integration.
//   Raw data is embedded in the binary via include_bytes! or a
//   hardcoded const array.
//
// CONTENTS (~180 lines):
//   // CIE 1931 CMF tabulated data (Stiles & Burch 1955 revision)
//   // 85 entries, 5nm steps, 360nm to 780nm
//   //
//   // Format: (wavelength_nm, x_bar, y_bar, z_bar)
//   pub const CIE_CMF_DATA: &[(f64, f64, f64, f64)] = &[
//       (360.0, 0.000130, 0.000004, 0.000606),
//       (365.0, 0.000232, 0.000007, 0.001086),
//       // ... 83 more entries ...
//       (780.0, 0.000002, 0.000001, 0.000000),
//   ];
//
//   // Convenience wrapper that returns gargantua-physics CieCmf struct
//   pub fn load_cmf() -> gargantua_physics::accretion::spectrum::CieCmf
//     // CieCmf::from_raw(CIE_CMF_DATA)
//
//   // Wavelength range constants
//   pub const CMF_LAMBDA_MIN: f64 = 360.0;  // nm
//   pub const CMF_LAMBDA_MAX: f64 = 780.0;  // nm
//   pub const CMF_STEP:       f64 = 5.0;    // nm per entry
//   pub const CMF_N_ENTRIES:  usize = 85;
//
// USES (imports from):
//   gargantua_physics::accretion::spectrum::CieCmf
//
// USED BY:
//   crate::spectrum::blackbody    → load_cmf() for blackbody LUT
//   crate::spectrum::doppler_lut  → load_cmf() for Doppler LUT
//
// NOTE FOR AI:
//   CIE_CMF_DATA is a compile-time const — no runtime I/O needed.
//   Source: CIE publication 15:2004, Table 1 (2° observer, 5nm steps).
//   y_bar values sum to approximately 106.857 (not normalized).
//   Normalization is handled inside blackbody_to_xyz() in spectrum.rs.
//   Do NOT modify CIE_CMF_DATA — it is a physical standard constant.
// ============================================================