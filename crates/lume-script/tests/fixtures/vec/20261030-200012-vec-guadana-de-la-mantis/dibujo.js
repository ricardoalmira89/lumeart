// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 720x1280: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 720, height: 1280, background: null }) // null = fondo transparente (objeto suelto)
setLight(-1, -1)

const P = {
  ink: "#16260f",      // contorno oscuro
  lima: "#9be23a",     // quitina clara
  limaSh: "#4f9a2a",   // sombra de la quitina clara
  verde: "#2f6e24",    // quitina oscura (placas)
  verdeSh: "#1e4a1a",
  morado: "#9a45e0",   // espinas y hombro
  moradoSh: "#5a2396",
  hueso: "#f4ead0",    // filo
  ambar: "#ffb52e",    // brillo del filo
  blanco: "#ffffff",
}

// Siluetas (puntos)
const S = {
  femur: [[340, 104], [382, 104], [400, 230], [400, 430], [392, 600], [360, 632], [328, 600], [320, 430], [320, 230]],
  tibia: [[328, 648], [394, 648], [404, 760], [398, 880], [360, 910], [324, 882], [318, 760]],
  hoja: [[344, 870], [316, 980], [318, 1085], [348, 1172], [408, 1232], [492, 1242], [574, 1206], [644, 1108, 0],
    [580, 1150], [506, 1172], [446, 1152], [410, 1092], [398, 990], [396, 880]],
}
// Espinas del borde interior (derecho): base en el borde, punta hacia abajo-derecha
const espinas = []
for (const [y, x, l] of [[300, 400, 44], [410, 401, 52], [520, 397, 56], [690, 397, 50], [790, 403, 62], [870, 398, 46]]) {
  espinas.push([[x - 8, y - 26], [x + l, y + 22, 0], [x - 6, y + 18]])
}

// Borde blanco y contorno de pegatina
layer("pegatina", () => {
  const silueta = (st) => {
    blob(S.femur, st); blob(S.tibia, st); blob(S.hoja, st)
    circle(360, 640, 36, st); circle(360, 74, 44, st)
    for (const e of espinas) blob(e, st)
  }
  silueta({ fill: P.blanco, stroke: P.blanco, width: 56, join: "round" })
  silueta({ fill: P.ink, stroke: P.ink, width: 26, join: "round" })
})

// Pieza de quitina: base, sombra de un tono a la derecha y brillo a la izquierda
function pieza(pts, base, sombra, dx) {
  const f = blob(pts, { fill: base })
  clip(f, () => {
    blob(shift(pts, dx, 0), { fill: base })
    blob(pts, { fill: sombra })
    blob(shift(pts, -dx, 0), { fill: base })
  })
  return f
}

layer("brazo", () => {
  const fe = pieza(S.femur, P.lima, P.limaSh, 0)
  clip(fe, () => {
    // sombra de un tono en el lado derecho
    blob([[372, 80], [430, 80], [430, 640], [372, 640], [384, 400]], { fill: P.limaSh })
    // placas oscuras de quitina
    for (const y of [180, 330, 480]) brush([[322, y], [360, y + 14], [400, y]], { width: 14, taper: [0, 0], color: P.verde })
    // brillo
    brush([[338, 120], [334, 300], [336, 560]], { width: [4, 14, 6], color: P.blanco })
  })
  const codo = circle(360, 640, 36, { fill: P.verde })
  clip(codo, () => {
    circle(372, 652, 30, { fill: P.verdeSh })
    circle(348, 626, 10, { fill: alpha(P.blanco, 0.9) })
  })
  const ti = blob(S.tibia, { fill: P.lima })
  clip(ti, () => {
    blob([[376, 640], [420, 640], [420, 920], [370, 920], [384, 780]], { fill: P.limaSh })
    brush([[320, 790], [360, 806], [404, 790]], { width: 14, taper: [0, 0], color: P.verde })
    brush([[336, 680], [332, 760], [336, 860]], { width: [4, 12, 5], color: P.blanco })
  })
})

layer("espinas", () => {
  for (const e of espinas) {
    const s = blob(e, { fill: P.morado })
    clip(s, () => {
      poly([[e[0][0] - 10, e[0][1] + 24], [e[1][0] + 20, e[1][1] - 4], [e[2][0], e[2][1] + 20]], { fill: P.moradoSh })
      brush([[e[0][0] + 6, e[0][1] + 8], [e[1][0] - 14, e[1][1] - 12]], { width: [5, 3], color: alpha(P.blanco, 0.8) })
    })
  }
})

layer("hoja", () => {
  const h = blob(S.hoja, { fill: P.verde })
  clip(h, () => {
    // lado exterior (izquierdo-abajo) en sombra suave
    blob(shift(S.hoja, 26, -20), { fill: P.verdeSh })
    blob(shift(S.hoja, 32, -26), { fill: P.verde })
    // filo color hueso en el borde interior (cóncavo)
    const filo = [[404, 880], [408, 990], [420, 1090], [452, 1150], [508, 1170], [584, 1148], [646, 1104]]
    brush(shift(filo, -4, 4), { width: [50, 76, 80, 66, 40], taper: [0.05, 0.45], color: P.hueso })
    // brillo ámbar a lo largo del filo
    brush(shift(filo, -30, 22), { width: [6, 16, 16, 10], taper: [0.2, 0.5], color: P.ambar })
    brush(shift(filo, -6, 2), { width: [3, 8, 8, 4], taper: [0.2, 0.5], color: P.blanco })
    // brillo blanco en el lomo
    brush([[336, 940], [334, 1060], [360, 1150]], { width: [4, 12, 4], color: alpha(P.blanco, 0.85) })
  })
})

layer("hombro", () => {
  const b = circle(360, 74, 44, { fill: P.morado })
  clip(b, () => {
    circle(376, 90, 42, { fill: P.moradoSh })
    circle(345, 57, 13, { fill: P.blanco })
  })
})
