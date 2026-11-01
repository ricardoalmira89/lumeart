// Geoda: mina flotante de cristal, vista cenital, estilo pegatina.
canvas({ width: 1024, height: 1024, background: null })
seed(7)
setLight(-1, -1)

const P = {
  linea: "#1c0f33",      // contorno oscuro
  cian: "#2fe0ff",
  cianSom: "#1688c9",
  lila: "#c9a2ff",
  lilaSom: "#7b4fd6",
  roca: "#4a2a78",
  rocaSom: "#2a1550",
  rocaLuz: "#6e45a8",
  magenta: "#ff2fa8",
  magentaLuz: "#ffb3e2",
  blanco: "#ffffff",
}
const C = [512, 512]
const LUZ = [-0.707, -0.707]

// punto en coordenadas del cristal: r a lo largo del eje, w perpendicular
function pt(ang, r, w) {
  const a = (ang - 90) * Math.PI / 180
  const ux = Math.cos(a), uy = Math.sin(a)
  return [C[0] + ux * r - uy * w, C[1] + uy * r + ux * w]
}
const CRISTALES = range(6).map(i => {
  const ang = i * 60
  const cian = i % 2 === 0
  const L = cian ? 452 : 418
  const W = cian ? 104 : 108
  const H = cian ? 265 : 250 // radio del hombro (lo más ancho)
  return {
    ang, cian, L, W, H,
    base: cian ? P.cian : P.lila,
    som: cian ? P.cianSom : P.lilaSom,
    sil: [pt(ang, 120, -70), pt(ang, H, -W), pt(ang, L, 0), pt(ang, H, W), pt(ang, 120, 70)],
  }
})
const NUCLEO = jitter(ring(512, 512, 182, 182, 10, 18), 8)

// --- Borde blanco de pegatina detrás de toda la silueta
layer("borde-pegatina", () => {
  for (const c of CRISTALES) poly(c.sil, { fill: P.blanco, stroke: P.blanco, width: 64, join: "round" })
  poly(NUCLEO, { fill: P.blanco, stroke: P.blanco, width: 64, join: "round" })
})

// --- Contorno oscuro grueso
layer("contorno", () => {
  for (const c of CRISTALES) poly(c.sil, { fill: P.linea, stroke: P.linea, width: 26, join: "round" })
  poly(NUCLEO, { fill: P.linea, stroke: P.linea, width: 26, join: "round" })
})

// --- Cristales facetados
layer("cristales", () => {
  for (const c of CRISTALES) {
    const a = (c.ang - 90) * Math.PI / 180
    // normal del lado "w negativo"
    const nx = Math.sin(a), ny = -Math.cos(a)
    const ladoLuz = (nx * LUZ[0] + ny * LUZ[1]) > 0 ? -1 : 1
    const s = c.sil
    // cara en sombra (silueta entera) y cara iluminada (mitad)
    const forma = poly(s, { fill: c.som })
    const mitad = ladoLuz < 0
      ? [s[0], s[1], s[2], pt(c.ang, 120, 0)]
      : [pt(c.ang, 120, 0), s[2], s[3], s[4]]
    poly(mitad, { fill: c.base })
    // faceta de la base en la mitad oscura, más honda
    const k = ladoLuz
    poly([pt(c.ang, 120, -k * 70), pt(c.ang, c.H, -k * c.W), pt(c.ang, c.H + 20, -k * 30), pt(c.ang, 130, 0)],
      { fill: shade(c.som, -0.3) })
    // cara blanca iluminada
    poly([pt(c.ang, 160, k * 16), pt(c.ang, c.H, k * (c.W - 34)), pt(c.ang, c.L - 45, k * 8), pt(c.ang, c.H + 10, k * 18)],
      { fill: P.blanco, opacity: 0.95 })
    // arista central
    brush([pt(c.ang, 125, 0), pt(c.ang, c.L - 10, 0)], { width: 10, taper: [0.1, 0.6], color: P.linea })
  }
})

// --- Núcleo de roca
layer("nucleo", () => {
  const roca = poly(NUCLEO, { fill: P.roca, stroke: P.linea, width: 26, join: "round", behind: true })
  clip(roca, () => {
    // facetas: cada borde con su cara inclinada, tono según mire a la luz
    const n = NUCLEO.length
    for (let i = 0; i < n; i++) {
      const p1 = NUCLEO[i], p2 = NUCLEO[(i + 1) % n]
      const mx = (p1[0] + p2[0]) / 2 - 512, my = (p1[1] + p2[1]) / 2 - 512
      const d = Math.hypot(mx, my)
      const luz = (mx * LUZ[0] + my * LUZ[1]) / d
      const tono = luz > 0.4 ? P.rocaLuz : luz < -0.3 ? P.rocaSom : P.roca
      const q1 = [512 + (p1[0] - 512) * 0.62, 512 + (p1[1] - 512) * 0.62]
      const q2 = [512 + (p2[0] - 512) * 0.62, 512 + (p2[1] - 512) * 0.62]
      poly([p1, p2, q2, q1], { fill: tono, stroke: shade(P.rocaSom, -0.3), width: 4, join: "round" })
    }
  })
})

// --- Corazón magenta
layer("corazon", () => {
  circle(512, 512, 106, { fill: P.linea })
  const cor = circle(512, 512, 94, { fill: P.magenta })
  clip(cor, () => {
    circle(538, 540, 96, { fill: "#c0168a" })        // sombra de un tono
    circle(498, 498, 82, { fill: P.magenta })
  })
  ellipse(478, 472, 34, 21, { fill: P.blanco, rotate: -40 })
  circle(540, 452, 10, { fill: P.blanco })
})
