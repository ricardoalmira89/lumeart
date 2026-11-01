// dibujo.js: reina insecto acorazada (jefe visto desde arriba, mirando hacia abajo), con esqueleto.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

// Paleta sacada de la referencia
const P = {
  ink: "#15310f",
  lima: "#a6e22e", limaClaro: "#c8f25e", verde: "#61ac24", oscuro: "#2f6a1f", hondo: "#1f4d14",
  alaClaro: "#d6eefd", ala: "#aad7f5", alaMedio: "#8cbbd3", alaBorde: "#5f8fae",
  rojo: "#f63242", rojoClaro: "#ff7278", rojoMedio: "#c21e30", rojoOsc: "#940e21", cavidad: "#4a0a12",
  naranja: "#ffb21f", naranjaOsc: "#c46a0a",
  metal: "#3e4a44", metalClaro: "#7d9084",
  blanco: "#ffffff",
}
const BORDE = 26 // grosor del borde blanco de pegatina

// s = -1 es el lado izquierdo (coordenadas tal cual), s = 1 lo refleja respecto a x = 512
const M = (s, pts) => pts.map(([x, y, ...r]) => [s < 0 ? x : 1024 - x, y, ...r])
const mx = (s, x) => (s < 0 ? x : 1024 - x)
const lado = (s) => (s < 0 ? "izq" : "der")
const tinta = (f, w = 5) => ink(f, { width: w, weight: 2.2, color: P.ink })
const remache = (x, y, r = 9) => {
  circle(x, y, r + 4, { fill: P.ink })
  circle(x, y, r, { fill: P.naranja })
  circle(x + r * 0.25, y + r * 0.3, r * 0.55, { fill: P.naranjaOsc, opacity: 0.5 })
  circle(x - r * 0.3, y - r * 0.35, r * 0.3, { fill: "#fff3c0" })
}

// ---------- Siluetas (en coordenadas del lado izquierdo) ----------
const ALA_SUP = [[160, 58, 0], [290, 128], [400, 225], [432, 330], [340, 420], [220, 345], [110, 225], [58, 125], [52, 88, 0.5], [95, 68]]
const ALA_SUP_VENA = [[118, 98], [152, 88], [252, 143], [362, 228], [428, 300], [405, 348], [332, 333], [230, 258], [140, 158]]
const ALA_SUP_LUZ = [[140, 108], [158, 104], [254, 156], [358, 238], [412, 300], [395, 332], [336, 320], [238, 250], [152, 158]]
const ALA_INF = [[140, 468, 0], [300, 434], [392, 448], [398, 515], [330, 542], [200, 552], [126, 549], [113, 530, 0.5]]
const ALA_INF_VENA = [[188, 488], [240, 469], [372, 452], [400, 498], [362, 524], [230, 520], [190, 506]]
const ALA_INF_LUZ = [[204, 488], [245, 476], [368, 462], [388, 498], [356, 514], [236, 512]]
const ABDOMEN = [[512, 33], [574, 48], [616, 105], [636, 190], [630, 268], [512, 300], [394, 268], [388, 190], [408, 105], [450, 48]]
const TORAX = [[512, 266], [622, 270], [692, 310], [710, 390], [692, 470], [640, 527], [512, 548], [384, 527], [332, 470], [314, 390], [332, 310], [402, 270]]
const HUECO = [[512, 272, 0], [574, 330], [586, 420], [542, 500], [512, 515, 0.5], [482, 500], [438, 420], [450, 330]]
const PLACA = [[492, 274, 0.5], [402, 270], [334, 308], [314, 390], [332, 468], [382, 524], [468, 540], [500, 516, 0.5], [470, 472], [448, 402], [458, 330]]
const CABEZA = [[512, 495], [602, 498], [670, 530], [694, 592], [682, 652], [630, 692], [512, 707], [394, 692], [342, 652], [330, 592], [354, 530], [422, 498]]
const BRAZO = [[372, 532], [300, 552], [256, 598], [244, 680], [286, 734], [332, 750, 0], [344, 684], [358, 622], [388, 580]]

// ---------- Huesos ----------
rig({ origin: [512, 400] })
bone("torax", { at: [512, 400] })
bone("abdomen", { parent: "torax", at: [512, 286] })
bone("cabeza", { parent: "torax", at: [512, 512] })
bone("ala_sup_izq", { parent: "torax", at: [425, 318] })
bone("ala_sup_der", { parent: "torax", at: [599, 318] })
bone("ala_inf_izq", { parent: "torax", at: [382, 492] })
bone("ala_inf_der", { parent: "torax", at: [642, 492] })
bone("nucleo", { parent: "torax", at: [512, 400] })
bone("placa_izq", { parent: "torax", at: [335, 330] })
bone("placa_der", { parent: "torax", at: [689, 330] })
bone("canon_izq", { parent: "cabeza", at: [296, 640] })
bone("canon_der", { parent: "cabeza", at: [728, 640] })

// ---------- Pasada 1: borde blanco de pegatina de cada pieza (todo detrás) ----------
const blanco = (pts) => blob(pts, { fill: P.blanco, stroke: P.blanco, width: BORDE, join: "round" })
for (const s of [-1, 1]) part("ala_sup_" + lado(s), () => blanco(M(s, ALA_SUP)))
for (const s of [-1, 1]) part("ala_inf_" + lado(s), () => blanco(M(s, ALA_INF)))
part("abdomen", () => blanco(ABDOMEN))
part("torax", () => blanco(TORAX))
for (const s of [-1, 1]) part("placa_" + lado(s), () => blanco(M(s, PLACA)))
part("cabeza", () => { blanco(BRAZO); blanco(M(1, BRAZO)); blanco(CABEZA) })
for (const s of [-1, 1]) part("canon_" + lado(s), () => rect(mx(s, 296) - 24, 606, 48, 128, { r: 16, fill: P.blanco, stroke: P.blanco, width: BORDE }))

// ---------- Pasada 2: piezas en color, de atrás hacia delante ----------
// Una ala: membrana translúcida azulada + nervio verde acorazado
function ala(s, MEM, VENA, LUZ, rx, ry, remacheEn) {
  const b = bbox(blob(M(s, MEM), { hidden: true }))
  const mem = blob(M(s, MEM), { fill: linear(b.x, b.y, b.x + b.w * 0.6, b.y + b.h, [P.alaClaro, P.ala, [1, P.alaMedio]]), opacity: 0.96 })
  clip(mem, () => {
    // reflejo del cristal a lo largo del borde de ataque
    const borde = M(s, MEM).slice(0, 3)
    brush(shift(borde, 0, 14), { width: [10, 16, 6], color: P.blanco, opacity: 0.7, taper: [0.2, 0.5] })
    blob(M(s, VENA).map(([x, y]) => [x, y + 26]), { fill: P.alaBorde, opacity: 0.35, blur: 8 })
  })
  tinta(mem, 4)
  const v = blob(M(s, VENA), { fill: P.oscuro })
  const luz = blob(M(s, LUZ), { fill: P.lima })
  clip(luz, () => {
    blob(M(s, LUZ).map(([x, y]) => [x, y - 12]), { fill: P.limaClaro, opacity: 0.6, blur: 4 })
    blob(M(s, LUZ).map(([x, y]) => [x, y + 16]), { fill: P.verde, opacity: 0.8 })
  })
  tinta(v, 5)
  if (remacheEn) remache(mx(s, remacheEn[0]), remacheEn[1], 9)
}
// ALAS SUPERIORES
for (const s of [-1, 1]) part("ala_sup_" + lado(s), () => ala(s, ALA_SUP, ALA_SUP_VENA, ALA_SUP_LUZ, 0, 0, [150, 112]))
// ALAS INFERIORES
for (const s of [-1, 1]) part("ala_inf_" + lado(s), () => ala(s, ALA_INF, ALA_INF_VENA, ALA_INF_LUZ, 0, 0, null))
// ABDOMEN a franjas
part("abdomen", () => {
  const ab = blob(ABDOMEN, { fill: P.verde })
  const cortes = [100, 160, 222, 286]
  clip(ab, () => {
    let y0 = 30
    for (const y1 of cortes) {
      // cada segmento: cojín con luz arriba a la izquierda y sombra abajo
      ellipse(470, y0 + (y1 - y0) * 0.45, 95, (y1 - y0) * 0.32, { fill: P.lima })
      ellipse(450, y0 + (y1 - y0) * 0.38, 45, (y1 - y0) * 0.14, { fill: P.limaClaro, opacity: 0.8 })
      rect(380, y1 - 13, 270, 16, { fill: P.oscuro, opacity: 0.85 })
      y0 = y1
    }
    // sombra del lado derecho
    blob([[600, 40], [660, 120], [660, 290], [590, 290], [612, 200], [600, 110]], { fill: P.oscuro, opacity: 0.7 })
    brush([[430, 60], [470, 42], [520, 38]], { width: 7, color: P.blanco, opacity: 0.85 })
    for (const y of cortes.slice(0, 3)) brush([[392, y - 4], [450, y + 2], [512, y + 4], [574, y + 2], [632, y - 4]], { width: [6, 7, 7, 7, 6], color: P.ink, taper: [0, 0] })
  })
  tinta(ab, 5)
  remache(512, 85, 8); remache(512, 145, 8); remache(512, 207, 8)
})
// TÓRAX (interior que queda a la vista al abrir las placas)
part("torax", () => {
  const t = blob(TORAX, { fill: P.oscuro })
  clip(t, () => {
    blob(grow(TORAX, 0.8), { fill: P.hondo, blur: 18 })
    // nervaduras internas
    for (const s of [-1, 1]) for (const k of [0, 1, 2]) brush(M(s, [[340 + k * 20, 330 + k * 60], [420, 360 + k * 50], [470, 380 + k * 45]]), { width: 6, color: P.ink, opacity: 0.5 })
  })
  const h = blob(HUECO, { fill: radial(512, 400, 120, [P.rojoMedio, P.rojoOsc, P.cavidad]) })
  clip(h, () => circle(512, 400, 80, { fill: P.rojo, blur: 22, opacity: 0.8 }))
  tinta(h, 4)
  tinta(t, 5)
})
// NÚCLEO rojo brillante, entero
part("nucleo", () => {
  circle(512, 400, 70, { fill: P.rojo, blur: 16, opacity: 0.55 })
  circle(512, 400, 66, { fill: P.ink })
  const n = circle(512, 400, 58, { fill: radial(495, 378, 75, [P.rojoClaro, P.rojo, [0.75, P.rojoMedio], [1, P.rojoOsc]], { fx: 485, fy: 370 }) })
  clip(n, () => {
    circle(545, 430, 50, { fill: P.rojoOsc, opacity: 0.55, blend: "multiply" })
    circle(470, 370, 34, { fill: P.rojoClaro, opacity: 0.45, blur: 6 })
  })
  ellipse(490, 376, 19, 11, { fill: P.blanco, rotate: -32 })
  circle(529, 367, 5, { fill: P.blanco })
})
// PLACAS del pecho (se abren girando desde su borde de fuera)
for (const s of [-1, 1]) part("placa_" + lado(s), () => {
  const p = blob(M(s, PLACA), { fill: radial(mx(s, 400), 360, 190, [P.limaClaro, P.lima, [1, P.verde]]) })
  clip(p, () => {
    // sombra de un tono: parte de abajo y borde interior
    blob(M(s, [[300, 440], [380, 432], [470, 448], [520, 470], [520, 560], [300, 560]]), { fill: P.oscuro })
    blob(M(s, [[300, 420], [380, 412], [460, 425], [470, 445], [380, 435], [300, 445]]), { fill: P.verde })
    brush(M(s, [[488, 290], [460, 340], [452, 410], [478, 480]]), { width: [4, 14, 14, 6], color: P.verde, taper: [0.2, 0.2] })
    // brillo blanco arriba
    brush(M(s, [[352, 330], [384, 300], [440, 290]]), { width: [4, 9, 3], color: P.blanco, opacity: 0.95 })
  })
  brush(M(s, [[352, 422], [400, 428], [446, 440]]), { width: [2, 6, 2], color: P.ink })
  tinta(p, 5)
  remache(mx(s, 372), 380, 9)
})
// CABEZA con brazos laterales y visor
part("cabeza", () => {
  for (const s of [-1, 1]) {
    const b = blob(M(s, BRAZO), { fill: P.verde })
    clip(b, () => {
      blob(M(s, [[300, 560], [372, 540], [380, 590], [310, 600], [262, 640]]), { fill: P.lima })
      blob(M(s, [[330, 640], [360, 640], [350, 760], [300, 760]]), { fill: P.oscuro })
      brush(M(s, [[272, 600], [300, 568], [340, 552]]), { width: [3, 7, 2], color: P.blanco, opacity: 0.8 })
    })
    tinta(b, 5)
  }
  const c = blob(CABEZA, { fill: P.verde })
  clip(c, () => {
    blob([[350, 520], [430, 500], [600, 500], [680, 540], [600, 560], [430, 560], [345, 575]], { fill: P.lima })
    blob([[330, 640], [420, 628], [600, 628], [700, 640], [700, 720], [330, 720]], { fill: P.oscuro })
    blob([[640, 520], [700, 560], [700, 650], [655, 640], [660, 580]], { fill: P.oscuro, opacity: 0.6 })
    brush([[400, 525], [450, 512], [512, 508]], { width: [3, 8, 2], color: P.blanco, opacity: 0.95 })
  })
  tinta(c, 5)
  // visor rojo alargado
  rect(388, 574, 248, 56, { r: 28, fill: P.rojo, blur: 10, opacity: 0.45 })
  rect(392, 577, 240, 50, { r: 25, fill: P.ink })
  const v = rect(400, 584, 224, 36, { r: 18, fill: linear(0, 584, 0, 620, [P.rojoClaro, P.rojo, [1, P.rojoMedio]]) })
  clip(v, () => rect(400, 606, 224, 16, { fill: P.rojoOsc, opacity: 0.5 }))
  rect(410, 589, 150, 7, { r: 3.5, fill: P.blanco, opacity: 0.9 })
  circle(611, 594, 4, { fill: P.blanco })
  remache(452, 665, 8); remache(572, 665, 8)
})
// CAÑONES
for (const s of [-1, 1]) part("canon_" + lado(s), () => {
  const cx = mx(s, 296)
  rect(cx - 24, 606, 48, 128, { r: 16, fill: P.ink })
  const c = rect(cx - 17, 613, 34, 114, { r: 11, fill: P.metal })
  clip(c, () => {
    rect(cx - 17, 613, 11, 114, { fill: P.metalClaro })
    rect(cx + 7, 613, 10, 114, { fill: "#2a332f" })
  })
  rect(cx - 21, 650, 42, 10, { fill: P.naranja })
  rect(cx - 21, 690, 42, 10, { fill: P.naranja })
  rect(cx - 21, 656, 42, 4, { fill: P.naranjaOsc })
  rect(cx - 21, 696, 42, 4, { fill: P.naranjaOsc })
  circle(cx, 630, 9, { fill: P.ink })
  circle(cx, 630, 6, { fill: P.rojo })
  circle(cx - 2, 628, 2, { fill: P.blanco })
  ellipse(cx, 722, 12, 6, { fill: "#0b0f0d" })
})

// ---------- Acciones ----------
// aleteo: alas superiores e inferiores desfasadas un cuarto de ciclo
const aleteo = range(9).map((i) => {
  const t = i / 8
  const sup = 16 * Math.cos(2 * Math.PI * t)
  const inf = 13 * Math.cos(2 * Math.PI * (t - 0.22))
  return [t, {
    ala_sup_izq: sup, ala_sup_der: -sup,
    ala_inf_izq: -inf, ala_inf_der: inf,
    abdomen: 2 * Math.sin(2 * Math.PI * t),
  }]
})
action("aleteo", { duration: 0.25, loop: true }, aleteo)

// abrir: pequeña anticipación hacia dentro y las placas giran hacia fuera hasta dejar el núcleo libre
action("abrir", { duration: 0.6, loop: false }, [
  [0, { placa_izq: 0, placa_der: 0, nucleo: { rot: 0, x: 0, y: 0 } }],
  [0.15, { placa_izq: { rot: -3, x: 4, y: 0 }, placa_der: { rot: 3, x: -4, y: 0 } }],
  [0.75, { placa_izq: { rot: 22, x: -88, y: 0 }, placa_der: { rot: -22, x: 88, y: 0 }, nucleo: { rot: 0, x: 0, y: 6 } }],
  [1, { placa_izq: { rot: 20, x: -82, y: 0 }, placa_der: { rot: -20, x: 82, y: 0 }, nucleo: { rot: 0, x: 0, y: 10 } }],
])

// golpe: sacudida del cuerpo y retroceso de los cañones (hacia arriba, porque disparan hacia abajo)
action("golpe", { duration: 0.25, loop: false }, [
  [0, { torax: { rot: 0, x: 0, y: 0 }, canon_izq: { rot: 0, x: 0, y: 0 }, canon_der: { rot: 0, x: 0, y: 0 }, cabeza: 0, abdomen: 0 }],
  [0.15, { torax: { rot: -4, x: -10, y: -6 }, canon_izq: { rot: 0, x: 0, y: -22 }, canon_der: { rot: 0, x: 0, y: -22 }, cabeza: 3, abdomen: 4 }],
  [0.4, { torax: { rot: 3, x: 8, y: 2 }, canon_izq: { rot: 0, x: 0, y: -14 }, canon_der: { rot: 0, x: 0, y: -14 }, cabeza: -2, abdomen: -3 }],
  [0.7, { torax: { rot: -1.5, x: -3, y: 0 }, canon_izq: { rot: 0, x: 0, y: -4 }, canon_der: { rot: 0, x: 0, y: -4 }, cabeza: 1, abdomen: 1.5 }],
  [1, { torax: { rot: 0, x: 0, y: 0 }, canon_izq: { rot: 0, x: 0, y: 0 }, canon_der: { rot: 0, x: 0, y: 0 }, cabeza: 0, abdomen: 0 }],
])
