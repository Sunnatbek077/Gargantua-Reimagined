# Gargantua — Physics Reference

This document describes every physical model implemented in `gargantua-physics` and used by the render pipeline. It covers the spacetime metric, geodesic integration, accretion disk thermodynamics, and relativistic optical effects. All equations use geometrised units: **G = c = 1**.

---

## Unit System

All physics code uses geometrised units where the gravitational constant G and the speed of light c are both set to 1. In these units:

- Mass M has dimensions of length (metres) and time (seconds).
- The Schwarzschild radius is simply r_s = 2M.
- Velocities are dimensionless fractions of c.

Conversion helpers are in `src/units.rs`:

```
r_s = 2GM/c²   (in SI metres)
M_geometric = GM/c²   (geometric mass in metres)
```

The UI displays mass in solar masses (M☉ = 1.989 × 10³⁰ kg). Internally everything is converted to geometric units before any physics calculation.

---

## Spacetime Metric

### Kerr-Newman Metric (`src/metric/kerr.rs`)

Gargantua uses the Kerr-Newman metric in Boyer-Lindquist coordinates (t, r, θ, φ). This is the most general stationary, axisymmetric black hole solution, parameterised by:

- **M** — mass (geometric units)
- **a** — specific angular momentum = J/M, with |a| < M (spin parameter)
- **Q** — electric charge (usually 0 for astrophysical black holes)

The physical constraint for a sub-extremal black hole is: a² + Q² < M².

**Auxiliary quantities:**

```
Σ(r, θ) = r² + a²cos²θ
Δ(r)    = r² − 2Mr + a² + Q²
```

**Metric components (non-zero):**

```
g_tt   = −(1 − (2Mr − Q²)/Σ)
g_rr   = Σ/Δ
g_θθ   = Σ
g_φφ   = (r² + a² + (2Mr − Q²)a²sin²θ/Σ) sin²θ
g_tφ   = −(2Mr − Q²)a sin²θ / Σ     ← frame dragging cross-term
```

**Key radii:**

```
Event horizon:     r_+ = M + √(M² − a² − Q²)
Inner horizon:     r_− = M − √(M² − a² − Q²)
Ergosphere:        r_e(θ) = M + √(M² − a²cos²θ − Q²)
Photon sphere:     r_ph ≈ 3M (Schwarzschild); 1M–3M range for Kerr
```

All 40 non-zero Christoffel symbols Γ^μ_νρ are implemented analytically in `kerr.rs::christoffel()`. Using analytic expressions (rather than finite differences) is ~3× faster and ~10× more accurate near the horizon where Δ → 0.

### Schwarzschild Metric (`src/metric/schwarzschild.rs`)

The special case a = Q = 0. Fully spherically symmetric — a diagonal metric with:

```
g_tt = −(1 − 2M/r),   g_rr = (1 − 2M/r)^{−1},   g_θθ = r²,   g_φφ = r²sin²θ
```

Used for unit tests (compare against Kerr with a = 0) and as the default when the user sets spin to zero in the UI.

---

## Geodesic Integration

### The Geodesic Equation

Photons follow null geodesics — paths through curved spacetime where the 4-velocity k^μ satisfies:

```
dk^μ/dλ = −Γ^μ_νρ k^ν k^ρ
dX^μ/dλ =  k^μ
```

where λ is the affine parameter along the path and Γ^μ_νρ are the Christoffel symbols of the metric.

The null condition k^μ k_μ = 0 must hold throughout integration (photons travel on the light cone). The integrator preserves this to within the chosen truncation error.

### Fixed-Step RK4 (`src/geodesic/rk4.rs`)

Standard 4th-order Runge-Kutta integration. Given a state (X^μ, k^μ), one step of size h:

```
k1 = f(state)
k2 = f(state + h/2 × k1)
k3 = f(state + h/2 × k2)
k4 = f(state + h × k3)
next = state + h/6 × (k1 + 2k2 + 2k3 + k4)
```

where `f` evaluates the right-hand side of the geodesic equation (the Christoffel contraction).

**Step size:** h = 0.1–0.5 M (affine parameter units). Smaller near the horizon.

**Performance:** ~5–15 million steps/second on Apple M-series (see `benches/geodesic_rk4.rs`).

### Adaptive RK4-5 (`src/geodesic/adaptive.rs`)

Cash-Karp Runge-Kutta with automatic step size control. Computes both RK4 and RK5 (Dormand-Prince) solutions per step:

```
error = |RK5 − RK4| / |RK4|

if error > ε_target:  halve h, retry
if error < 0.1 × ε_target:  double h for next step
```

Default tolerance ε = 1×10⁻⁶. Near the horizon (Δ → 0), h automatically shrinks to maintain accuracy. Used for camera geodesics in real-time mode; the GPU baking pass uses fixed-step for predictable compute time.

### Termination Conditions (`src/geodesic/termination.rs`)

Integration stops when:

1. **ReachedHorizon** — r ≤ r₊ (photon absorbed by the black hole)
2. **EscapedToInfinity** — r > r_max (default 1000 M; contributes to background starfield)
3. **MaxStepsReached** — safety guard after 1024 steps (indicates divergence)

---

## Accretion Disk

### ISCO (`src/accretion/isco.rs`)

The Innermost Stable Circular Orbit defines the inner edge of the accretion disk. Matter inside r_ISCO cannot orbit stably and spirals into the black hole.

**Bardeen-Press-Teukolsky formula:**

```
z₁ = 1 + (1 − a²)^{1/3} [(1 + a)^{1/3} + (1 − a)^{1/3}]
z₂ = √(3a² + z₁²)
r_ISCO = M(3 + z₂ ∓ √((3 − z₁)(3 + z₁ + 2z₂)))
```

where − gives the prograde (co-rotating) ISCO and + gives the retrograde ISCO.

**Limits:**
- Schwarzschild (a = 0): r_ISCO = 6M
- Maximal prograde (a → M): r_ISCO → M (grazing the horizon)
- Maximal retrograde (a → M): r_ISCO → 9M

**Luminosity efficiency:**

The radiative efficiency η = 1 − E_ISCO where E_ISCO is the specific binding energy at the ISCO. For Schwarzschild: η ≈ 5.72%. For maximal Kerr: η ≈ 42%. This is the fraction of rest-mass energy that the disk radiates before matter crosses the ISCO. It controls how bright the disk appears in the render.

### Novikov-Thorne Disk Model (`src/accretion/novikov_thorne.rs`)

The standard model for a geometrically thin, optically thick accretion disk. Gives the radiated flux Q(r) and temperature T(r) as a function of radius:

```
Q(r) = (3GMṀ / 8πr³) × f(r, a)
T(r) = (Q(r) / σ)^{1/4}
```

where Ṁ is the mass accretion rate, σ is the Stefan-Boltzmann constant, and f(r, a) is the Novikov-Thorne relativistic correction factor that accounts for the zero-torque inner boundary condition at the ISCO and frame-dragging effects.

**Temperature range:** For a stellar-mass black hole (10 M☉) at 10% Eddington accretion: T_peak ≈ 10⁷ K (soft X-ray peak). For a supermassive AGN (10⁸ M☉): T_peak ≈ 10⁵ K (UV peak). The blackbody LUT covers 1,000 K–10⁸ K to handle both regimes.

### MHD Disk (`src/accretion/mhd.rs`)

Extends Novikov-Thorne with magneto-hydrodynamic turbulence and a Blandford-Znajek relativistic jet:

**Turbulent temperature:**
```
T_turb(r, φ) = T_NT(r) × (1 + δT/T_NT)
δT ∝ (r/r_ISCO)^{−3/4} × sin(nφ + seed)
```

The turbulent seed is incremented each frame to animate disk variability.

**Jet luminosity (Blandford-Znajek):**
```
L_jet = (κ/4π) × Φ_BH² × Ω_H²
Ω_H = a/(2Mr₊)   ← angular velocity of the horizon
```

The jet power is proportional to (a/M)² — only spinning black holes launch jets.

### Blackbody Spectrum → Colour (`src/accretion/spectrum.rs`)

Converts T(r) to a visible RGB colour by integrating the Planck function against the CIE 1931 2° colour matching functions:

```
B_λ(T) = 2hc²/λ⁵ × 1/(exp(hc/λkT) − 1)

X = ∫ B_λ(T) x̄(λ) dλ   (380–780 nm, 5 nm steps)
Y = ∫ B_λ(T) ȳ(λ) dλ
Z = ∫ B_λ(T) z̄(λ) dλ
```

XYZ is then converted to sRGB or Display P3 via the Bradford-adapted 3×3 primary matrix. The result is baked into a 1024-point log-scale 1D LUT (see `gargantua-bake`) so the GPU only needs a single texture sample per pixel instead of running the full integration.

**Colour accuracy:** ΔE₀₀ < 1.0 compared to a reference Planck integrator across the full T range. Validated in `crates/gargantua-bake/tests/spectrum.rs`.

---

## Relativistic Optical Effects

### Doppler Beaming (`src/effects/doppler.rs`)

Disk matter orbits at relativistic velocities (β = v/c up to ~0.5 at the ISCO for a Kerr black hole). This causes:

**Relativistic Doppler factor:**
```
D = 1 / (γ(1 − β cos α))

γ = 1/√(1 − β²)   (Lorentz factor)
α = angle between orbital velocity and line of sight
```

- Approaching side (α ≈ 0): D > 1 → blueshift, brightening
- Receding side (α ≈ π): D < 1 → redshift, dimming

**Observed intensity and frequency:**
```
I_obs = I_emit × D⁴     (includes beaming: D³ for solid angle + D for frequency)
ν_obs = ν_emit × D
```

The factor D⁴ causes the approaching side of the disk to appear dramatically brighter — the characteristic asymmetry visible in all Kerr black hole images.

### Gravitational Redshift (`src/effects/redshift.rs`)

Light emitted at radius r near the black hole arrives at infinity with a frequency shift:

```
ν_∞ / ν_emit = √(−g_tt(r, π/2))   (equatorial plane)
```

For Schwarzschild: √(1 − 2M/r). At r = 2M (horizon): complete redshift (z → ∞). At r = 6M (Schwarzschild ISCO): z ≈ 0.225 (22.5% redshift).

The combined observed colour shift is:

```
F_obs = F_emit × g³ × D³
```

where g = √(−g_tt) is the gravitational redshift factor and D is the Doppler factor. This combined g³D³ factor (cubed because it applies to both frequency and solid angle) is what gives the accretion disk its characteristic colour gradient — blue-white on the approaching side, orange-red receding into the shadow.

### Stellar Aberration (`src/effects/aberration.rs`)

When the camera moves at velocity β, the apparent positions of background stars shift toward the direction of motion (relativistic headlight effect):

```
cos θ' = (cos θ + β) / (1 + β cos θ)
```

At β = 0.5c: stars within 60° of the forward direction compress into ~30° (headlight focusing). At β = 0.9c: almost all stars pile up in a ring around the direction of travel. The starfield shader applies this transformation via a 3×3 aberration matrix computed in `effects/aberration.rs`.

### Frame Dragging / Lense-Thirring Effect (`src/effects/frame_dragging.rs`)

A spinning black hole drags spacetime itself into rotation. Locally non-rotating observers (ZAMOs) are carried along at the frame-dragging angular velocity:

```
ω(r, θ) = −g_tφ / g_φφ
```

At the event horizon: ω = Ω_H = a/(2Mr₊) — the horizon rotates rigidly. In the ergosphere (r < r_e): ω > 0 even for zero angular momentum photons — they are forced to co-rotate with the black hole. This is the physical origin of the photon ring's asymmetric twist shape.

### Penrose Process (`src/effects/penrose.rs`)

Inside the ergosphere (r₊ < r < r_e), particles can have negative energy as measured by a distant observer. A particle splitting inside the ergosphere can thus extract energy from the black hole's rotation:

```
ΔE_extracted = −E_negative_fragment > 0
```

The Blandford-Znajek mechanism (implemented in `mhd.rs`) is the electromagnetic analogue — magnetic field lines threading the ergosphere extract spin energy to power the relativistic jet. The UI overlay (`PhysicsReadout`) displays whether the camera is currently inside the ergosphere and the estimated Penrose extraction rate.

---

## GPU Physics Uniforms

The `PhysicsUniforms` struct (in `gargantua-render/src/bindgroups/physics.rs`) is a flat `bytemuck::Pod` struct that packs all GPU-needed physics parameters into 256 bytes:

```wgsl
struct PhysicsUniforms {
    M:              f32,   // geometric mass
    a:              f32,   // spin parameter
    Q:              f32,   // charge
    r_plus:         f32,   // outer event horizon
    r_s:            f32,   // Schwarzschild radius
    r_isco_pro:     f32,   // prograde ISCO
    r_isco_retro:   f32,   // retrograde ISCO
    edr_headroom:   f32,   // EDR peak (Mac) or HDR10 nits / 10000 (Win)
    sim_time:       f32,   // for MHD turbulence animation seed
    disk_inner_r:   f32,
    disk_outer_r:   f32,
    accretion_rate: f32,
    beta_mag:       f32,
    ...
}
```

Uploaded once per frame by `PhysicsSync::sync()`. All WGSL shaders read from `@group(0) @binding(0) var<uniform> physics: PhysicsUniforms`.

---

## Numerical Validation

Key physics quantities are validated against known analytical results in `crates/gargantua-physics/tests/`:

| Test | Expected value | Source |
|---|---|---|
| Schwarzschild r_ISCO | 6M | Bardeen 1972 |
| Kerr a=0.998 prograde r_ISCO | 1.237M | Bardeen 1972 |
| Schwarzschild photon sphere | 3M | MTW §25.5 |
| M87* spin (a/M) | ~0.90 ± 0.05 | EHT 2021 |
| SgrA* ISCO | ~6M (low spin) | EHT 2022 |
| Novikov-Thorne η (Schwarzschild) | 5.72% | Shakura & Sunyaev 1973 |
| Novikov-Thorne η (max Kerr) | 42.3% | Thorne 1974 |

---

## References

- Bardeen, J.M., Press, W.H., Teukolsky, S.A. (1972). "Rotating Black Holes: Locally Nonrotating Frames, Energy Extraction, and Scalar Synchrotron Radiation." *ApJ* 178, 347.
- Novikov, I.D., Thorne, K.S. (1973). "Astrophysics of Black Holes." in *Black Holes*, eds. DeWitt & DeWitt.
- Thorne, K.S. (1974). "Disk-accretion onto a black hole." *ApJ* 191, 507.
- Misner, C.W., Thorne, K.S., Wheeler, J.A. (1973). *Gravitation*. Freeman.
- Event Horizon Telescope Collaboration (2021). "First M87 Event Horizon Telescope Results. VIII." *ApJL* 910, L13.