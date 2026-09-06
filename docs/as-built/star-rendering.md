# Star Rendering

The starfield renders emissive Gaussian spots on shared rectangles, with GPU
twinkling and pixel integration to keep small stars stable as the camera moves.

## Architecture and data flow

`src/camera/stars.rs` generates positions, radii, colors, and twinkle parameters
when entering `Splash` or `GameOver`. Every star shares one rectangle mesh and
one additive `StarMaterial`; each entity's `MeshTag` indexes its GPU data.
Transforms retain world positions and radius scales, while CPU rotation updates
translations.

`src/camera/star_material.rs` defines `StarMaterial` as an extended
`StandardMaterial`. Read-only storage bindings contain the `StarData` array at
binding 100 and a shared clock vector at 101. Rust and `assets/shaders/star.wgsl`
must agree on the two-vector `StarData` layout: emissive color, then initial
phase, amplitude fraction, speed fraction, and padding.

`src/camera/star_twinkling.rs` owns the CPU data copy and integrates master speed
into an `f64` phase. Ordinary frames upload only the clock vector: phase,
amplitude, and padding. The shader computes each star's brightness multiplier as
`max(1 + amplitude × amplitude_fraction × sin(initial_phase + speed_fraction × phase), 0)`.

## Invariants

- Tags must remain aligned with the shared data array.
- Live speed changes preserve accumulated phase; zero speed freezes it. At an
  absolute master phase of 1024 radians, each star absorbs its scaled phase
  modulo τ before the clock resets. Both buffers update together to preserve
  continuity and GPU float precision.
- Respawning resets rotation and twinkle state. A zero-count spawn clears data
  and buffer handles.
- Rotation and twinkling continue during gameplay pause: they use ordinary
  `Time` in ungated `Update` systems. Gameplay pause stops physics and gated
  gameplay systems.

## Calibration and gotchas

The shader projects world radius into physical render pixels and uses
`sigma = max(radius / sqrt(2), 0.4246609)`, giving a minimum Gaussian full width
at half maximum of one pixel. Rectangles extend four sigma plus a half-pixel
integration margin.

Each fragment integrates the Gaussian over its pixel using normal-CDF
differences. Multiplying by `π × projected_radius²` conserves the intended
disk-equivalent light, apart from truncated tails and approximation error.
Enlarging tiny footprints therefore lowers their peak brightness instead of
adding light.

Additive blending prevents rectangle margins from hiding other stars. The
material's black base color and zero reflectance prevent lighting from revealing
those margins.

Preserve the material's emissive alpha: Bevy uses it as exposure weight. Apply
coverage and twinkle only to RGB. The shader retains Bevy's PBR and post-lighting
processing.
