// dibujo.js: nave con módulo Lanza (vista cenital, morro hacia arriba), con esqueleto.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1b2140",
  white: "#f4f7fc", lav: "#aab5d8", lavD: "#8b97c3",
  teal: "#1db5a7", tealD: "#117583", tealL: "#8eebd9",
  cyan: "#5fe3fa", blue: "#2f8fe0", crystal: "#e9f8ff",
  orange: "#ff7d1f", orangeD: "#9b3b10", yellow: "#ffc61e", cream: "#fff3c4",
  noz: "#2c3058", nozL: "#4c5590",
}

// La nave se dibuja en coordenadas de la referencia y se encoge un 88 %
// para dejar sitio a la boca del cañón por delante del morro.
const S = 0.88
const T = (x, y) => [512 + (x - 512) * S, 512 + (y - 512) * S - 25]
const G = (fn) => group({ move: [0, -25] }, () => group({ scale: S, around: [512, 512] }, fn))
const BORDER = { fill: "#ffffff", stroke: "#ffffff", width: 36, join: "round" }
const INK = { width: 10, weight: 1.6, color: P.ink }

// ---------- Geometría (coordenadas de referencia) ----------
const FUS = [[512, 48, 0], [543, 120], [577, 232], [603, 370], [614, 540], [613, 700], [603, 790], [572, 834], [512, 848], [452, 834], [421, 790], [411, 700], [410, 540], [421, 370], [447, 232], [481, 120]]
const WING = [[495, 322, 0], [252, 492, 0], [252, 728, 0], [495, 728, 0]]
const podPts = (cx) => [[0, 435], [44, 445], [64, 490], [66, 600], [62, 760], [52, 818, 0.5], [0, 826], [-48, 818, 0.5], [-60, 760], [-66, 600], [-62, 490], [-44, 445]].map(([dx, y, s]) => s === undefined ? [cx + dx, y] : [cx + dx, y, s])
const PLATE = [[462, 392, 0], [424, 390, 0], [376, 424, 0.3], [368, 446, 0.4], [368, 556, 0.4], [382, 574, 0.3], [462, 580, 0]]
// media banda que cruza el casco desde el eje y dobla hacia abajo en el borde como lengüeta corta
const CLAMP = [[514, 162, 0], [463, 161, 0.5], [455, 166], [448, 194], [453, 201, 0.5], [514, 201, 0]]
const mx = (pts) => pts.map(([x, y, s]) => s === undefined ? [1024 - x, y] : [1024 - x, y, s])

// ---------- Esqueleto ----------
rig({ origin: T(512, 1000) })
bone("fuselaje", { at: T(512, 540) })
bone("ala_izq", { parent: "fuselaje", at: T(470, 450) })
bone("ala_der", { parent: "fuselaje", at: T(554, 450) })
bone("gondola_izq", { parent: "ala_izq", at: T(255, 600) })
bone("gondola_der", { parent: "ala_der", at: T(769, 600) })
bone("llama_izq", { parent: "fuselaje", at: T(437, 905) })
bone("llama_centro", { parent: "fuselaje", at: T(512, 918) })
bone("llama_der", { parent: "fuselaje", at: T(587, 905) })
bone("canon", { parent: "fuselaje", at: T(512, 480) })
bone("abrazadera_izq", { parent: "fuselaje", at: T(480, 177) })
bone("abrazadera_der", { parent: "fuselaje", at: T(544, 177) })
bone("placa_izq", { parent: "fuselaje", at: T(470, 450) })
bone("placa_der", { parent: "fuselaje", at: T(554, 450) })

// ---------- Borde blanco de pegatina (una pasada por hueso, detrás de todo) ----------
const canonSil = (st) => {
  rect(492, 4, 12, 872, { ...st, r: 6 })
  rect(520, 4, 12, 872, { ...st, r: 6 })
  rect(500, 30, 24, 840, { ...st, r: 10 })
  ellipse(512, 6, 15, 17, st)
}
const nozzles = (st) => {
  rect(404, 826, 66, 86, { ...st, r: 12 })
  rect(554, 826, 66, 86, { ...st, r: 12 })
  rect(461, 842, 102, 82, { ...st, r: 12 })
}
layer("borde", () => {
  part("ala_izq", () => G(() => poly(WING, BORDER)))
  part("ala_der", () => G(() => poly(mx(WING), BORDER)))
  part("gondola_izq", () => G(() => blob(podPts(196), BORDER)))
  part("gondola_der", () => G(() => blob(podPts(828), BORDER)))
  part("canon", () => G(() => canonSil(BORDER)))
  part("fuselaje", () => G(() => { blob(FUS, BORDER); nozzles(BORDER) }))
  part("abrazadera_izq", () => G(() => blob(CLAMP, BORDER)))
  part("abrazadera_der", () => G(() => blob(mx(CLAMP), BORDER)))
})

// ---------- Piezas de color, de atrás hacia delante ----------
function wing() {
  const w = poly(WING, { fill: P.teal })
  clip(w, () => {
    blob([[240, 642], [410, 632], [520, 630], [520, 760], [240, 760]], { fill: P.tealD })
    brush([[268, 524], [345, 470], [430, 410]], { width: 12, color: P.tealL, taper: [0.5, 0.2] })
    brush([[300, 520], [360, 478], [400, 452]], { width: 6, color: alpha(P.tealL, 0.7) })
  })
  ink(w, INK)
}
function pod(cx) {
  const p = blob(podPts(cx), { fill: P.white })
  clip(p, () => {
    blob([[cx + 16, 430], [cx + 90, 430], [cx + 90, 850], [cx - 90, 850], [cx - 90, 744], [cx + 2, 738], [cx + 24, 700], [cx + 22, 560]], { fill: P.lav })
    rect(cx - 80, 690, 160, 26, { fill: P.teal })
    rect(cx + 20, 690, 80, 26, { fill: P.tealD, opacity: 0.6 })
    brush([[cx - 50, 520], [cx - 54, 620], [cx - 50, 700]], { width: 7, color: "#ffffff", taper: [0.4, 0.4] })
  })
  ink(p, INK)
  ellipse(cx, 489, 37, 27, { fill: P.ink })
  const intake = ellipse(cx, 487, 28, 18, { fill: P.orange })
  clip(intake, () => {
    ellipse(cx, 498, 26, 13, { fill: P.orangeD })
    brush([[cx - 14, 477], [cx, 473], [cx + 14, 477]], { width: 4, color: "#ffd7a8", taper: [0.3, 0.3] })
  })
}
function plate() {
  const p = blob(PLATE, { fill: P.white })
  clip(p, () => {
    blob([[360, 530], [400, 548], [470, 548], [470, 600], [360, 600]], { fill: P.lav })
    blob([[360, 380], [470, 380], [470, 420], [400, 412], [372, 440]], { fill: "#ffffff" })
    for (const x of [384, 402]) {
      brush([[x, 458 - (x - 384) * 0.5], [x, 500], [x, 538]], { width: 6, color: P.cyan, taper: [0.15, 0.15], glow: { blur: 6, color: P.cyan } })
    }
    brush([[386, 432], [410, 416], [436, 400]], { width: 5, color: P.cyan, taper: [0.2, 0.2], glow: { blur: 6, color: P.cyan } })
  })
  ink(p, { ...INK, width: 8 })
  circle(393, 556, 6, { fill: P.orange, stroke: P.ink, width: 4 })
}

layer("alas", () => {
  part("ala_izq", () => G(wing))
  part("ala_der", () => G(() => flip(512, wing)))
})
layer("gondolas", () => {
  part("gondola_izq", () => G(() => pod(196)))
  part("gondola_der", () => G(() => pod(828)))
})
layer("placas", () => {
  part("placa_izq", () => G(plate))
  part("placa_der", () => G(() => flip(512, plate)))
})
function canon() {
  // cuerpo central entre los raíles
  // cañón largo y fino que corre bajo el fuselaje, de la boca (y 4) a la popa (y 876):
  // desde arriba solo asoman la boca y los raíles junto a la punta del morro.
  const body = rect(500, 30, 24, 840, { fill: P.tealD, r: 10 })
  clip(body, () => {
    brush([[512, 40], [512, 860]], { width: 4, color: P.cyan, taper: [0.05, 0.05], glow: { blur: 6, color: P.cyan } })
  })
  ink(body, { ...INK, width: 5 })
  // dos raíles paralelos
  for (const x of [492, 520]) {
    const r = rect(x, 4, 12, 872, { fill: P.white, r: 6 })
    clip(r, () => {
      rect(x + 7, 0, 6, 880, { fill: P.lav })
      rect(x + 2, 18, 3, 840, { fill: "#ffffff" })
    })
    ink(r, { ...INK, width: 5 })
  }
  // puente de la boca y cristal, por delante de la punta del morro (y 48)
  const br = rect(488, 16, 48, 14, { fill: P.white, r: 5 })
  clip(br, () => rect(488, 24, 48, 8, { fill: P.lav }))
  ink(br, { ...INK, width: 5 })
  circle(512, 6, 20, { fill: alpha(P.cyan, 0.45), blur: 8 })
  const c = ellipse(512, 6, 11, 13, { fill: P.crystal, glow: { blur: 9, color: P.cyan } })
  clip(c, () => {
    ellipse(518, 12, 8, 10, { fill: "#a9e6fb" })
    ellipse(508, 1, 3, 4, { fill: "#ffffff" })
  })
  ink(c, { ...INK, width: 5 })
}
function flame(x, top, w, tip) {
  ellipse(x, (top + tip) / 2 + 20, w * 1.4, (tip - top) / 2 + 10, { fill: alpha(P.orange, 0.45), blur: 14 })
  blob([[x - w, top], [x + w, top], [x + w * 0.9, top + 40], [x, tip, 0], [x - w * 0.9, top + 40]], { fill: P.orange })
  blob([[x - w * 0.5, top], [x + w * 0.5, top], [x + w * 0.45, top + 40], [x, tip - (tip - top) * 0.3, 0], [x - w * 0.45, top + 40]], { fill: P.yellow })
}
function fuselaje() {
  // toberas
  for (const [x, y, w, h] of [[404, 826, 66, 86], [554, 826, 66, 86], [461, 842, 102, 82]]) {
    const n = rect(x, y, w, h, { fill: P.noz, r: 12 })
    clip(n, () => {
      rect(x + 10, y, 9, h, { fill: P.nozL })
      rect(x, y + h - 14, w, 14, { fill: shade(P.noz, -0.4) })
    })
    ink(n, { ...INK, width: 8 })
  }
  const f = blob(FUS, { fill: P.white })
  clip(f, () => {
    // franja verde azulada en chevron del morro
    poly([[380, 206], [512, 184], [650, 206], [650, 124], [512, 104], [380, 124]], { fill: P.teal })
    poly([[546, 90], [650, 90], [650, 215], [546, 196]], { fill: P.tealD, opacity: 0.45 })
    // sombra lavanda a la derecha y abajo
    blob([[566, 170, 0], [640, 170], [640, 880], [380, 880], [395, 790], [470, 772], [560, 768], [592, 700], [598, 520], [588, 360]], { fill: P.lav })
    brush([[436, 340], [424, 520], [428, 720]], { width: 8, color: "#ffffff", taper: [0.4, 0.4] })
  })
  ink(f, INK)
  // cabina
  ellipse(512, 314, 52, 102, { fill: P.ink })
  const cab = blob([[512, 216], [545, 240], [556, 310], [552, 380], [530, 406], [494, 406], [472, 380], [468, 310], [479, 240]], { fill: P.cyan })
  clip(cab, () => {
    blob([[540, 230], [570, 240], [570, 420], [520, 420], [542, 380], [550, 310]], { fill: P.blue })
    brush([[494, 262], [486, 290], [486, 312]], { width: 12, color: "#ffffff", taper: [0.3, 0.5] })
    circle(498, 346, 5, { fill: "#ffffff" })
  })
  // núcleo amarillo
  circle(512, 540, 55, { fill: P.ink })
  circle(512, 540, 45, { fill: P.cream })
  circle(512, 540, 39, { fill: radial(500, 525, 50, [P.yellow, "#f5960c"]) })
  circle(499, 527, 10, { fill: "#ffffff" })
  circle(527, 554, 4, { fill: "#ffffff" })
}
function abrazadera() {
  const c = blob(CLAMP, { fill: P.white })
  clip(c, () => {
    rect(440, 190, 80, 14, { fill: P.lav })
    // lengüeta doblada sobre el borde del casco, en sombra
    poly([[440, 150], [466, 150], [459, 210], [440, 210]], { fill: P.lavD })
    rect(474, 150, 6, 60, { fill: P.cyan })
    rect(486, 150, 6, 60, { fill: P.cyan })
    brush([[468, 167], [490, 166], [512, 166]], { width: 3, color: "#ffffff", taper: [0.3, 0.3] })
  })
  ink(c, { ...INK, width: 6 })
  circle(502, 181, 4.5, { fill: P.lavD, stroke: P.ink, width: 3 })
}

layer("canon", () => {
  part("canon", () => G(canon))
})
layer("llamas", () => {
  part("llama_izq", () => G(() => flame(437, 840, 22, 962)))
  part("llama_centro", () => G(() => flame(512, 852, 30, 998)))
  part("llama_der", () => G(() => flame(587, 840, 22, 962)))
})

// ---------- Acciones ----------
const rest = { canon: { x: 0, y: 0 }, abrazadera_izq: { rot: 0, x: 0, y: 0 }, abrazadera_der: { rot: 0, x: 0, y: 0 }, placa_izq: { rot: 0, x: 0, y: 0 }, placa_der: { rot: 0, x: 0, y: 0 } }

const fold = (k) => ({
  ala_izq: -26 * k, ala_der: 26 * k,
  gondola_izq: 16 * k, gondola_der: -16 * k,
  placa_izq: -26 * k, placa_der: 26 * k,
})
action("plegar", { duration: 1, loop: false }, [
  [0, fold(0)], [0.5, fold(0.5)], [1, fold(1)],
])

const fl = (a, b, c) => ({ llama_izq: { y: a }, llama_centro: { y: b }, llama_der: { y: c } })
action("motor", { duration: 0.4, loop: true }, [
  [0, fl(0, 34, 17)], [0.25, fl(17, 17, 34)], [0.5, fl(34, 0, 17)], [0.75, fl(17, 17, 0)], [1, fl(0, 34, 17)],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ...fold(0), ...fl(0, 0, 0) }],
  [0.45, { ...fold(0.6), ...fl(50, 60, 50) }],
  [1, { ...fold(0.45), ...fl(46, 56, 46) }],
])

// acoplar: el cañón entra desde atrás, luego las abrazaderas y por último las placas, cada uno con rebote.
const apart = {
  canon: { x: 0, y: 260 },
  abrazadera_izq: { rot: -35, x: -130, y: 30 }, abrazadera_der: { rot: 35, x: 130, y: 30 },
  placa_izq: { rot: -20, x: -120, y: 0 }, placa_der: { rot: 20, x: 120, y: 0 },
}
const A = (o) => ({ ...apart, ...o })
action("acoplar", { duration: 0.7, loop: false }, [
  [0, apart],
  [0.25, A({ canon: { x: 0, y: -14 } })],
  [0.35, A({ canon: { x: 0, y: 0 } })],
  [0.5, A({ canon: { x: 0, y: 0 }, abrazadera_izq: { rot: 4, x: 8, y: 0 }, abrazadera_der: { rot: -4, x: -8, y: 0 } })],
  [0.6, A({ canon: { x: 0, y: 0 }, abrazadera_izq: { rot: 0, x: 0, y: 0 }, abrazadera_der: { rot: 0, x: 0, y: 0 } })],
  [0.8, { ...rest, placa_izq: { rot: 3, x: 8, y: 0 }, placa_der: { rot: -3, x: -8, y: 0 } }],
  [1, rest],
])
layer("fuselaje", () => {
  part("fuselaje", () => G(fuselaje))
})
layer("abrazaderas", () => {
  part("abrazadera_izq", () => G(abrazadera))
  part("abrazadera_der", () => G(() => flip(512, abrazadera)))
})
