// dibujo.js: torreta orgánica de la Colmena, vista cenital, apuntando hacia arriba.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  linea: "#1c0b28",     // contorno oscuro
  morado: "#8a3fd0",    // quitina de la base
  moradoOsc: "#4a1a78",
  moradoClaro: "#c58cff",
  verde: "#5fd03a",     // quitina del cañón
  verdeOsc: "#1f5e2c",  // anillo
  magenta: "#ff2fd0",   // boca encendida
  amarillo: "#ffd83a",  // ojo
  blanco: "#ffffff",
}
setLight(-1, -1)

const C = [512, 512]          // centro de la base (pivote de giro)
const RB = 300                // radio exterior de la base
const RA = 248, RD = 165      // anillo verde y cúpula interior

// Cañón: aguijón corto y grueso, simétrico
const half = [[512, 470], [366, 440], [356, 320], [384, 200], [404, 118], [400, 72, 0.5], [448, 36]]
const canonPts = [...half, ...half.slice(1).reverse().map(([x, y, s]) => s === undefined ? [1024 - x, y] : [1024 - x, y, s])]

layer("pegatina", () => {
  // borde blanco grueso detrás de toda la silueta
  circle(C[0], C[1], RB, { fill: P.blanco, stroke: P.blanco, width: 70 })
  blob(canonPts, { fill: P.blanco, stroke: P.blanco, width: 70, join: "round" })
  // contorno oscuro
  circle(C[0], C[1], RB, { fill: P.linea, stroke: P.linea, width: 30 })
  blob(canonPts, { fill: P.linea, stroke: P.linea, width: 30, join: "round" })
})

layer("base", () => {
  const base = circle(C[0], C[1], RB, { fill: P.morado })
  clip(base, () => {
    // sombra de un tono abajo a la derecha
    const luz = circle(C[0] - 55, C[1] - 55, RB + 5, { hidden: true })
    rect(0, 0, 1024, 1024, { fill: P.moradoOsc })
    clip(luz, () => circle(C[0], C[1], RB, { fill: P.morado }))
    // placas: juntas radiales
    for (let i = 0; i < 10; i++) {
      const a = i * 36 + 18
      const p1 = polar(C[0], C[1], RA + 6, a), p2 = polar(C[0], C[1], RB + 10, a)
      brush([p1, p2], { width: 16, taper: [0, 0.5], color: P.linea })
    }
    // brillo del borde superior izquierdo
    brush(arc(C[0], C[1], RB - 26, -175, -125, 10), { width: 20, taper: [0.3, 0.3], color: P.moradoClaro })
    brush(arc(C[0], C[1], RB - 26, 150, 170, 6), { width: 14, taper: [0.3, 0.3], color: P.moradoClaro })
  })
})

layer("anillo", () => {
  circle(C[0], C[1], RA, { fill: P.verdeOsc, stroke: P.linea, width: 18 })
  clip(circle(C[0], C[1], RA, { hidden: true }), () => {
    circle(C[0] + 30, C[1] + 30, RA, { fill: "#143f1e" })
    circle(C[0] - 12, C[1] - 12, RA - 10, { fill: P.verdeOsc })
  })
  brush(arc(C[0], C[1], RA - 30, 175, 235, 10), { width: 14, color: "#4fae55" })
})

layer("cañon", () => {
  const canon = blob(canonPts, { fill: P.verde })
  clip(canon, () => {
    // sombra de un tono a la derecha
    blob([[560, 30], [680, 30], [690, 480], [590, 480], [610, 320], [590, 150, 0.5]], { fill: "#2f8a35" })
    // segmentos de quitina
    for (const [y, w] of [[150, 100], [235, 122], [320, 140]]) {
      brush([[512 - w, y + 10], [512, y - 12], [512 + w, y + 10]], { width: 18, taper: [0.15, 0.15], color: P.linea })
    }
    // brillos blancos marcados a la izquierda
    brush([[436, 165], [420, 215]], { width: 22, taper: [0.4, 0.4], color: P.blanco })
    brush([[406, 255], [396, 300]], { width: 24, taper: [0.4, 0.4], color: P.blanco })
  })
  ink(canon, { width: 14, weight: 2, color: P.linea })
  // boca encendida
  ellipse(512, 74, 80, 30, { fill: P.linea, stroke: P.linea, width: 10 })
  ellipse(512, 75, 70, 25, { fill: "#8a1270" })
  ellipse(512, 76, 60, 19, { fill: P.magenta })
  ellipse(504, 73, 30, 8, { fill: "#ffc4f2" })
})

layer("cupula", () => {
  const dome = circle(C[0], C[1], RD, { fill: P.morado, stroke: P.linea, width: 22 })
  clip(dome, () => {
    rect(0, 0, 1024, 1024, { fill: P.moradoOsc })
    circle(C[0] - 28, C[1] - 28, RD - 5, { fill: P.morado })
    brush(arc(C[0], C[1], RD - 30, 185, 250, 10), { width: 22, color: P.moradoClaro })
  })
  // ojo
  circle(C[0], C[1], 54, { fill: P.amarillo, stroke: P.linea, width: 16 })
  clip(circle(C[0], C[1], 54, { hidden: true }), () => {
    circle(C[0] + 18, C[1] + 18, 54, { fill: "#e09a10" })
    circle(C[0] - 4, C[1] - 4, 48, { fill: P.amarillo })
  })
  ellipse(C[0], C[1], 12, 36, { fill: P.linea })
  circle(C[0] - 20, C[1] - 22, 11, { fill: P.blanco })
})
