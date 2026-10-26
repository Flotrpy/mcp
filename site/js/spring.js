// A tiny velocity-aware spring (semi-implicit Euler).
//
// Re-targeting keeps the current position *and velocity*, so motion never
// jumps or hits a "brick wall" when the user changes their mind mid-flight —
// the property WWDC's "Designing Fluid Interfaces" calls interruptibility.

const reduceMotion = () => matchMedia("(prefers-reduced-motion: reduce)").matches;

export class Spring {
  /**
   * @param {number} value  starting value
   * @param {(v: number) => void} onUpdate  called every frame with the value
   * @param {{stiffness?: number, damping?: number}} [opts]
   */
  constructor(value, onUpdate, { stiffness = 220, damping = 24 } = {}) {
    this.value = value;
    this.velocity = 0;
    this.target = value;
    this.stiffness = stiffness;
    this.damping = damping;
    this.onUpdate = onUpdate;
    this.frame = 0;
    this.last = 0;
  }

  set(target) {
    this.target = target;
    if (reduceMotion()) {
      cancelAnimationFrame(this.frame);
      this.frame = 0;
      this.value = target;
      this.velocity = 0;
      this.onUpdate(target);
      return;
    }
    if (!this.frame) {
      this.last = performance.now();
      this.frame = requestAnimationFrame((t) => this.tick(t));
    }
  }

  tick(now) {
    // Fixed 1/120 s substeps keep the integration stable at any frame rate.
    let dt = Math.min((now - this.last) / 1000, 1 / 20);
    this.last = now;
    while (dt > 0) {
      const h = Math.min(dt, 1 / 120);
      const accel = -this.stiffness * (this.value - this.target) - this.damping * this.velocity;
      this.velocity += accel * h;
      this.value += this.velocity * h;
      dt -= h;
    }
    const settled = Math.abs(this.value - this.target) < 0.01 && Math.abs(this.velocity) < 0.01;
    if (settled) {
      this.value = this.target;
      this.velocity = 0;
    }
    this.onUpdate(this.value);
    this.frame = settled ? 0 : requestAnimationFrame((t) => this.tick(t));
  }
}
