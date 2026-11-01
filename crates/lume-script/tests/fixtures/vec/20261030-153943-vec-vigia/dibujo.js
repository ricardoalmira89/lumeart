// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Vigía: torreta-ojo flotante. Luz desde arriba a la izquierda.
setLight(-1, -1)
const P = {
  ink: "#2a0f45",
  body: "#8b3fd6",
  bodyLight: "#c088f5",
  bodyShadow: "#5b2499",
  spike: "#4b1b80",
  spikeLight: "#7a3cc0",
  gap: "#1c0830",
  sclera: "#fff4da",
  scleraShadow: "#e6c294",
  iris: "#ffc928",
  irisDark: "#e27300",
  pupil: "#1a0722",
  white: "#ffffff",
}
const C = [512, 512], R = 350

// Púas en las diagonales
function spikePts(deg) {
  const a = polar(C[0], C[1], R - 30, deg - 12)
  const b = polar(C[0], C[1], R - 30, deg + 12)
  const t = polar(C[0], C[1], 480, deg)
  const m1 = polar(C[0], C[1], 400, deg - 5)
  const m2 = polar(C[0], C[1], 400, deg + 5)
  return [[...a, 0], [...m1, 0.5], [...t, 0], [...m2, 0.5], [...b, 0]]
}
const DIAG = [45, 135, 225, 315]
const EYE = "M 196 520 Q 512 250 828 520 Q 512 720 196 520 Z"

layer("borde-pegatina", () => {
  for (const d of DIAG) blob(spikePts(d), { fill: P.white, stroke: P.white, width: 70, join: "round" })
  circle(C[0], C[1], R, { fill: P.white, stroke: P.white, width: 70 })
  for (const d of DIAG) blob(spikePts(d), { fill: P.ink, stroke: P.ink, width: 26, join: "round" })
  circle(C[0], C[1], R, { fill: P.ink, stroke: P.ink, width: 26 })
})

layer("puas", () => {
  for (const d of DIAG) {
    const s = blob(spikePts(d), { fill: P.spike })
    clip(s, () => {
      // cara iluminada de la púa (lado hacia arriba-izquierda)
      const t = polar(C[0], C[1], 480, d)
      const a = polar(C[0], C[1], R - 30, d - 12)
      const b = polar(C[0], C[1], R - 30, d + 12)
      const lit = (d === 225 || d === 315) ? a : b
      poly([C, t, lit], { fill: P.spikeLight })
    })
  }
})

layer("cuerpo", () => {
  const body = circle(C[0], C[1], R, { fill: P.body })
  clip(body, () => {
    // sombra de un tono abajo a la derecha y luz arriba a la izquierda
    const sh = circle(C[0], C[1], R + 5, { fill: P.bodyShadow })
    cut(sh, [circle(C[0] - 55, C[1] - 60, R, { hidden: true })])
    const li = circle(C[0] - 40, C[1] - 45, R - 20, { fill: P.bodyLight })
    cut(li, [circle(C[0] + 40, C[1] + 40, R - 20, { hidden: true })])
  })
  // costuras de los párpados (ecuador, fuera del ojo)
  brush([[166, 515], [200, 520]], { width: 14, taper: [0, 0.5], color: P.ink })
  brush([[858, 515], [824, 520]], { width: 14, taper: [0, 0.5], color: P.ink })
})

layer("ojo", () => {
  path(EYE, { fill: P.gap, stroke: P.gap, width: 30, join: "round" })
  const eye = path(EYE, { fill: P.sclera })
  clip(eye, () => {
    ellipse(512, 640, 360, 80, { fill: P.scleraShadow })
    ellipse(512, 400, 330, 70, { fill: P.scleraShadow })
    // iris mirando hacia abajo
    const iris = circle(512, 575, 128, { fill: P.iris })
    freeform(iris, [
      [470, 520, "#fff27a", 0.3],
      [512, 580, P.iris, 0.4],
      [560, 680, P.irisDark, 0.3],
    ])
    circle(512, 575, 128, { fill: "none", stroke: P.irisDark, width: 16 })
    blob([[512, 470, 0], [548, 560], [512, 690, 0], [476, 560]], { fill: P.pupil })
    // brillos
    ellipse(458, 512, 34, 24, { fill: P.white, rotate: -30 })
    circle(570, 630, 12, { fill: P.white, opacity: 0.9 })
    // sombra del párpado superior sobre el ojo
    path("M 150 480 Q 512 250 874 480 L 874 300 L 150 300 Z", { fill: P.ink, opacity: 0.35 })
  })
})

layer("labios-parpados", () => {
  // labio del párpado superior y del inferior: banda clara con tinta
  curve([[205, 515], [360, 418], [512, 388], [664, 418], [819, 515]], { stroke: P.bodyLight, width: 22, cap: "round" })
  brush([[196, 520], [360, 412], [512, 382], [664, 412], [828, 520]], { width: [8, 18, 22, 18, 8], taper: [0.1, 0.1], color: P.ink })
  brush([[196, 520], [360, 600], [512, 620], [664, 600], [828, 520]], { width: [8, 16, 20, 16, 8], taper: [0.1, 0.1], color: P.ink })
  curve([[260, 570], [400, 628], [512, 638], [624, 628], [764, 570]], { stroke: P.bodyLight, width: 10, cap: "round", opacity: 0.8 })
})

layer("brillos", () => {
  brush([[250, 380], [300, 290], [390, 220]], { width: [10, 30, 12], taper: [0.4, 0.4], color: P.white })
  circle(440, 205, 16, { fill: P.white })
  brush([[300, 700], [360, 760]], { width: [6, 14, 6], color: P.white, taper: [0.4, 0.4] })
})
