// dibujo.js: mantis jefe vista desde arriba, cabeza abajo, montada con esqueleto.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#10290f",      // contorno oscuro
  white: "#ffffff",    // borde de pegatina
  lime: "#93e22c",     // verde lima claro
  green: "#4c9a22",    // verde medio
  greenD: "#2b6519",   // verde sombra
  wingL: "#b6f07e",    // ala clara
  wingM: "#8fcf6b",    // ala media
  vein: "#2f6a1e",     // venas
  purple: "#8443c8",   // violeta
  purpleD: "#4f2382",  // violeta sombra
  purpleL: "#c19af0",  // brillo violeta
  orange: "#f8ab1c",   // acento naranja
  orangeD: "#d06a08",
  red: "#e3173a",      // ojos
  redD: "#9e0f28",
  pupil: "#0b0b0b",
}
const OUT = 10   // grosor del contorno oscuro
const STK = 46   // grosor del borde blanco (la mitad asoma)

// ---------- Geometría compartida (silueta blanca y pieza de color) ----------
const sx = (side, p) => side < 0 ? p : [1024 - p[0], p[1]]
const LEGS = {
  tras: [[440, 470], [400, 400], [318, 300], [250, 232], [155, 140], [122, 95], [112, 57]],
  del: [[440, 535], [380, 512], [290, 480], [220, 560], [150, 650], [124, 700], [118, 752]],
}
const LEG_JOINTS = { tras: [[318, 300, 24], [155, 140, 20]], del: [[290, 480, 24], [150, 650, 20]] }
const ABD = [[512, 52], [578, 82], [628, 150], [652, 262], [642, 372], [604, 448], [512, 476], [420, 448], [382, 372], [372, 262], [396, 150], [446, 82]]
const TOR = [[512, 428], [584, 436], [628, 478], [638, 545], [616, 602], [560, 622], [512, 618], [464, 622], [408, 602], [386, 545], [396, 478], [440, 436]]
const HEAD = [[512, 582], [568, 590], [598, 628], [596, 700], [584, 772], [618, 830], [608, 892], [560, 944], [512, 968, 0.5], [464, 944], [416, 892], [406, 830], [440, 772], [428, 700], [426, 628], [456, 590]]
const SMALL_EYES = [[390, 700], [634, 700]]
const RED = { izq: [418, 838], der: [606, 838] }

// ---------- Esqueleto ----------
rig({ origin: [512, 525] })
bone("torax", { at: [512, 525] })
bone("abdomen", { parent: "torax", at: [512, 445] })
bone("cabeza", { parent: "torax", at: [512, 600] })
bone("ojo_izq", { parent: "cabeza", at: [448, 836] })
bone("ojo_der", { parent: "cabeza", at: [576, 836] })
bone("pata_tras_izq", { parent: "torax", at: [430, 455] })
bone("pata_tras_der", { parent: "torax", at: [594, 455] })
bone("pata_del_izq", { parent: "torax", at: [425, 528] })
bone("pata_del_der", { parent: "torax", at: [599, 528] })

// ---------- Pasada 1: borde blanco de pegatina detrás de todo ----------
for (const s of [-1, 1]) {
  const n = s < 0 ? "izq" : "der"
  for (const k of ["tras", "del"]) {
    part(`pata_${k}_${n}`, () => {
      curve(LEGS[k].map(p => sx(s, p)), { stroke: P.white, width: 34 + STK, cap: "round" })
      for (const [x, y, r] of LEG_JOINTS[k]) { const c = sx(s, [x, y]); circle(c[0], c[1], r + 5 + STK / 2, { fill: P.white }) }
    })
  }
}
part("abdomen", () => { blob(ABD, { fill: P.white, stroke: P.white, width: STK }) })
part("torax", () => { blob(TOR, { fill: P.white, stroke: P.white, width: STK }) })
part("cabeza", () => {
  blob(HEAD, { fill: P.white, stroke: P.white, width: STK })
  for (const [x, y] of SMALL_EYES) circle(x, y, 58 + STK / 2, { fill: P.white })
})
for (const n of ["izq", "der"]) part(`ojo_${n}`, () => { const [x, y] = RED[n]; ellipse(x, y, 68 + STK / 2, 63 + STK / 2, { fill: P.white }); ellipse(x, y, 84, 78, { fill: alpha(P.red, 0.22), blur: 14 }) })

// ---------- Pasada 2: piezas de color ----------
// PATAS (detrás del cuerpo)
function joint(x, y, r) {
  circle(x, y, r + 6, { fill: P.ink })
  const j = circle(x, y, r, { fill: P.purple })
  clip(j, () => {
    circle(x + r * 0.45, y + r * 0.45, r * 0.95, { fill: P.purpleD })
  })
  circle(x - r * 0.35, y - r * 0.35, r * 0.33, { fill: P.orange })
}
for (const s of [-1, 1]) {
  const n = s < 0 ? "izq" : "der"
  for (const k of ["tras", "del"]) {
    part(`pata_${k}_${n}`, () => {
      const pts = LEGS[k].map(p => sx(s, p))
      curve(pts, { stroke: P.ink, width: 34, cap: "round", join: "round" })
      curve(pts, { stroke: P.greenD, width: 20, cap: "round", join: "round" })
      curve(shift(pts, -2, -2), { stroke: P.green, width: 13, cap: "round", join: "round" })
      brush(shift(pts.slice(1), -3, -3), { width: 6, taper: [0.15, 0.25], color: P.lime })
      for (const [x, y, r] of LEG_JOINTS[k]) { const c = sx(s, [x, y]); joint(c[0], c[1], r) }
    })
  }
}

// ABDOMEN con alas plegadas
const WING = [[506, 118], [468, 112, 0.5], [432, 138], [414, 200], [410, 300], [420, 385], [452, 432], [504, 444], [512, 300]]
part("abdomen", () => {
  const body = blob(ABD, { fill: P.green, stroke: P.ink, width: OUT * 2, behind: true })
  clip(body, () => {
    // luz arriba a la izquierda, sombra abajo a la derecha
    ellipse(470, 170, 120, 170, { fill: P.lime, opacity: 0.85 })
    ellipse(600, 400, 110, 140, { fill: P.greenD, opacity: 0.7 })
    // segmentos violetas en los costados
    for (const s of [-1, 1]) {
      const segs = [[150, 52], [212, 60], [278, 60], [342, 58], [404, 52]]
      for (const [y, h] of segs) {
        const x = s < 0 ? 362 : 1024 - 362 - 62
        const seg = rect(x, y - h / 2, 62, h - 6, { r: 14, fill: P.purple, stroke: P.ink, width: 6 })
        clip(seg, () => {
          rect(x + (s < 0 ? 34 : 0), y - h / 2, 30, h, { fill: P.purpleD })
          brush([[x + 14 + (s < 0 ? 18 : 6), y - 8], [x + 46 - (s < 0 ? 0 : 14), y - 12]], { width: 5, color: P.purpleL, taper: [0.3, 0.5] })
        })
      }
    }
    // línea de borde interior
    blob(grow(ABD, 0.92), { fill: "none", stroke: P.vein, width: 4, opacity: 0.4 })
  })
  // alas plegadas
  for (const s of [-1, 1]) {
    const pts = WING.map(p => s < 0 ? p : [1024 - p[0], ...p.slice(1)])
    const w = blob(pts, { fill: s < 0 ? P.wingL : P.wingM, stroke: P.ink, width: 7, opacity: 0.95 })
    clip(w, () => {
      const cx = s < 0 ? 470 : 554
      ellipse(cx + s * 40, 330, 50, 160, { fill: s < 0 ? P.wingM : shade(P.wingM, -0.2), opacity: 0.6 })
      // vena central y venillas en V
      const vx = (x) => s < 0 ? x : 1024 - x
      brush([[vx(468), 130], [vx(462), 260], [vx(470), 430]], { width: 6, color: P.vein, taper: [0.2, 0.2] })
      for (const y of [175, 235, 295, 355, 410]) {
        brush([[vx(418), y + 22], [vx(465), y]], { width: 3.5, color: P.vein, taper: [0.1, 0.3] })
        brush([[vx(465), y], [vx(505), y + 28]], { width: 3.5, color: P.vein, taper: [0.1, 0.3] })
      }
      if (s < 0) brush([[448, 165], [432, 260], [438, 350]], { width: 12, color: P.white, opacity: 0.75, taper: [0.4, 0.5] })
    })
  }
  // pliegue central entre alas
  brush([[512, 120], [512, 300], [512, 440]], { width: 7, color: P.ink, taper: [0.1, 0.1] })
  // ocelo naranja en la punta
  circle(512, 92, 22, { fill: P.ink })
  circle(512, 92, 16, { fill: P.orange })
  circle(507, 87, 5, { fill: P.white, opacity: 0.8 })
})

// TÓRAX
part("torax", () => {
  const t = blob(TOR, { fill: P.green, stroke: P.ink, width: OUT * 2, behind: true })
  clip(t, () => {
    ellipse(470, 460, 110, 60, { fill: P.lime, opacity: 0.6 })
    ellipse(420, 590, 80, 70, { fill: P.greenD, opacity: 0.8 })
    ellipse(620, 560, 50, 90, { fill: P.greenD, opacity: 0.8 })
  })
  const sh = ellipse(512, 526, 86, 72, { fill: P.purpleD, stroke: P.ink, width: 8 })
  clip(sh, () => {
    blob([[430, 520], [440, 470], [490, 455], [545, 458], [545, 500], [520, 540], [470, 548]], { fill: P.purple })
    brush([[458, 515], [470, 482], [510, 470]], { width: 11, color: P.purpleL, taper: [0.3, 0.6] })
  })
})

// CABEZA (ojitos oscuros, escudo, mandíbulas)
part("cabeza", () => {
  const h = blob(HEAD, { fill: P.green, stroke: P.ink, width: OUT * 2, behind: true })
  clip(h, () => {
    ellipse(470, 640, 80, 70, { fill: P.lime, opacity: 0.9 })
    ellipse(455, 740, 40, 60, { fill: P.lime, opacity: 0.7 })
    ellipse(455, 910, 50, 40, { fill: P.lime, opacity: 0.6 })
    ellipse(585, 760, 40, 110, { fill: P.greenD, opacity: 0.6 })
    ellipse(570, 920, 60, 50, { fill: P.greenD, opacity: 0.6 })
  })
  // ojitos oscuros laterales
  for (const [x, y] of SMALL_EYES) {
    circle(x, y, 58, { fill: P.ink })
    const e = circle(x, y, 46, { fill: P.purple })
    clip(e, () => { circle(x + 14, y + 16, 44, { fill: P.purpleD }) })
    brush([[x - 28, y - 22], [x - 5, y - 36], [x + 26, y - 26]], { width: 7, color: P.purpleL, taper: [0.3, 0.4] })
    circle(x, y, 32, { fill: P.ink })
    circle(x, y, 27, { fill: P.orange })
    circle(x, y, 20, { fill: P.pupil })
    circle(x - 7, y - 7, 5, { fill: P.white, opacity: 0.85 })
  }
  // escudo violeta con franja naranja
  const sh = ellipse(512, 700, 57, 92, { fill: P.purpleD, stroke: P.ink, width: 8 })
  clip(sh, () => {
    blob([[470, 690], [478, 630], [512, 610], [512, 700], [495, 770], [470, 740]], { fill: P.purple })
    brush([[478, 690], [482, 650], [500, 628]], { width: 9, color: P.purpleL, taper: [0.3, 0.5] })
  })
  blob([[512, 622, 0], [520, 700], [512, 778, 0], [504, 700]], { fill: P.orange })
  // triángulo violeta entre los ojos
  const tri = blob([[468, 800, 0.4], [556, 800, 0.4], [512, 908, 0]], { fill: P.purpleD, stroke: P.ink, width: 6 })
  clip(tri, () => { blob([[468, 800], [518, 800], [512, 900, 0]], { fill: P.purple }) })
  brush([[475, 795], [512, 790], [550, 795]], { width: 6, color: P.ink, taper: [0, 0] })
  // mandíbulas naranjas
  for (const s of [-1, 1]) {
    const m = blob([[512 + s * 6, 900], [512 + s * 44, 898], [512 + s * 42, 918], [512 + s * 8, 948, 0]].map(p => p), { fill: P.orange, stroke: P.ink, width: 6 })
    clip(m, () => { blob([[512 + s * 44, 905], [512 + s * 30, 940], [512 + s * 10, 950], [512 + s * 20, 915]], { fill: P.orangeD }) })
  }
})

// OJOS ROJOS
for (const n of ["izq", "der"]) {
  part(`ojo_${n}`, () => {
    const [x, y] = RED[n]
    ellipse(x, y, 68, 63, { fill: P.ink })
    const e = ellipse(x, y, 57, 52, { fill: P.red })
    clip(e, () => {
      ellipse(x + 20, y + 22, 55, 45, { fill: P.redD, opacity: 0.8 })
      for (let i = -4; i <= 4; i++) for (let j = -4; j <= 4; j++) {
        circle(x + i * 13 + (j % 2) * 6, y + j * 13, 2.6, { fill: "#ff9aa8", opacity: 0.55 })
      }
    })
    ellipse(x - 18, y - 22, 18, 9, { fill: P.white, rotate: -22 })
    circle(x - 2, y - 32, 6, { fill: P.white })
  })
}

// ---------- Acciones ----------
// Patas izq: ángulo negativo las abre (traseras hacia arriba, delanteras hacia abajo); der al revés.
action("acechar", { duration: 1.6, loop: true }, [
  [0, { abdomen: { rot: 0, y: 0 }, cabeza: 0, pata_tras_izq: 0, pata_tras_der: 0, pata_del_izq: 0, pata_del_der: 0, ojo_izq: 0, ojo_der: 0 }],
  [0.25, { abdomen: { rot: 3, y: -6 }, cabeza: 5, pata_tras_izq: -2, pata_del_der: 3, ojo_izq: -3, ojo_der: 3 }],
  [0.5, { abdomen: { rot: 0, y: 2 }, cabeza: 0, pata_tras_izq: 0, pata_tras_der: 2, pata_del_izq: -3, pata_del_der: 0, ojo_izq: 0, ojo_der: 0 }],
  [0.75, { abdomen: { rot: -3, y: -6 }, cabeza: -5, pata_tras_der: 0, pata_del_izq: 0, pata_tras_izq: 2, ojo_izq: 3, ojo_der: -3 }],
  [1, { abdomen: { rot: 0, y: 0 }, cabeza: 0, pata_tras_izq: 0, pata_tras_der: 0, pata_del_izq: 0, pata_del_der: 0, ojo_izq: 0, ojo_der: 0 }],
])
action("aviso", { duration: 0.6, loop: false }, [
  [0, { cabeza: { rot: 0, y: 0 }, abdomen: { rot: 0, y: 0 }, torax: { rot: 0, y: 0 }, pata_tras_izq: 0, pata_tras_der: 0, pata_del_izq: 0, pata_del_der: 0, ojo_izq: 0, ojo_der: 0 }],
  [0.6, { cabeza: { rot: 0, y: -30 }, abdomen: { rot: 0, y: 26 }, torax: { rot: 0, y: -6 }, pata_tras_izq: -10, pata_tras_der: 10, pata_del_izq: -12, pata_del_der: 12, ojo_izq: -6, ojo_der: 6 }],
  [1, { cabeza: { rot: 0, y: -26 }, abdomen: { rot: 0, y: 22 }, torax: { rot: 0, y: -5 }, pata_tras_izq: -9, pata_tras_der: 9, pata_del_izq: -11, pata_del_der: 11, ojo_izq: -5, ojo_der: 5 }],
])
action("golpe", { duration: 0.25, loop: false }, [
  [0, { torax: { rot: 0, x: 0, y: 0 }, abdomen: 0, cabeza: 0 }],
  [0.2, { torax: { rot: -5, x: -12, y: -6 }, abdomen: 6, cabeza: 6, pata_tras_izq: -4, pata_del_der: 4 }],
  [0.45, { torax: { rot: 4, x: 10, y: 4 }, abdomen: -6, cabeza: -5, pata_tras_izq: 3, pata_del_der: -3 }],
  [0.7, { torax: { rot: -2, x: -4, y: 0 }, abdomen: 3, cabeza: 2, pata_tras_izq: 0, pata_del_der: 0 }],
  [1, { torax: { rot: 0, x: 0, y: 0 }, abdomen: 0, cabeza: 0, pata_tras_izq: 0, pata_del_der: 0 }],
])
