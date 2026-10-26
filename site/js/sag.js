// Shelf sag, ported from crates/joinery/src/sag.rs so the page computes
// exactly what the server does. The species table mirrors
// crates/joinery/src/model.rs; site/tests/sag.test.mjs fails if they drift.

export const MM_PER_INCH = 25.4;
export const LB_PER_KG = 2.2046226;
export const LIMIT_IN_PER_FT = 1 / 32;

export const SPECIES = [
  { id: "red-oak", name: "Red oak", moe: 1_820_000, creep: 1.5 },
  { id: "white-oak", name: "White oak", moe: 1_780_000, creep: 1.5 },
  { id: "hard-maple", name: "Hard maple", moe: 1_830_000, creep: 1.5 },
  { id: "cherry", name: "Black cherry", moe: 1_490_000, creep: 1.5 },
  { id: "walnut", name: "Black walnut", moe: 1_680_000, creep: 1.5 },
  { id: "poplar", name: "Yellow poplar", moe: 1_580_000, creep: 1.5 },
  { id: "white-pine", name: "Eastern white pine", moe: 1_240_000, creep: 1.5 },
  { id: "douglas-fir", name: "Douglas fir", moe: 1_950_000, creep: 1.5 },
  { id: "birch-ply", name: "Birch plywood", moe: 1_200_000, creep: 1.75 },
  { id: "mdf", name: "MDF", moe: 400_000, creep: 2.0 },
  { id: "particleboard", name: "Particleboard", moe: 300_000, creep: 2.0 },
];

export const LOADS = { books: 20, dishes: 25, records: 35, decor: 5 }; // lb per foot

/**
 * Simply supported beam, uniform load: δ = 5WL³ / 384EI, I = bt³/12.
 * Inputs in millimetres and kilograms; returns millimetres.
 */
export function check({ spanMm, depthMm, thicknessMm, loadKg }, species) {
  const L = spanMm / MM_PER_INCH;
  const b = depthMm / MM_PER_INCH;
  const t = thicknessMm / MM_PER_INCH;
  const W = loadKg * LB_PER_KG;
  const I = (b * t ** 3) / 12;
  const deflection = (5 * W * L ** 3) / (384 * species.moe * I);
  const longTerm = deflection * species.creep;
  const limit = (LIMIT_IN_PER_FT * L) / 12;
  const utilisation = longTerm / limit;
  return {
    deflectionMm: deflection * MM_PER_INCH,
    longTermMm: longTerm * MM_PER_INCH,
    limitMm: limit * MM_PER_INCH,
    utilisation,
    passes: utilisation <= 1,
    maxSpanMm: L * Math.cbrt(1 / utilisation) * MM_PER_INCH,
    minThicknessMm: t * Math.cbrt(utilisation) * MM_PER_INCH,
  };
}

/** Load for a preset over a span, in kilograms. */
export function presetLoadKg(preset, spanMm) {
  return (LOADS[preset] * (spanMm / MM_PER_INCH / 12)) / LB_PER_KG;
}
