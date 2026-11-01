// Helpers available to every anim.js. Kept tiny and deterministic.
function lerp(a, b, t) { return a + (b - a) * t; }
function clamp(v, lo, hi) { return Math.min(hi, Math.max(lo, v)); }
function ease(t) { return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2; }
function ease_in(t) { return t * t * t; }
function ease_out(t) { return 1 - Math.pow(1 - t, 3); }
function wave(f, period) { return Math.sin(2 * Math.PI * f / period); }
function pingpong(f, n) {
  var p = 2 * (n - 1);
  if (p <= 0) return 0;
  var k = ((f % p) + p) % p;
  return k < n ? k : p - k;
}
function cycle(list, f) { return list[((f % list.length) + list.length) % list.length]; }
