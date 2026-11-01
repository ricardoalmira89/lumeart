// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)
setLight(-1, -1)

const P = {
  line: "#1c1a33",     // contorno
  white: "#f4f6fb",    // casco
  whiteSh: "#aeb6d6",  // sombra del casco
  amber: "#e8202e",    // banda (rojo intenso)
  amberSh: "#8e0f22",
  amberLt: "#ff6f6a",
  metal: "#3b4057",    // anillo y cañones
  metalSh: "#24273a",
  metalLt: "#6d7596",
  glow: "#ff3b45",
}

const CX = 512, CY = 585
const R = 330          // casco
const BAND_O = 300, BAND_I = 195
const RING = 185       // anillo giratorio
const TUR = 135        // torreta
const BX = 52, BW = 58, BTOP = 92, BBOT = 470

// Silueta completa para el borde de pegatina
function barrelPts(x) {
  return [[x - BW / 2, BBOT], [x - BW / 2, BTOP, 0], [x + BW / 2, BTOP, 0], [x + BW / 2, BBOT]]
}

layer("borde pegatina", () => {
  circle(CX, CY, R, { fill: P.white, stroke: P.white, width: 70, behind: true })
  for (const s of [-1, 1]) rect(CX + s * BX - BW / 2 - 6, BTOP - 6, BW + 12, BBOT - BTOP, { fill: P.white, stroke: P.white, width: 70, r: 14 })
  rect(CX - 120, 300, 240, 220, { fill: P.white, stroke: P.white, width: 70, r: 40 })
})

layer("contorno", () => {
  circle(CX, CY, R + 22, { fill: P.line })
  for (const s of [-1, 1]) rect(CX + s * BX - BW / 2 - 20, BTOP - 20, BW + 40, BBOT - BTOP, { fill: P.line, r: 26 })
})

layer("casco", () => {
  const hull = circle(CX, CY, R, { fill: P.white })
  clip(hull, () => {
    // sombra de un solo tono abajo a la derecha
    circle(CX - 70, CY - 70, R + 10, { fill: P.whiteSh, hidden: true })
    const sh = circle(CX + 60, CY + 60, R + 40, { fill: P.whiteSh })
    cut(sh, [circle(CX - 40, CY - 40, R - 10, { hidden: true })])
    // banda ámbar
    const band = circle(CX, CY, BAND_O, { fill: P.amber })
    cut(band, [circle(CX, CY, BAND_I, { hidden: true })])
    clip(band, () => {
      const bsh = circle(CX + 50, CY + 50, BAND_O + 30, { fill: P.amberSh })
      cut(bsh, [circle(CX - 30, CY - 30, BAND_O - 5, { hidden: true })])
      brush(arc(CX, CY, BAND_O - 30, 225, 330, 12), { width: 22, color: P.amberLt, taper: [0.4, 0.4] })
    })
    // líneas de la banda
    circle(CX, CY, BAND_O, { fill: "none", stroke: P.line, width: 12 })
    circle(CX, CY, BAND_I, { fill: "none", stroke: P.line, width: 12 })
    // brillo del casco
    brush(arc(CX, CY, R - 14, 230, 300, 10), { width: 20, color: "#ffffff", taper: [0.4, 0.4] })
    circle(CX - 222, CY - 222, 10, { fill: "#ffffff" })
  })
})

layer("anillo giratorio", () => {
  circle(CX, CY, RING + 14, { fill: P.line })
  const ring = circle(CX, CY, RING, { fill: P.metal })
  clip(ring, () => {
    const rs = circle(CX + 30, CY + 30, RING + 10, { fill: P.metalSh })
    cut(rs, [circle(CX - 18, CY - 18, RING - 4, { hidden: true })])
  })
  // muescas
  repeat_radial(CX, CY, 12, (i) => {
    rect(CX - 11, CY - RING - 4, 22, 34, { fill: P.line, r: 4 })
  })
  brush(arc(CX, CY, RING - 12, 240, 300, 8), { width: 10, color: P.metalLt })
})

layer("cañones", () => {
  // puente oscuro entre los dos cañones
  rect(CX - BX, BTOP + 60, BX * 2, BBOT - BTOP - 60, { fill: P.line })
  for (const s of [-1, 1]) {
    const x = CX + s * BX
    const b = rect(x - BW / 2, BTOP, BW, BBOT - BTOP, { fill: P.metal, r: 10, stroke: P.line, width: 30, behind: true })
    clip(b, () => {
      rect(x + BW / 2 - 18, BTOP, 18, BBOT - BTOP, { fill: P.metalSh })
      rect(x - BW / 2 + 9, BTOP + 50, 11, BBOT - BTOP - 80, { fill: P.metalLt, r: 5 })
      // aros
      rect(x - BW / 2, BTOP + 110, BW, 14, { fill: P.line })
    })
    // boca roja brillante
    rect(x - BW / 2, BTOP, BW, 42, { fill: P.glow, r: 10, glow: { blur: 18, color: P.glow } })
    rect(x - BW / 2 + 10, BTOP + 8, BW - 20, 12, { fill: "#ffc4c2", r: 6 })
    line(x - BW / 2, BTOP + 44, x + BW / 2, BTOP + 44, { stroke: P.line, width: 10 })
  }
})

layer("torreta", () => {
  circle(CX, CY, TUR + 16, { fill: P.line })
  const t = circle(CX, CY, TUR, { fill: P.white })
  clip(t, () => {
    const ts = circle(CX + 40, CY + 40, TUR + 20, { fill: P.whiteSh })
    cut(ts, [circle(CX - 22, CY - 22, TUR - 2, { hidden: true })])
    brush(arc(CX, CY, TUR - 18, 235, 305, 8), { width: 14, color: "#ffffff" })
  })
  // lente
  circle(CX, CY, 70, { fill: P.line })
  circle(CX, CY, 56, { fill: radial(CX - 12, CY - 12, 60, ["#ffc4c2", P.glow, P.amberSh]), glow: { blur: 30, color: alpha(P.glow, 0.8) } })
  circle(CX - 20, CY - 20, 14, { fill: "#ffffff" })
})
