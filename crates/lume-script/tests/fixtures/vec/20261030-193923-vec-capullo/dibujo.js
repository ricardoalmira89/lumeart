// dibujo.js: capullo de la Colmena (contenedor enemigo, vista cenital)
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1f2410",      // contorno oscuro
  seda: "#8a9a3a",     // verde oliva base
  sedaClara: "#c4cf62",
  sedaOsc: "#4f5c1e",
  ambar: "#c79a3c",    // ámbar apagado
  ambarOsc: "#7d5a1e",
  quitina: "#2e2a3a",  // tapas
  quitinaLuz: "#6a6480",
  ventana: "#f1f3ea",  // pálida, sin color propio
  ventanaSom: "#c9ccc0",
  blanco: "#ffffff",
}

const CX = 512, CY = 512
const cuerpoPts = ring(CX, CY, 292, 372, 12)
const tapaArriba = [[392, 196], [412, 120, 0.6], [470, 74], [554, 74], [612, 120, 0.6], [632, 196]]
const tapaAbajo = [[392, 828], [412, 904, 0.6], [470, 950], [554, 950], [612, 904, 0.6], [632, 828]]

// Filamentos sueltos en los bordes (detrás del borde de pegatina)
layer("filamentos", () => {
  const fil = [
    [[250, 330], [196, 300], [180, 250], [146, 228]],
    [[240, 700], [186, 730], [176, 790], [140, 812]],
    [[790, 560], [846, 588], [852, 650], [890, 676]],
  ]
  for (const f of fil) {
    curve(f, { stroke: P.blanco, width: 40, cap: "round" })
  }
  for (const f of fil) {
    curve(f, { stroke: P.ink, width: 18, cap: "round" })
    brush(f, { width: [7, 5, 2], color: P.ambar, taper: [0, 0.6] })
  }
})

// Borde blanco de pegatina y contorno oscuro
layer("pegatina", () => {
  blob(cuerpoPts, { fill: P.ink, stroke: P.blanco, width: 76, behind: true })
  blob(tapaArriba, { fill: P.ink, stroke: P.blanco, width: 76, behind: true })
  blob(tapaAbajo, { fill: P.ink, stroke: P.blanco, width: 76, behind: true })
  blob(cuerpoPts, { fill: P.ink, stroke: P.ink, width: 34 })
  blob(tapaArriba, { fill: P.ink, stroke: P.ink, width: 34 })
  blob(tapaAbajo, { fill: P.ink, stroke: P.ink, width: 34 })
})

// Cuerpo de seda con volumen
layer("cuerpo", () => {
  const cuerpo = freeform(blob(cuerpoPts, { fill: P.seda }), [
    [380, 300, P.sedaClara, 0.3],
    [512, 520, P.seda, 0.6],
    [690, 760, P.sedaOsc, 0.3],
  ])

  clip(cuerpo, () => {
    // Hebras gruesas en espiral: bandas inclinadas que se combean al rodear el óvalo
    for (let i = 0; i < 8; i++) {
      const y0 = 110 + i * 112
      const pts = [[190, y0 + 40], [350, y0 + 92], [512, y0 + 84 + rand(-6, 6)], [680, y0 + 30], [840, y0 - 50]]
      curve(shift(pts, 4, 16), { stroke: P.sedaOsc, width: 54, cap: "round" })
      curve(pts, { stroke: i % 2 ? P.ambar : mix(P.seda, P.ambar, 0.55), width: 44, cap: "round" })
      brush(shift(pts, -4, -10), { width: [2, 9, 3], color: alpha(P.sedaClara, 0.9), taper: [0.4, 0.4] })
      brush(shift(pts, 4, 24), { width: [2, 7, 2], color: P.ink, taper: [0.3, 0.3] })
    }
    // Sombra de un tono a la derecha/abajo
    const som = blob(cuerpoPts, { fill: alpha("#3a4512", 0.55), blend: "multiply" })
    cut(som, [blob(shift(grow(cuerpoPts, 0.97), -60, -70), { hidden: true })])
  })
  // Brillo blanco grande arriba a la izquierda
  brush([[300, 400], [330, 290], [400, 210]], { width: [6, 24, 8], color: P.blanco, taper: [0.4, 0.4] })
  circle(290, 470, 12, { fill: P.blanco })
})

// Tapas de quitina oscura
layer("tapas", () => {
  for (const [pts, arriba] of [[tapaArriba, true], [tapaAbajo, false]]) {
    const t = blob(pts, { fill: P.quitina })
    clip(t, () => {
      const b = bbox(t)
      // sombra de un tono a la derecha
      ellipse(b.cx + 120, b.cy, 110, 140, { fill: "#1a1724" })
      // segmentos de quitina, concéntricos al extremo
      const tip = arriba ? b.y : b.y + b.h
      const s = arriba ? 1 : -1
      for (const d of [48, 92]) {
        curve([[b.x - 10, tip + s * (d + 50)], [b.cx, tip + s * d], [b.x + b.w + 10, tip + s * (d + 50)]], { stroke: P.ink, width: 9, cap: "round" })
      }
    })
    const b = bbox(t)
    brush(arriba ? [[418, 172], [436, 124], [478, 96]] : [[414, 846], [424, 892], [456, 926]], { width: [4, 13, 4], color: P.blanco })
    blob(pts, { fill: "none", stroke: P.ink, width: 16 })
  }
})

// Ventana translúcida pálida con aparato atrapado
layer("ventana", () => {
  const wPts = ring(CX, CY + 10, 150, 178, 10)
  blob(grow(wPts, 1.16), { fill: P.ambarOsc })
  blob(grow(wPts, 1.12), { fill: P.ambar })
  brush(arc(CX, CY + 10, 170, 220, 330, 8).map(p => [p[0], p[1] * 1.02]), { width: [3, 10, 3], color: P.sedaClara })
  const win = blob(wPts, { fill: P.ventana })
  clip(win, () => {
    // silueta del aparato (engranaje + núcleo + antenas)
    const g = P.ventanaSom
    repeat_radial(CX, CY + 10, 8, (i) => rect(CX - 16, CY - 95, 32, 40, { fill: g, r: 6 }))
    circle(CX, CY + 10, 80, { fill: g })
    circle(CX, CY + 10, 40, { fill: P.ventana })
    circle(CX, CY + 10, 18, { fill: g })
    rect(CX - 120, CY + 2, 60, 16, { fill: g, r: 6 })
    rect(CX + 60, CY + 2, 60, 16, { fill: g, r: 6 })
    // sombra interior de un tono abajo-derecha
    blob(shift(wPts, 34, 40), { fill: "#00000000", stroke: alpha("#8c907f", 0.5), width: 70, blend: "multiply" })
  })
  blob(wPts, { fill: "none", stroke: P.ink, width: 14 })
  brush([[410, 470], [430, 400], [480, 360]], { width: [5, 16, 5], color: P.blanco })
  circle(408, 510, 8, { fill: P.blanco })
})

// Contorno de tinta con peso
layer("tinta", () => {
  ink(blob(cuerpoPts, { hidden: true }), { width: 10, weight: 2.2, color: P.ink })
})
