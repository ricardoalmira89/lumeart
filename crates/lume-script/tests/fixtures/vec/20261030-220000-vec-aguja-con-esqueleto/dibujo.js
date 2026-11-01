// dibujo.js: nave-insecto "aguja" con esqueleto, vista desde arriba mirando hacia abajo.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  tinta: "#2c0f1d",
  blanco: "#ffffff",
  cuerpo: "#ff4a24",
  cuerpoLuz: "#f9a37e",
  cuerpoSombra: "#c70a02",
  aleta: "#b81b2d",
  aletaOsc: "#760e2a",
  anillo: "#3b1429",
  anilloLuz: "#8c6a82",
  oro: "#ffd23c",
  oroOsc: "#d07800",
  llama: "#e2580b",
  llamaMed: "#ff8e1e",
  llamaLuz: "#ffe46e",
  ojoAro: "#262b3a",
  iris: "#22d6ff",
  irisOsc: "#0a86c4",
  pupila: "#0b5a76",
}
const BORDE = 64   // borde blanco de pegatina (la mitad queda fuera)
const LINEA = 12   // contorno oscuro (stroke detrás del relleno)

// ---- siluetas (se reutilizan para borde, relleno y clip) ----
const S = {
  llama: [[512, 36, 0], [548, 72], [571, 114], [562, 158], [532, 250], [492, 250], [462, 158], [454, 114], [477, 72]],
  aletaIzq: [[348, 106, 0], [462, 170, 0.5], [506, 200], [512, 262], [472, 298], [428, 285, 0.5]],
  aletaDer: [[676, 106, 0], [562, 170, 0.5], [518, 200], [512, 262], [552, 298], [598, 285, 0.5]],
  aguijon: [[468, 668, 0], [556, 668, 0], [556, 736, 0.5], [516, 966, 0], [508, 966, 0], [468, 736, 0.5]],
  cuerpo: [[512, 128], [560, 162], [597, 240], [614, 335], [614, 435], [600, 545], [576, 630], [563, 705], [462, 705], [452, 630], [428, 545], [415, 435], [418, 335], [434, 240], [466, 162]],
  anillo: [[455, 688, 0], [572, 686, 0], [575, 734, 0.5], [565, 742, 0], [461, 741, 0], [452, 734, 0.5]],
  cresta: [[512, 114, 0], [535, 172], [543, 258], [535, 330], [512, 352, 0.5], [489, 330], [481, 258], [489, 172]],
}

// ---- esqueleto ----
rig({ origin: [512, 470] })
bone("cuerpo", { at: [512, 470] })
bone("llama", { parent: "cuerpo", at: [512, 150] })
bone("aleta_izq", { parent: "cuerpo", at: [470, 222] })
bone("aleta_der", { parent: "cuerpo", at: [554, 222] })
bone("aguijon", { parent: "cuerpo", at: [512, 712] })
bone("cresta", { parent: "cuerpo", at: [512, 335] })
bone("ojo", { parent: "cuerpo", at: [512, 470] })
bone("pupila", { parent: "ojo", at: [512, 468] })

// ---- pasada 1: borde blanco de pegatina de cada pieza, detrás de todo ----
const borde = (pts, f = blob) => f(pts, { fill: P.blanco, stroke: P.blanco, width: BORDE, join: "round" })
part("llama", () => borde(S.llama))
part("aleta_izq", () => borde(S.aletaIzq))
part("aleta_der", () => borde(S.aletaDer))
part("aguijon", () => borde(S.aguijon))
part("cuerpo", () => { borde(S.cuerpo); borde(S.anillo) })

// ---- pasada 2: piezas en color, de atrás hacia delante ----
const pieza = (pts, fill) => blob(pts, { fill, stroke: P.tinta, width: LINEA * 2, behind: true, join: "round" })

// llama del motor (larga por abajo: la base queda escondida bajo el cuerpo al estirarse)
part("llama", () => {
  const f = pieza(S.llama, P.llama)
  clip(f, () => {
    blob([[505, 60, 0], [540, 110], [548, 160], [520, 210], [470, 200], [462, 130]], { fill: P.llamaMed })
    blob([[466, 90], [496, 60], [488, 130], [470, 160], [440, 140]], { fill: P.llama })
  })
  blob([[512, 74, 0], [532, 108], [540, 145], [512, 170], [486, 145], [492, 108]], { fill: P.llamaLuz })
  brush([[505, 92], [498, 128]], { width: 6, color: alpha(P.blanco, 0.7) })
})

// aletas laterales
part("aleta_izq", () => {
  const f = pieza(S.aletaIzq, P.aleta)
  clip(f, () => {
    blob([[440, 290], [470, 230], [520, 210], [520, 310]], { fill: P.aletaOsc })
    brush([[372, 128], [403, 158], [436, 182]], { width: [4, 11, 3], taper: [0.4, 0.5], color: alpha(P.blanco, 0.85) })
  })
})
part("aleta_der", () => {
  const f = pieza(S.aletaDer, P.aletaOsc)
  clip(f, () => {
    blob([[676, 106, 0], [566, 172], [560, 230], [600, 200]], { fill: P.aleta })
    blob([[676, 106, 0], [650, 160], [610, 280], [690, 280]], { fill: shade(P.aletaOsc, -0.25) })
  })
})

// aguijón (sale por debajo del anillo)
part("aguijon", () => {
  const f = pieza(S.aguijon, P.oro)
  clip(f, () => {
    poly([[514, 660], [600, 660], [600, 980], [514, 980]], { fill: P.oroOsc })
    brush([[488, 748], [494, 820], [503, 900]], { width: [9, 7, 2], taper: [0.05, 0.6], color: alpha(P.blanco, 0.85) })
  })
})

// cuerpo en huso + anillo oscuro
part("cuerpo", () => {
  const f = pieza(S.cuerpo, P.cuerpo)
  clip(f, () => {
    // resplandor del ojo
    circle(512, 470, 120, { fill: radial(512, 470, 120, [[0, "#ff9a7a"], [0.55, alpha("#ff8a6a", 0.5)], [1, alpha(P.cuerpo, 0)]]) })
    // sombra derecha
    blob([[560, 150], [650, 150], [650, 720], [552, 720], [578, 600], [598, 470], [598, 340], [582, 240]], { fill: P.cuerpoSombra })
    // luz izquierda
    blob([[470, 190], [446, 260], [432, 360], [430, 470], [442, 570], [470, 650], [450, 660], [410, 560], [398, 430], [404, 320], [430, 230]], { fill: P.cuerpoLuz, opacity: 0.85 })
    brush([[470, 212], [456, 262], [447, 340]], { width: [6, 14, 5], taper: [0.3, 0.5], color: P.blanco })
  })
  circle(452, 392, 8, { fill: P.blanco })
  // tapa del motor
  rect(480, 128, 66, 26, { r: 9, fill: P.anillo, stroke: P.tinta, width: 8 })
  // anillo
  const a = pieza(S.anillo, P.anillo)
  clip(a, () => {
    poly([[452, 724], [580, 722], [580, 750], [452, 750]], { fill: shade(P.anillo, -0.3) })
  })
  rect(472, 696, 40, 9, { r: 4.5, fill: P.anilloLuz })
})

// cresta central
part("cresta", () => {
  const f = pieza(S.cresta, P.aleta)
  clip(f, () => {
    rect(512, 100, 50, 270, { fill: P.aletaOsc })
  })
  brush([[500, 160], [495, 215], [498, 275]], { width: [3, 8, 3], taper: [0.4, 0.4], color: alpha(P.blanco, 0.85) })
})

// ojo (aro oscuro e iris)
part("ojo", () => {
  circle(512, 470, 70, { fill: P.ojoAro, stroke: P.tinta, width: 6, glow: { blur: 10, color: alpha(P.ojoAro, 0.7) } })
  const iris = circle(514, 470, 55, { fill: P.iris })
  clip(iris, () => {
    const c = circle(540, 498, 54, { fill: P.irisOsc })
    circle(522, 470, 40, { fill: P.iris })
  })
})
part("pupila", () => {
  circle(512, 468, 17, { fill: P.pupila })
})
// brillos fijos del ojo, encima de la pupila
part("ojo", () => {
  ellipse(491, 447, 17, 11, { fill: P.blanco, rotate: -22 })
  circle(527, 437, 5.5, { fill: P.blanco })
})

// ---- animaciones ----
// Aletas: positivo pliega la izquierda hacia el cuerpo, negativo la abre (la derecha al revés).
// Llama: y negativo la saca más por arriba (más larga), positivo la esconde bajo el cuerpo.
action("volar", { duration: 0.4, loop: true }, [
  [0, { llama: { y: 0 }, aleta_izq: 0, aleta_der: 0, pupila: { x: -8, y: 0 } }],
  [0.25, { llama: { y: -16 }, aleta_izq: -3, aleta_der: 3 }],
  [0.5, { llama: { y: 4 }, aleta_izq: 2, aleta_der: -2, pupila: { x: 8, y: 0 } }],
  [0.75, { llama: { y: -12 }, aleta_izq: -3, aleta_der: 3 }],
  [1, { llama: { y: 0 }, aleta_izq: 0, aleta_der: 0, pupila: { x: -8, y: 0 } }],
])

action("aviso", { duration: 0.5, loop: false }, [
  [0, { cuerpo: { rot: 0, x: 0, y: 0 }, aleta_izq: 0, aleta_der: 0, aguijon: { y: 0 }, llama: { y: 0 }, pupila: { x: 0, y: 0 } }],
  [0.6, { cuerpo: { y: -14 }, aleta_izq: -30, aleta_der: 30, aguijon: { y: 32 }, llama: { y: 12 }, pupila: { x: 0, y: 0 } }],
  [1, { cuerpo: { y: -18 }, aleta_izq: -34, aleta_der: 34, aguijon: { y: 40 }, llama: { y: 16 }, pupila: { x: 0, y: 0 } }],
])

action("picado", { duration: 0.3, loop: true }, [
  [0, { aleta_izq: 22, aleta_der: -22, aguijon: { y: 40 }, llama: { rot: 0, y: -52 }, pupila: { x: 0, y: 4 } }],
  [0.33, { llama: { rot: -4, y: -40 } }],
  [0.66, { llama: { rot: 4, y: -58 } }],
  [1, { aleta_izq: 22, aleta_der: -22, aguijon: { y: 40 }, llama: { rot: 0, y: -52 }, pupila: { x: 0, y: 4 } }],
])

action("golpe", { duration: 0.25, loop: false }, [
  [0, { cuerpo: { rot: 0, x: 0, y: 0 }, aleta_izq: 0, aleta_der: 0, pupila: { x: 0, y: 0 } }],
  [0.2, { cuerpo: { rot: -7, x: -14, y: -4 }, aleta_izq: -12, aleta_der: 6, pupila: { x: 6, y: -4 } }],
  [0.45, { cuerpo: { rot: 6, x: 12, y: 2 }, aleta_izq: 8, aleta_der: -12, pupila: { x: -6, y: 3 } }],
  [0.7, { cuerpo: { rot: -2, x: -4, y: 0 }, aleta_izq: -3, aleta_der: 3 }],
  [1, { cuerpo: { rot: 0, x: 0, y: 0 }, aleta_izq: 0, aleta_der: 0, pupila: { x: 0, y: 0 } }],
])
