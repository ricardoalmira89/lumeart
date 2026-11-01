// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// La "aguja": dardo alienígena en vista cenital, mira hacia abajo (hacia el jugador).
// Luz desde arriba a la izquierda.
setLight(-1, -1)
const CX = 512
const P = {
  ink: "#2a0f1e",      // contorno oscuro (violeta muy oscuro)
  body: "#ff4a26",     // rojo anaranjado
  fin: "#b8182a",      // rojo oscuro de las aletas
  band: "#3a1630",     // banda oscura
  metal: "#ffd23a",    // punta amarilla metálica
  cyan: "#22e6ff",     // sensor
  flameO: "#ff8a1c",   // llama naranja
  flameY: "#fff27a",   // núcleo de la llama
  white: "#ffffff",
}

// Formas base (simétricas)
const half = (pts) => [...pts, ...pts.slice().reverse().map(([x, y, s]) => s === undefined ? [2 * CX - x, y] : [2 * CX - x, y, s])]
const bodyPts = [[CX, 132], [548, 158], [592, 250], [614, 380], [602, 520], [568, 640], [552, 712, 0.4]]
  .concat([[472, 712, 0.4], [456, 640], [422, 520], [410, 380], [432, 250], [476, 158]])
const finL = [[500, 330], [430, 280, 0.5], [352, 112, 0], [432, 168, 0.5], [496, 196]]
const finR = finL.map(([x, y, s]) => s === undefined ? [2 * CX - x, y] : [2 * CX - x, y, s])
const finC = [[CX, 118, 0], [538, 210], [532, 330], [CX, 352, 0.5], [492, 330], [486, 210]]
const tipPts = [[462, 716, 0], [562, 716, 0], [548, 780, 0.6], [CX, 970, 0], [476, 780, 0.6]]
const flamePts = [[CX, 36, 0], [560, 100], [558, 150], [CX, 172], [466, 150], [464, 100]]

layer("borde-pegatina", () => {
  const st = { fill: P.white, stroke: P.white, width: 56, join: "round" }
  blob(flamePts, st)
  blob(finL, st); blob(finR, st)
  blob(bodyPts, st)
  blob(tipPts, st)
})

layer("llama", () => {
  const f = blob(flamePts, { fill: P.flameO })
  clip(f, () => {
    ellipse(CX + 12, 70, 40, 90, { fill: shade(P.flameO, -0.35), opacity: 0.6 })
  })
  blob([[CX, 74, 0], [532, 116], [526, 158], [498, 158], [492, 116]], { fill: P.flameY })
  ink(f, { width: 10, weight: 1.8, color: P.ink })
})

layer("aletas-laterales", () => {
  for (const pts of [finL, finR]) {
    const f = blob(pts, { fill: P.fin })
    cel(f, { color: shade(P.fin, -0.45) })
    ink(f, { width: 12, weight: 1.5, color: P.ink })
  }
  // brillo en la aleta izquierda
  brush([[438, 186], [400, 156], [372, 132]], { width: 12, color: alpha(P.white, 0.75), taper: [0.3, 0.5] })
})

layer("cuerpo", () => {
  // tobera
  rect(476, 128, 72, 34, { fill: P.band, r: 10 })
  const b = blob(bodyPts, { fill: P.body })
  clip(b, () => {
    // sombra de un tono a la derecha
    blob([[560, 140], [640, 300], [640, 600], [580, 730], [540, 730], [585, 560], [592, 380], [565, 240]], { fill: shade(P.body, -0.4) })
    // franja de luz a la izquierda
    brush([[470, 200], [440, 300], [432, 420], [446, 560], [470, 650]], { width: 22, color: shade(P.body, 0.45), taper: [0.35, 0.4] })
  })
  ink(b, { width: 13, weight: 1.5, color: P.ink })
  // brillo blanco marcado
  brush([[468, 226], [452, 290], [446, 350]], { width: 16, color: P.white, taper: [0.3, 0.5] })
  circle(452, 392, 8, { fill: P.white })
})

layer("aleta-central", () => {
  const f = blob(finC, { fill: P.fin })
  clip(f, () => rect(CX, 100, 60, 280, { fill: shade(P.fin, -0.45) }))
  ink(f, { width: 10, weight: 1.4, color: P.ink })
  brush([[505, 160], [498, 220], [498, 280]], { width: 7, color: alpha(P.white, 0.8) })
})

layer("sensor", () => {
  circle(CX, 470, 70, { fill: P.ink })
  const s = circle(CX, 470, 54, { fill: P.cyan, glow: { blur: 22, color: alpha(P.cyan, 0.8) } })
  clip(s, () => {
    circle(CX + 18, 488, 50, { fill: shade(P.cyan, -0.4) })
    circle(CX - 6, 462, 38, { fill: P.cyan })
    circle(CX, 470, 16, { fill: "#0a5a78" })
  })
  ellipse(CX - 22, 446, 18, 12, { fill: P.white, rotate: -35 })
  circle(CX + 14, 438, 5, { fill: P.white })
})

layer("punta", () => {
  const t = blob(tipPts, { fill: P.metal })
  clip(t, () => {
    poly([[CX, 700], [570, 700], [CX, 990]], { fill: shade(P.metal, -0.4) })
  })
  ink(t, { width: 11, weight: 1.4, color: P.ink })
  brush([[484, 744], [492, 820], [504, 900]], { width: 12, color: P.white, taper: [0.2, 0.6] })
  // banda oscura donde empieza la punta
  const band = rect(458, 690, 108, 42, { fill: P.band, r: 14 })
  ink(band, { width: 10, weight: 1.6, color: P.ink })
  rect(472, 697, 40, 8, { fill: alpha(P.white, 0.35), r: 4 })
})
