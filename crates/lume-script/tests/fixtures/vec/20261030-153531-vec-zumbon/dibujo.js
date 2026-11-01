// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Zumbón: dron insecto enemigo, vista cenital, mira hacia abajo.
const P = {
  lima: "#9be22a",
  limaSombra: "#4f9a1c",
  limaLuz: "#e4ff7a",
  franja: "#2f6e1a",
  cabeza: "#3f8a22",
  tinta: "#16301a",
  visor: "#ff2a3a",
  visorLuz: "#ffd0d0",
  ala: "#a8dcff",
  alaSombra: "#6ea8e6",
  vena: "#ffffff",
  borde: "#ffffff",
  mandibula: "#d8f070",
}
setLight(-1, -1)
const CX = 512

// Siluetas (ocultas) para reutilizarlas en borde, tinta y clips
const alaPts = [[452, 430], [360, 330], [230, 250], [120, 222], [62, 262], [80, 350], [170, 430], [310, 490], [440, 500]]
const abdPts = [[512, 62, 0], [600, 110], [640, 210], [632, 320], [580, 405], [512, 425], [444, 405], [392, 320], [384, 210], [424, 110]]
const torPts = ring(CX, 480, 112, 86, 12)
const cabPts = [[512, 552], [612, 572], [642, 640], [608, 720], [512, 752], [416, 720], [382, 640], [412, 572]]
const mandPts = [[432, 712], [488, 736], [480, 780], [500, 846, 0], [448, 812], [420, 766]]

// Sombra de un solo tono: la forma menos una copia desplazada hacia la luz
function sombra(pts, color, dx = -34, dy = -30) {
  const s = blob(pts, { fill: color })
  cut(s, [blob(shift(pts, dx, dy), { hidden: true })])
}

layer("borde", () => {
  // Borde blanco de pegatina alrededor de toda la figura
  const sil = []
  mirror(CX, () => {
    sil.push(blob(alaPts, { fill: P.borde, stroke: P.borde, width: 56, behind: true }))
    sil.push(blob(mandPts, { fill: P.borde, stroke: P.borde, width: 56, behind: true }))
  })
  blob(abdPts, { fill: P.borde, stroke: P.borde, width: 56 })
  blob(torPts, { fill: P.borde, stroke: P.borde, width: 56 })
  blob(cabPts, { fill: P.borde, stroke: P.borde, width: 56 })
})

layer("alas", () => {
  mirror(CX, () => {
    const a = blob(alaPts, { fill: alpha(P.ala, 0.82) })
    clip(a, () => {
      blob([[452, 440], [330, 420], [170, 400], [90, 330], [70, 420], [200, 520], [452, 520]], { fill: alpha(P.alaSombra, 0.7) })
      blob([[400, 360], [260, 270], [150, 245], [180, 280], [300, 330]], { fill: alpha("#ffffff", 0.55) })
    })
    brush([[448, 455], [340, 380], [220, 310], [110, 270]], { width: [16, 12, 4], taper: [0, 0.5], color: P.vena })
    brush([[300, 360], [240, 410], [170, 420]], { width: [9, 4], taper: [0, 0.6], color: alpha(P.vena, 0.85) })
    ink(a, { width: 9, weight: 2.2, color: "#1d3a6a" })
  })
})

layer("mandibulas", () => {
  mirror(CX, () => {
    const m = blob(mandPts, { fill: P.mandibula })
    clip(m, () => sombra(mandPts, shade(P.mandibula, -0.4), -14, -10))
    ink(m, { width: 9, weight: 2.2, color: P.tinta })
  })
})

layer("abdomen", () => {
  const ab = blob(abdPts, { fill: P.lima })
  clip(ab, () => {
    for (const y of [140, 220, 305]) {
      blob([[360, y], [512, y + 22], [664, y], [664, y + 34], [512, y + 58], [360, y + 34]], { fill: P.franja })
    }
    sombra(abdPts, alpha(P.franja, 0.55), -40, -20)
    ellipse(458, 175, 22, 60, { fill: alpha("#ffffff", 0.85), rotate: 12 })
  })
  ink(ab, { width: 11, weight: 2.2, color: P.tinta })
})

layer("torax", () => {
  const t = blob(torPts, { fill: P.lima })
  clip(t, () => {
    sombra(torPts, P.limaSombra, -34, -26)
    ellipse(470, 445, 40, 20, { fill: alpha("#ffffff", 0.85), rotate: -15 })
  })
  ink(t, { width: 11, weight: 2.2, color: P.tinta })
})

layer("cabeza", () => {
  const c = blob(cabPts, { fill: P.cabeza })
  clip(c, () => {
    sombra(cabPts, shade(P.cabeza, -0.4), -34, -28)
    ellipse(455, 590, 34, 14, { fill: alpha("#ffffff", 0.7), rotate: -12 })
  })
  // Visor rojo
  const v = rect(404, 622, 216, 60, { r: 30, fill: P.visor, stroke: P.tinta, width: 10 })
  clip(v, () => {
    rect(400, 656, 224, 30, { fill: shade(P.visor, -0.35) })
  })
  rect(432, 633, 84, 14, { r: 7, fill: P.visorLuz })
  circle(588, 641, 7, { fill: "#ffffff" })
  ink(c, { width: 11, weight: 2.2, color: P.tinta })
})
