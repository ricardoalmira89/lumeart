// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Dron escolta frontal: vista cenital, mira hacia arriba. Luz arriba a la izquierda.
setLight(-1, -1)
const P = {
  ink: "#141a33",       // contorno oscuro azul-noche
  white: "#f4f7fb",     // casco
  whiteSh: "#b9c4d8",   // sombra del casco
  blue: "#1f5bff",      // banda y aletas
  blueSh: "#1233a8",
  blueLt: "#6f9bff",
  metal: "#4a5163",     // cañones
  metalSh: "#2b3040",
  metalLt: "#7d8598",
  cyan: "#3ff2ff",      // lente y bocas
  cyanDk: "#0a8fb8",
  glow: "#9fd8ff",      // propulsor
}

const CX = 512, CY = 585, R = 280
const GX = 92            // separación de los cañones respecto al eje
const GW = 112, GTOP = 70

// Siluetas (se reutilizan para el borde de pegatina)
const finPts = [[CX - R + 30, CY - 70], [CX - R - 120, CY + 10, 0], [CX - R - 110, CY + 120, 0], [CX - R + 40, CY + 110]]
const finPtsR = finPts.map(([x, y, s]) => s === undefined ? [2 * CX - x, y] : [2 * CX - x, y, s])
const thrPts = [[CX - 85, CY + R - 40], [CX + 85, CY + R - 40], [CX + 62, CY + R + 60, 0.5], [CX - 62, CY + R + 60, 0.5]]

function silhouette(st) {
  const s = []
  s.push(blob(finPts, st), blob(finPtsR, st))
  s.push(poly(thrPts, st))
  for (const sx of [-1, 1]) s.push(rect(CX + sx * GX - GW / 2, GTOP, GW, 380, { ...st, r: 26 }))
  s.push(circle(CX, CY, R, st))
  return s
}

layer("borde-pegatina", () => {
  silhouette({ fill: "#ffffff", stroke: "#ffffff", width: 90, behind: true, join: "round" })
  silhouette({ fill: P.ink, stroke: P.ink, width: 40, join: "round" })
})

layer("propulsor", () => {
  ellipse(CX, CY + R + 95, 70, 45, { fill: P.glow, opacity: 0.0 })
  const t = poly(thrPts, { fill: P.metal })
  clip(t, () => {
    rect(CX, CY + R - 50, 120, 140, { fill: P.metalSh })
    rect(CX - 80, CY + R - 50, 30, 140, { fill: P.metalLt })
  })
  ellipse(CX, CY + R + 58, 58, 20, { fill: P.glow, stroke: P.ink, width: 10 })
  ellipse(CX - 12, CY + R + 54, 26, 8, { fill: "#ffffff" })
})

layer("aletas", () => {
  for (const pts of [finPts, finPtsR]) {
    const f = blob(pts, { fill: P.blue })
    clip(f, () => {
      // sombra de un tono abajo
      rect(0, CY + 60, 1024, 120, { fill: P.blueSh })
      brush([[pts[1][0] + (pts === finPts ? 20 : -20), CY + 5], [(pts[0][0] + pts[1][0]) / 2, CY - 30]], { width: 22, color: P.blueLt, taper: [0.2, 0.4] })
    })
  }
})

layer("cañones", () => {
  for (const sx of [-1, 1]) {
    const x = CX + sx * GX - GW / 2
    const g = rect(x, GTOP, GW, 380, { fill: P.metal, r: 26 })
    clip(g, () => {
      rect(x + GW * 0.62, GTOP, GW, 400, { fill: P.metalSh })
      rect(x + 16, GTOP + 70, 20, 260, { fill: P.metalLt, r: 10 })
      // anillo de refuerzo
      rect(x, GTOP + 150, GW, 34, { fill: P.metalSh })
      rect(x, GTOP + 150, GW, 10, { fill: P.metalLt, opacity: 0.6 })
    })
    // boca cian
    rect(x + 14, GTOP + 12, GW - 28, 46, { fill: P.cyan, stroke: P.ink, width: 12, r: 18 })
    rect(x + 26, GTOP + 20, 26, 12, { fill: "#ffffff", r: 6 })
  }
})

layer("casco", () => {
  const body = circle(CX, CY, R, { fill: P.blue })
  clip(body, () => {
    // sombra de la banda (abajo derecha)
    circle(CX, CY, R, { fill: P.blueSh })
    circle(CX - 34, CY - 34, R, { fill: P.blue })
  })
  // brillo de la banda
  brush(arc(CX, CY, R - 32, -170, -100, 10), { width: 26, color: P.blueLt, taper: [0.3, 0.3] })
  // cúpula blanca
  const cap = circle(CX, CY, R - 82, { fill: P.white, stroke: P.ink, width: 22 })
  clip(cap, () => {
    circle(CX, CY, R - 82, { fill: P.whiteSh })
    circle(CX - 30, CY - 30, R - 86, { fill: P.white })
  })
  // brillo nítido de la cúpula
  brush(arc(CX, CY, R - 130, -150, -105, 8), { width: 26, color: "#ffffff", taper: [0.3, 0.3] })
  circle(CX - 145, CY - 25, 14, { fill: "#ffffff" })
})

layer("lente", () => {
  circle(CX, CY, 104, { fill: P.metalSh, stroke: P.ink, width: 20 })
  const lens = circle(CX, CY, 80, { fill: P.cyan })
  clip(lens, () => {
    circle(CX, CY, 80, { fill: P.cyanDk })
    circle(CX - 16, CY - 16, 78, { fill: P.cyan })
  })
  circle(CX, CY, 36, { fill: "#d8feff" , glow: { blur: 30, color: P.cyan } })
  ellipse(CX - 34, CY - 36, 24, 15, { fill: "#ffffff", rotate: -40 })
  circle(CX + 34, CY + 32, 8, { fill: "#ffffff", opacity: 0.8 })
})
