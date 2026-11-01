// dibujo.js: cabeza de ciempiés con esqueleto, vista desde arriba, mirando hacia abajo.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  ink: "#13281a",        // contorno oscuro
  white: "#ffffff",      // borde de pegatina
  lime: "#8ee23c",       // verde lima base
  limeMid: "#55b22c",    // sombra lateral
  limeDark: "#25702e",   // banda superior oscura
  limeLight: "#dcf98a",  // brillos
  socket: "#2c6a26",     // cuenca de los ojos
  purple: "#7b3dca",     // caparazón
  purpleDark: "#4d2392", // sombra del caparazón
  purpleLight: "#cfa6f7",
  red: "#ec1620",        // ojos
  redDark: "#8f0a14",
  orange: "#f2a823",     // mandíbulas
  orangeDark: "#c2601a",
  orangeLight: "#ffe39a",
}

const CX = 506          // eje de simetría
const OUT = 22          // contorno oscuro (la mitad queda visible)
const WHITE = 58       // borde blanco total
// refleja puntos respecto al eje (s = 1 izquierda, s = -1 derecha)
const M = (pts, s) => pts.map((p) => [CX + s * (p[0] - CX), p[1], ...p.slice(2)])

// ---------- geometría ----------
const HEAD = ring(CX, 410, 340, 276, 16)
const SHELL_L = [[CX, 214, 0], [400, 216, 0.5], [348, 228, 0.5], [318, 290], [292, 352, 0.5], [340, 420], [398, 466, 0.5], [455, 476], [CX, 478, 0]]
const PALP_L = [[268, 592], [222, 625], [180, 670], [158, 715], [152, 748]]
const MAND_L = [[340, 650], [318, 720], [326, 785], [365, 838], [430, 872], [494, 888, 0], [462, 850], [420, 805], [400, 755], [410, 705], [440, 660]]

rig({ origin: [CX, 430] })
bone("cabeza", { at: [CX, 430] })
bone("palpo_izq", { parent: "cabeza", at: [268, 592] })
bone("palpo_der", { parent: "cabeza", at: [CX * 2 - 268, 592] })
bone("mandibula_izq", { parent: "cabeza", at: [385, 680] })
bone("mandibula_der", { parent: "cabeza", at: [CX * 2 - 385, 680] })
bone("caparazon_izq", { parent: "cabeza", at: [470, 476] })
bone("caparazon_der", { parent: "cabeza", at: [CX * 2 - 470, 476] })

// ---------- piezas ----------
function palpWhite(s) {
  curve(M(PALP_L, s), { stroke: P.white, width: 46 + WHITE - OUT, cap: "round" })
  const k = M([[152, 748]], s)[0]
  circle(k[0], k[1], 44 + (WHITE - OUT) / 2, { fill: P.white })
}
function palp(s) {
  const pts = M(PALP_L, s)
  curve(pts, { stroke: P.ink, width: 50, cap: "round" })
  curve(pts, { stroke: s > 0 ? "#4fae2c" : "#4aa52a", width: 30, cap: "round" })
  // luz a lo largo del tubo, hacia el lado de la luz
  brush(shift(pts.slice(0, 4), -4 * s, -4), { width: [3, 9, 6], color: alpha(P.limeLight, 0.85), taper: [0.2, 0.5] })
  const k = pts[pts.length - 1]
  const knob = circle(k[0], k[1], 42, { fill: "#2b7a2f", stroke: P.ink, width: OUT, behind: true })
  clip(knob, () => {
    circle(k[0] + 14, k[1] + 16, 34, { fill: "#1f5c24" })
  })
  ink(knob, { width: 4, weight: 2, color: P.ink })
  circle(k[0] - 8, k[1] - 9, 11, { fill: P.limeLight })
}

function mandWhite(s) {
  blob(M(MAND_L, s), { fill: P.white, stroke: P.white, width: WHITE, join: "round" })
}
function mandible(s) {
  const pts = M(MAND_L, s)
  const m = blob(pts, { fill: P.orange, stroke: P.ink, width: OUT, behind: true, join: "round" })
  clip(m, () => {
    // sombra de un tono en el borde interior y en la punta
    blob(M([[425, 650], [395, 720], [410, 800], [470, 860], [520, 900], [520, 640]], s), { fill: P.orangeDark })
    // luz en el borde exterior
    brush(M([[338, 700], [334, 760], [352, 810], [395, 845]], s), { width: [4, 12, 5], color: P.orangeLight, taper: [0.3, 0.5] })
  })
  ink(m, { width: 5, weight: 2.2, color: P.ink })
}

function shellHalf(s) {
  const pts = M(SHELL_L, s)
  const sh = blob(pts, { fill: P.purpleDark, stroke: P.ink, width: OUT - 4, behind: true, join: "round" })
  clip(sh, () => {
    // la sombra de un tono queda abajo (y abajo a la derecha en la mitad derecha)
    blob(shift(pts, s > 0 ? 0 : -16, -16), { fill: P.purple })
    if (s < 0) rect(CX - 2, 200, 24, 240, { fill: P.purple })
    if (s > 0) {
      // reflejo de cristal arriba a la izquierda
      brush([[338, 322], [352, 270], [392, 246], [450, 238]], { width: [6, 16, 10, 4], color: P.purpleLight, taper: [0.3, 0.5] })
    }
  })
  ink(sh, { width: 4, weight: 2, color: P.ink })
}

// ===== 1. Pasada blanca (borde de pegatina de toda la figura) =====
part("palpo_izq", () => palpWhite(1))
part("palpo_der", () => palpWhite(-1))
part("mandibula_izq", () => mandWhite(1))
part("mandibula_der", () => mandWhite(-1))
part("cabeza", () => {
  blob(HEAD, { fill: P.white, stroke: P.white, width: WHITE })
})

// ===== 2. Piezas detrás de la cabeza =====
part("palpo_izq", () => palp(1))
part("palpo_der", () => palp(-1))
part("mandibula_izq", () => mandible(1))
part("mandibula_der", () => mandible(-1))

// ===== 3. Cabeza =====
part("cabeza", () => {
  const head = blob(HEAD, { fill: P.limeMid, stroke: P.ink, width: OUT + 4, behind: true })
  clip(head, () => {
    // cuerpo lima, la sombra queda a la derecha y abajo
    ellipse(CX - 22, 418, 322, 262, { fill: P.lime })
    // banda oscura superior
    const band = ellipse(CX, 410, 345, 280, { fill: P.limeDark })
    const cutter = ellipse(CX - 6, 452, 352, 262, { hidden: true })
    cut(band, [cutter])
    // brillos de goma
    blob([[236, 300], [262, 252], [318, 226], [334, 238], [300, 280], [264, 338], [240, 350]], { fill: alpha(P.limeLight, 0.95) })
    circle(238, 402, 15, { fill: alpha(P.limeLight, 0.9) })
    // hueco oscuro bajo el caparazón (se ve al abrirse)
    const shellAll = [[CX, 212], ...SHELL_L.slice(1, -1), [CX, 480], ...M(SHELL_L, -1).slice(1, -1).reverse()]
    blob(grow(shellAll, 0.96), { fill: "#163018" })
  })
  ink(head, { width: 5, weight: 2.4, color: P.ink })

  // ojos
  for (const s of [1, -1]) {
    const ex = CX + s * (358 - CX), ey = 574
    const sock = ellipse(ex, ey + 2, 108, 92, { fill: radial(ex, ey, 108, [P.socket, "#1f4f1d"]), stroke: P.ink, width: OUT, behind: true })
    ink(sock, { width: 4, weight: 2, color: P.ink })
    const eye = circle(ex, ey - 2, 66, { fill: radial(ex - 16, ey - 22, 80, [[0, "#ff5a4a"], [0.45, P.red], [1, P.redDark]]), stroke: "#4a0a0e", width: 8 })
    clip(eye, () => {
      ellipse(ex + 10, ey + 34, 60, 26, { fill: alpha(P.redDark, 0.55) })
    })
    circle(ex - 18, ey - 24, 18, { fill: P.white })
    circle(ex + 20, ey + 14, 7, { fill: alpha(P.white, 0.9) })
  }
})

// ===== 4. Caparazón =====
part("caparazon_izq", () => shellHalf(1))
part("caparazon_der", () => shellHalf(-1))

// ===== Acciones =====
action("morder", { duration: 0.6, loop: true }, [
  [0, { mandibula_izq: 0, mandibula_der: 0, palpo_izq: 0, palpo_der: 6 }],
  [0.25, { palpo_izq: -10, palpo_der: 0 }],
  [0.5, { mandibula_izq: 24, mandibula_der: -24, palpo_der: -10 }],
  [0.6, { mandibula_izq: -3, mandibula_der: 3 }],
  [0.7, { palpo_izq: 8 }],
  [0.75, { mandibula_izq: 0, mandibula_der: 0 }],
  [0.85, { palpo_der: 10 }],
  [1, { mandibula_izq: 0, mandibula_der: 0, palpo_izq: 0, palpo_der: 6 }],
])

action("golpe", { duration: 0.25, loop: false }, [
  [0, { cabeza: { rot: 0, x: 0, y: 0 }, caparazon_izq: 0, caparazon_der: 0 }],
  [0.2, { cabeza: { rot: -5, x: -12, y: -4 }, caparazon_izq: -14, caparazon_der: 14 }],
  [0.45, { cabeza: { rot: 4, x: 10, y: 3 }, caparazon_izq: -10, caparazon_der: 10 }],
  [0.7, { cabeza: { rot: -2, x: -4, y: 0 }, caparazon_izq: -3, caparazon_der: 3 }],
  [1, { cabeza: { rot: 0, x: 0, y: 0 }, caparazon_izq: 0, caparazon_der: 0 }],
])
