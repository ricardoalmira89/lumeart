// dibujo.js: nave de refugiados (fondo), vista cenital, morro hacia arriba.
canvas({ width: 1024, height: 1024, background: null })
seed(7)

const P = {
  hueso: "#c4bca8",
  huesoLuz: "#d3cbb6",
  sombra: "#8e897f",
  sombraFria: "#7f8288",
  ocre: "#a68a5e",
  ocreOsc: "#8a7452",
  metal: "#77746f",
  oscuro: "#5f5e60",
  calido: "#dcb67c",
  azul: "#a9c9d8",
  azulOsc: "#6f8fa3",
}
const CX = 512
setLight(-1, -1)

// Pieza con volumen: base + sombra al lado derecho/abajo + luz arriba-izquierda, todo recortado
function panel(s, base) {
  const b = bbox(s)
  clip(s, () => {
    rect(b.x + b.w * 0.55, b.y - 10, b.w, b.h + 20, { fill: P.sombraFria, opacity: 0.5, blur: b.w * 0.15, blend: "multiply" })
    rect(b.x - 20, b.y + b.h * 0.8, b.w + 40, b.h, { fill: P.sombraFria, opacity: 0.25, blur: b.h * 0.15, blend: "multiply" })
    rect(b.x - b.w * 0.2, b.y - 10, b.w * 0.45, b.h + 20, { fill: P.huesoLuz, opacity: 0.35, blur: b.w * 0.15, blend: "screen" })
  })
}

layer("sombra", () => {
  // halo de sombra muy suave para separar la silueta del fondo sin borde
  group({ move: [8, 10], opacity: 0.12 }, () => {
    blob([[CX, 50], [CX + 60, 150], [CX + 145, 260], [CX + 145, 900], [CX + 80, 980], [CX - 80, 980], [CX - 145, 900], [CX - 145, 260], [CX - 60, 150]], { fill: "#2a2a35", blur: 8 })
  })
})

layer("columna", () => {
  // espina central que une todo
  const s = rect(CX - 42, 180, 84, 640, { fill: P.metal })
  panel(s, P.metal)
  clip(s, () => {
    for (let y = 200; y < 810; y += 26) rect(CX - 42, y, 84, 3, { fill: P.oscuro, opacity: 0.35 })
  })
})

layer("motores", () => {
  // góndolas
  for (const dx of [-62, 62]) {
    const x = CX + dx
    const g = blob([[x - 40, 860, 0.5], [x + 40, 860, 0.5], [x + 44, 950, 0.5], [x + 34, 985, 0], [x - 34, 985, 0], [x - 44, 950, 0.5]], { fill: P.sombra })
    panel(g, P.sombra)
    clip(g, () => {
      rect(x - 50, 930, 100, 8, { fill: P.oscuro, opacity: 0.45 })
      rect(x - 50, 900, 100, 5, { fill: P.ocreOsc, opacity: 0.5 })
    })
    // tobera
    ellipse(x, 984, 32, 12, { fill: P.oscuro })
    ellipse(x, 986, 24, 8, { fill: P.azulOsc })
    // brillo azul claro, suave
    element(dx < 0 ? "brillo_motor_izq" : "brillo_motor_der", { pivot: [x, 986] }, () => {
      ellipse(x, 994, 30, 22, { fill: P.azul, opacity: 0.35, blur: 12 })
      ellipse(x, 988, 18, 9, { fill: P.azul, opacity: 0.8, blur: 3 })
      ellipse(x, 987, 9, 4, { fill: "#d6e6ec", opacity: 0.8, blur: 1.5 })
    })
  }
  // bloque motor
  const m = blob([[CX - 135, 790, 0.4], [CX + 135, 790, 0.4], [CX + 140, 870, 0.5], [CX + 105, 905, 0], [CX - 105, 905, 0], [CX - 140, 870, 0.5]], { fill: P.hueso })
  panel(m, P.hueso)
  clip(m, () => {
    rect(CX - 150, 820, 300, 18, { fill: P.ocre, opacity: 0.85 })
    rect(CX - 150, 838, 300, 3, { fill: P.ocreOsc, opacity: 0.5 })
    // rejillas
    for (let i = 0; i < 6; i++) rect(CX - 60 + i * 20, 858, 10, 30, { fill: P.metal, opacity: 0.45, r: 3 })
    grain({ opacity: 0.08, size: 1.5 })
  })
})

layer("modulos", () => {
  // cuatro contenedores de carga enganchados en fila
  const tops = [250, 382, 514, 646]
  tops.forEach((y, i) => {
    const w = 296 - (i === 0 ? 24 : 0)
    const h = 122
    const x = CX - w / 2
    // enganche
    rect(CX - 24, y - 14, 48, 18, { fill: P.oscuro, r: 3 })
    const s = rect(x, y, w, h, { fill: P.hueso, r: 10 })
    clip(s, () => {
      // franja ocre (alternando posición)
      const fy = i % 2 ? y + h - 30 : y + 12
      rect(x, fy, w, 18, { fill: P.ocre, opacity: 0.85 })
      // costillas
      for (let k = 1; k < 6; k++) rect(x + (w * k) / 6 - 2, y, 4, h, { fill: P.sombra, opacity: 0.35 })
      // escotilla en el lado izquierdo de algunos
      if (i !== 2) rect(x + 22, y + 46, 34, 30, { fill: P.sombra, opacity: 0.55, r: 3 })
      // desgaste
      for (let k = 0; k < 7; k++) ellipse(rand(x, x + w), rand(y, y + h), rand(8, 26), rand(4, 10), { fill: P.ocreOsc, opacity: rand(0.08, 0.18), blur: 4 })
      // panel que falta / parche
      if (i === 1) rect(x + w - 80, y + 20, 48, 40, { fill: "#b6ae9a", r: 2 })
      grain({ opacity: 0.08, size: 1.5 })
    })
    panel(s, P.hueso)
  })
})

layer("cabina", () => {
  // cuello
  const n = rect(CX - 70, 200, 140, 60, { fill: P.sombra, r: 8 })
  panel(n, P.sombra)
  // morro
  const c = blob([[CX, 40], [CX + 52, 70, 0.6], [CX + 92, 140], [CX + 100, 215, 0.5], [CX + 80, 235, 0], [CX - 80, 235, 0], [CX - 100, 215, 0.5], [CX - 92, 140], [CX - 52, 70, 0.6]], { fill: P.hueso })
  freeform(c, [[CX - 45, 100, P.huesoLuz, 0.5], [CX + 95, 200, P.sombra, 0.1], [CX + 40, 120, P.hueso, 0.5], [CX - 70, 215, P.hueso, 0.4], [CX + 10, 210, P.hueso, 0.4]])
  clip(c, () => {
    // franjas ocres en V
    brush([[CX - 100, 205], [CX, 175], [CX + 100, 205]], { width: 16, taper: [0, 0], color: P.ocre })
    // ventanillas cálidas
    blob([[CX - 34, 94, 0.5], [CX + 34, 94, 0.5], [CX + 42, 118, 0.5], [CX - 42, 118, 0.5]], { fill: P.oscuro })
    for (let i = 0; i < 4; i++) {
      const wx = CX - 30 + i * 20
      rect(wx - 6, 99, 13, 14, { fill: P.calido, r: 3, opacity: 0.9 })
    }
    ellipse(CX, 106, 50, 18, { fill: P.calido, opacity: 0.2, blur: 10 })
    // fila lateral de ojos de buey
    for (let k = 0; k < 4; k++) {
      circle(CX - 66 + k * 2, 140 + k * 18, 4, { fill: P.calido, opacity: 0.7 })
      circle(CX + 66 - k * 2, 140 + k * 18, 4, { fill: P.calido, opacity: 0.45 })
    }
    grain({ opacity: 0.08, size: 1.5 })
  })
})

layer("detalles", () => {
  // antena y contenedor apilado (carga improvisada)
  rect(CX + 30, 520, 50, 36, { fill: P.ocreOsc, r: 4, opacity: 0.9 })
  rect(CX + 30, 520, 50, 8, { fill: P.ocre, r: 3 })
  rect(CX - 95, 680, 60, 40, { fill: "#a7a08f", r: 4 })
  rect(CX - 95, 680, 20, 40, { fill: P.huesoLuz, opacity: 0.4, r: 4 })
  line(CX - 20, 60, CX - 20, 30, { stroke: P.metal, width: 3 })
  circle(CX - 20, 30, 3, { fill: P.calido, opacity: 0.6 })
})
