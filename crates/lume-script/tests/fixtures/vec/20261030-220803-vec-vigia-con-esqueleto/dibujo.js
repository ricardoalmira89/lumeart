// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Paleta de la referencia
const P = {
  tinta: "#270e40",    // contorno oscuro
  osc: "#5b2499",      // sombra de la esfera
  base: "#8a3fd4",     // violeta medio
  luz: "#bf87f4",      // violeta claro (borde de luz)
  puaLuz: "#7b3dc6",   // cara clara de las púas
  puaOsc: "#4b1b80",   // cara oscura de las púas
  crema: "#fff4da",    // esclerótica
  tostado: "#e6c292",  // sombra de la esclerótica
  naranja: "#e27300",  // aro del iris
  amarillo: "#fdd447", // iris
  pupila: "#1d0a2a",
  blanco: "#ffffff",
}

const C = [512, 512]  // centro de la esfera
const R = 350         // radio del relleno de la esfera
const BORDE = 27      // borde blanco de pegatina

// Ojo: almendra entre las esquinas (195, 515) y (830, 515)
const ECX = 512.5, EHW = 317.5, EY = 515
const u2 = (x) => ((x - ECX) / EHW) ** 2
const topY = (x) => EY - 131 * Math.max(0, 1 - u2(x))
const botY = (x) => EY + 106 * Math.max(0, 1 - u2(x))
const xsOjo = (a, b, n) => range(n + 1).map((i) => lerp(a, b, i / n))
const supEdge = (a = 195, b = 830, dy = 0) => xsOjo(a, b, 40).map((x) => [x, topY(x) + dy])
const infEdge = (a = 195, b = 830, dy = 0) => xsOjo(a, b, 40).map((x) => [x, botY(x) + dy])
const esferaTop = (x) => C[1] - Math.sqrt(Math.max(0, R * R - (x - C[0]) ** 2))
const esferaBot = (x) => C[1] + Math.sqrt(Math.max(0, R * R - (x - C[0]) ** 2))

// Púas: diagonales; luz en la mitad cuya cara mira a la izquierda
const PUAS = [
  { n: "pua_1", ang: 225 }, // arriba izquierda
  { n: "pua_2", ang: 315 }, // arriba derecha
  { n: "pua_3", ang: 45 },  // abajo derecha
  { n: "pua_4", ang: 135 }, // abajo izquierda
].map((p) => {
  const a = (p.ang * Math.PI) / 180
  const u = [Math.cos(a), Math.sin(a)]
  const v = [-u[1], u[0]]
  const at = (d, s) => [C[0] + u[0] * d + v[0] * s, C[1] + u[1] * d + v[1] * s]
  return { ...p, u, v, at, pivot: at(340, 0) }
})

// ---------- Esqueleto ----------
rig({ origin: C })
bone("cuerpo", { at: C })
PUAS.forEach((p) => bone(p.n, { parent: "cuerpo", at: p.pivot }))
bone("iris", { parent: "cuerpo", at: [512, 575] })
bone("parpado_sup", { parent: "cuerpo", at: [512, 384] })
bone("parpado_inf", { parent: "cuerpo", at: [512, 621] })

// 1. Borde blanco de pegatina de la esfera (detrás de todo)
part("cuerpo", () => {
  circle(C[0], C[1], R + 7 + BORDE, { fill: P.blanco })
})

// 2. Púas (detrás de la esfera): la base se mete 90 unidades en la esfera
PUAS.forEach((p) => part(p.n, () => {
  const tip = p.at(486, 0), dentro = p.at(250, 0)
  const A = p.at(250, 112), B = p.at(250, -112)
  const luzEnA = p.v[0] < 0
  poly([tip, A, dentro, B], { fill: P.blanco, stroke: P.blanco, width: 12 + 2 * BORDE, join: "round" })
  poly([tip, A, dentro], { fill: luzEnA ? P.puaLuz : P.puaOsc })
  poly([tip, dentro, B], { fill: luzEnA ? P.puaOsc : P.puaLuz })
  // brillo fino en la arista de la cara clara
  const L = luzEnA ? 1 : -1
  brush([p.at(470, 6 * L), p.at(400, 38 * L), p.at(345, 62 * L)], { width: 5, color: alpha(P.luz, 0.8), taper: [0.6, 0.3] })
  poly([tip, A, dentro, B], { stroke: P.tinta, width: 13, join: "round" })
}))

// 3. Esclerótica entera (bajo párpados e iris)
part("cuerpo", () => {
  const scl = ellipse(512, 505, 345, 165, { fill: P.crema })
  clip(scl, () => {
    // sombra que dejan los párpados en el blanco del ojo
    poly([...xsOjo(170, 855, 30).map((x) => [x, 466 - 22 * u2(x)]), [855, 300], [170, 300]], { fill: P.tostado })
    poly([...xsOjo(170, 855, 30).map((x) => [x, 566 + 14 * u2(x)]), [855, 700], [170, 700]], { fill: P.tostado })
  })
})

// 4. Iris con pupila rasgada
part("iris", () => {
  circle(512, 575, 136, { fill: P.naranja })
  circle(512, 575, 121, { fill: radial(470, 515, 190, ["#fff58a", P.amarillo, [1, "#f2a21c"]]) })
  blob([[512, 468, 0], [536, 520], [549, 575], [536, 630], [512, 682, 0], [488, 630], [475, 575], [488, 520]], { fill: P.pupila })
  ellipse(458, 512, 33, 23, { fill: P.blanco, rotate: -22 })
  circle(560, 640, 6, { fill: alpha(P.blanco, 0.7) })
})

// 5. Párpados (se esconden bajo la superficie de la esfera)
// Cada párpado llega 150 unidades más allá de su borde (cubre el hueco aunque baje 125)
// y nunca se sale de la esfera, que lo tapa en reposo.
const xsTapa = xsOjo(192, 833, 40)
const tapaSup = [
  ...xsTapa.map((x) => [x, topY(x)]),
  ...xsTapa.slice().reverse().map((x) => [x, Math.max(topY(x) - 150, esferaTop(x) + 10)]),
]
const tapaInf = [
  ...xsTapa.map((x) => [x, botY(x)]),
  ...xsTapa.slice().reverse().map((x) => [x, Math.min(botY(x) + 150, esferaBot(x) - 10)]),
]

part("parpado_sup", () => {
  const t = poly(tapaSup, { fill: linear(0, 240, 0, 390, [shade(P.base, -0.15), P.base]) })
  clip(t, () => {
    brush(supEdge(215, 810, -15), { width: [3, 8, 3], color: P.luz, taper: [0.2, 0.2] })
    brush(supEdge(300, 640, -48), { width: [2, 6, 2], color: alpha(P.blanco, 0.35), taper: [0.4, 0.4] })
  })
  // labio claro del párpado, que asoma sobre el blanco del ojo
  brush(supEdge(200, 825, 15), { width: [2, 8, 2], color: P.luz, taper: [0.1, 0.1] })
  brush(supEdge(192, 833), { width: [8, 22, 8], color: P.tinta, taper: [0.08, 0.08] })
})
part("parpado_inf", () => {
  const t = poly(tapaInf, { fill: linear(0, 620, 0, 780, [P.base, shade(P.base, -0.15)]) })
  clip(t, () => {
    brush(infEdge(230, 795, 16), { width: [3, 9, 3], color: P.luz, taper: [0.2, 0.2] })
  })
  brush(infEdge(192, 833), { width: [8, 22, 8], color: P.tinta, taper: [0.08, 0.08] })
})

// 6. Superficie de la esfera con el hueco del ojo
// Hueco recorrido al revés que el círculo, para que quede perforado
const hueco = [...infEdge(), ...supEdge().reverse().slice(1)]
const dEsfera = `M ${C[0] - R},${C[1]} A ${R} ${R} 0 1 1 ${C[0] + R},${C[1]} A ${R} ${R} 0 1 1 ${C[0] - R},${C[1]} Z ` +
  "M " + hueco.map((q) => q[0].toFixed(1) + "," + q[1].toFixed(1)).join(" L ") + " Z"

part("cuerpo", () => {
  const sup = path(dEsfera, { hidden: true })
  clip(sup, () => {
    circle(C[0], C[1], R, { fill: P.osc })
    clip(circle(470, 452, 350, { hidden: true }), () => {
      circle(470, 452, 350, { fill: P.luz })
      circle(560, 560, 340, { fill: P.base })
    })
    // brillos
    circle(440, 205, 16, { fill: P.blanco })
    blob([[252, 378, 0], [268, 318], [305, 268], [390, 220, 0], [320, 278], [282, 330]], { fill: P.blanco })
    blob([[300, 700, 0], [335, 722], [360, 760, 0], [330, 742]], { fill: P.blanco })
    // borde de luz del hueco
    brush(infEdge(258, 767, 19), { width: 9, color: alpha(P.luz, 0.9), taper: [0, 0], cap: "round" })
  })
  // contornos
  ink(circle(C[0], C[1], R, { hidden: true }), { width: 13, weight: 1.6, color: P.tinta })
  brush(supEdge(), { width: [10, 24, 10], color: P.tinta, taper: [0.06, 0.06] })
  brush(infEdge(), { width: [10, 22, 10], color: P.tinta, taper: [0.06, 0.06] })
  brush([[156, 510], [176, 512], [198, 515]], { width: 10, color: P.tinta, taper: [0, 0.5] })
  brush([[868, 510], [848, 512], [827, 515]], { width: 10, color: P.tinta, taper: [0, 0.5] })
})

// ---------- Acciones ----------
// Párpados: 0 = abiertos del todo (escondidos bajo la esfera); sup +125 / inf -125 = cerrados.
const ENTORNA = { parpado_sup: { y: 62 }, parpado_inf: { y: -50 } }
const ABIERTO = { parpado_sup: { y: 0 }, parpado_inf: { y: 0 } }
const CERRADO = { parpado_sup: { y: 128 }, parpado_inf: { y: -126 } }
// Púas: giro en grados y desplazamiento hacia fuera por la diagonal
const puas = (rots, fuera = 0) => Object.fromEntries(PUAS.map((p, i) => [p.n,
  { rot: rots[i % rots.length], x: p.u[0] * fuera, y: p.u[1] * fuera }]))

action("vigilar", { duration: 2, loop: true }, [
  [0,    { ...ENTORNA, iris: { x: -38 }, ...puas([4, -4]), cuerpo: { y: 0 } }],
  [0.15, { iris: { x: -38 } }],
  [0.25, { ...puas([0, 0], 3) }],
  [0.5,  { ...ENTORNA, iris: { x: 0 }, ...puas([-4, 4]), cuerpo: { y: 3 } }],
  [0.65, { iris: { x: 38 } }],
  [0.75, { ...puas([0, 0], 3) }],
  [0.85, { iris: { x: 38 } }],
  [1,    { ...ENTORNA, iris: { x: -38 }, ...puas([4, -4]), cuerpo: { y: 0 } }],
])

action("aviso", { duration: 0.6, loop: false }, [
  [0,    { ...ENTORNA, iris: { x: 0 }, ...puas([0]), cuerpo: { y: 0 } }],
  [0.1,  { ...ABIERTO, cuerpo: { y: -8 } }],
  [0.3,  { ...puas([8, -8], 34), cuerpo: { y: 2 } }],
  [0.5,  { ...puas([4, -4], 24) }],
  [1,    { ...ABIERTO, iris: { x: 0 }, ...puas([4, -4], 26), cuerpo: { y: 0 } }],
])

action("disparo", { duration: 0.35, loop: false }, [
  [0,    { ...ABIERTO, iris: { x: 0 }, cuerpo: { y: 0 }, ...puas([0]) }],
  [0.2,  { cuerpo: { y: -34 }, ...puas([7, -7], 10) }],
  [0.4,  { ...puas([-6, 6], 4) }],
  [0.55, { cuerpo: { y: 6 }, ...puas([5, -5], 6) }],
  [0.75, { ...puas([-3, 3], 2) }],
  [1,    { ...ABIERTO, iris: { x: 0 }, cuerpo: { y: 0 }, ...puas([0]) }],
])

action("golpe", { duration: 0.25, loop: false }, [
  [0,    { ...ABIERTO, cuerpo: { x: 0, rot: 0 }, iris: { x: 0 }, ...puas([0]) }],
  [0.15, { ...CERRADO, cuerpo: { x: 16, rot: 5 }, ...puas([-8, 8]) }],
  [0.4,  { cuerpo: { x: -12, rot: -4 }, ...puas([7, -7]) }],
  [0.55, { ...CERRADO }],
  [0.7,  { cuerpo: { x: 5, rot: 2 }, ...puas([-3, 3]) }],
  [1,    { ...ABIERTO, cuerpo: { x: 0, rot: 0 }, iris: { x: 0 }, ...puas([0]) }],
])
