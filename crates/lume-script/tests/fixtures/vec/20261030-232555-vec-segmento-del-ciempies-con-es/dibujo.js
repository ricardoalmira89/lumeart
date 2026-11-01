// dibujo.js: segmento de ciempiés visto desde arriba, avanza hacia ABAJO.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  tinta: "#102a17",     // contorno oscuro
  verde: "#236f2c",     // cuerpo y patas
  verdeOsc: "#174a22",  // sombra de un tono
  lima: "#8fe03c",      // brillos verdes
  limaClaro: "#e4ff9a", // brillo más fuerte
  violeta: "#7b3fcc",   // caparazón
  violetaOsc: "#53279a",
  lavanda: "#d2a8ff",
  magenta: "#ec1fae",   // perla
  blanco: "#ffffff",
}
setLight(-1, -1)

const CX = 517, CY = 517, R = 255  // cuerpo
const BORDE = 34                   // grosor del borde blanco de pegatina

// Pata de la izquierda de la pantalla; dy la baja (0 = trasera, 160 = delantera).
// La base se mete bien dentro del cuerpo y acaba redondeada en la articulación.
function legPts(dy) {
  return shift([
    [380, 395], [340, 362], [280, 356], [215, 366], [158, 392], [120, 432],
    [100, 480], [98, 518], [114, 532, 0.5], [132, 520], [158, 484], [195, 462],
    [240, 462], [300, 474], [362, 470], [392, 434],
  ], 0, dy)
}
// Refleja unos puntos respecto al eje del cuerpo (x pasa a 2·CX − x), conservando la suavidad.
const mx = (pts) => pts.map(([x, y, ...k]) => [2 * CX - x, y, ...k])

// Dibuja una pata; mirrorX = true la coloca en el otro lado con sus puntos ya reflejados
// (sin flip, para que el esqueleto se exporte bien a Godot).
function leg(dy, mirrorX, dark) {
  const m = mirrorX ? mx : (p) => p
  const pts = m(legPts(dy))
  blob(pts, { fill: P.blanco, stroke: P.blanco, width: BORDE + 22, behind: true })
  const base = mix(P.verde, P.verdeOsc, dark)
  const s = blob(pts, { fill: base })
  clip(s, () => {
    // sombra de un tono en la cara inferior
    blob(m(shift([[90, 470], [150, 430], [230, 420], [330, 440], [420, 470], [420, 520], [90, 560]], 0, dy)),
      { fill: mix(P.verdeOsc, "#0e3016", dark) })
    // brillo alargado en el lomo
    brush(m(shift([[178, 404], [212, 400], [246, 402]], 0, dy)), { width: 12, taper: [0.5, 0.5], color: P.lima })
  })
  // la tinta reflejada se calcula con la luz reflejada, igual que salía con flip
  if (mirrorX) setLight(1, -1)
  ink(s, { width: 13, weight: 1.6, color: P.tinta })
  if (mirrorX) setLight(-1, -1)
}

rig({ origin: [CX, CY] })
bone("cuerpo", { at: [CX, CY] })
// Lado anatómico: el segmento mira hacia abajo, así que su izquierda es la derecha de la pantalla.
bone("pata_tras_der", { parent: "cuerpo", at: [340, 415] })
bone("pata_del_der", { parent: "cuerpo", at: [340, 575] })
bone("pata_tras_izq", { parent: "cuerpo", at: [2 * CX - 340, 415] })
bone("pata_del_izq", { parent: "cuerpo", at: [2 * CX - 340, 575] })
bone("perla", { parent: "cuerpo", at: [CX, 327] })

// Borde blanco del cuerpo (detrás de las patas, para que no corte sus bases)
part("cuerpo", () => {
  circle(CX, CY, R, { fill: P.blanco, stroke: P.blanco, width: BORDE * 2 })
})

// Patas (detrás del cuerpo)
part("pata_tras_der", () => leg(0, false, 0))
part("pata_del_der", () => leg(160, false, 0))
part("pata_tras_izq", () => leg(0, true, 0))
part("pata_del_izq", () => leg(160, true, 0))

// Cuerpo, caparazón y brillos
part("cuerpo", () => {
  // cuerpo redondo: sombra de un tono abajo a la derecha
  const body = circle(CX, CY, R, { fill: P.verdeOsc })
  clip(body, () => {
    circle(CX - 26, CY - 30, R - 14, { fill: P.verde })
    // brillo grande en media luna arriba a la izquierda
    blob([[282, 478, 0.5], [296, 412], [340, 352], [410, 306], [505, 286, 0], [450, 306],
      [385, 348], [334, 410], [312, 478], [296, 496]], { fill: P.lima })
    blob([[312, 420, 0], [340, 372], [380, 338], [428, 320, 0], [390, 345], [352, 380]],
      { fill: P.limaClaro, opacity: 0.85 })
  })
  circle(293, 546, 13, { fill: P.limaClaro })
  ink(body, { width: 15, weight: 1.6, color: P.tinta })

  // caparazón violeta con su anillo oscuro
  const shellPts = [[517, 372], [600, 388], [640, 425, 0.5], [656, 520], [645, 570, 0.5], [595, 640],
    [517, 662, 0.5], [440, 640, 0.5], [380, 555, 0.5], [372, 490], [394, 410, 0.5], [450, 382]]
  blob(shellPts, { fill: P.tinta, stroke: "#123c1c", width: 26, behind: true })
  const shell = blob(shellPts, { fill: P.violeta })
  clip(shell, () => {
    blob([[640, 400], [700, 520], [640, 700], [480, 700], [520, 650], [590, 620], [640, 540], [645, 450]],
      { fill: P.violetaOsc })
    brush([[400, 515], [408, 448], [440, 412], [480, 400]], { width: [6, 16, 10], taper: [0.5, 0.6], color: P.lavanda })
    // surco central entre las dos alas
    rect(509, 360, 16, 320, { fill: "#3e1d78" })
  })
  ink(shell, { width: 10, weight: 1.5, color: "#2a1250" })
})

// Perla magenta
part("perla", () => {
  circle(CX, 327, 38, { fill: P.tinta })
  const pearl = circle(CX, 327, 28, { fill: radial(507, 315, 34, ["#ff6fd8", P.magenta, "#a80f7a"]) })
  ink(pearl, { width: 3, weight: 1.5, color: "#5a0640" })
  circle(507, 317, 9, { fill: P.blanco })
})

// Acciones. Patas de la pantalla izquierda (_der): ángulo negativo = punta hacia abajo (adelante).
// Patas de la pantalla derecha (_izq): ángulo positivo = punta hacia abajo (adelante).
const A = 18
const pasoA = { pata_del_izq: A, pata_tras_der: -A, pata_del_der: A, pata_tras_izq: -A }
const pasoB = { pata_del_izq: -A, pata_tras_der: A, pata_del_der: -A, pata_tras_izq: A }
action("andar", { duration: 0.5, loop: true, move: 496 }, [
  [0, { ...pasoA, cuerpo: 1.5, perla: { y: 0 } }],
  [0.25, { cuerpo: 0, perla: { y: -7 } }],
  [0.5, { ...pasoB, cuerpo: -1.5, perla: { y: 0 } }],
  [0.75, { cuerpo: 0, perla: { y: -7 } }],
  [1, { ...pasoA, cuerpo: 1.5, perla: { y: 0 } }],
])

// Golpe: las cuatro patas se recogen hacia el cuerpo y vuelven.
const reposo = { pata_del_izq: 0, pata_tras_der: 0, pata_del_der: 0, pata_tras_izq: 0, perla: { y: 0 } }
action("golpe", { duration: 0.25, loop: false }, [
  [0, reposo],
  [0.3, {
    pata_tras_der: { rot: 22, x: 70 }, pata_del_der: { rot: -22, x: 70 },
    pata_tras_izq: { rot: -22, x: -70 }, pata_del_izq: { rot: 22, x: -70 },
    perla: { y: 6 },
  }],
  [1, reposo],
])
