// dibujo.js: nave con módulo Contraviento (vista cenital, mirando hacia arriba)
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1c2340",
  white: "#f3f6fb",
  lav: "#a9b5d6",
  teal: "#1bb5a7",
  tealD: "#127482",
  tealL: "#8eeedd",
  glass: "#5fe3fb",
  glassD: "#2a8fe0",
  core: "#f7a10e",
  coreL: "#ffd23f",
  noz: "#2b2f55",
  nozL: "#4a5085",
  flame: "#ff7a1a",
  flameL: "#ffd84a",
  acc: "#ff5a26",
  accD: "#c2381a",
  accL: "#ff9a5c",
  intakeD: "#9c3a12",
}
const B = "#ffffff" // borde de pegatina
const BW = 56
const INK = (s, w = 11) => ink(s, { width: w, weight: 1.6, color: P.ink })
const mx = (pts) => pts.map(([x, y, k]) => (k === undefined ? [1024 - x, y] : [1024 - x, y, k]))

// ---------- geometría ----------
const HULL = [[512, 56, 0], [548, 125], [578, 240], [600, 400], [614, 560], [620, 700], [614, 790], [590, 832], [512, 850], [434, 832], [410, 790], [404, 700], [410, 560], [424, 400], [446, 240], [476, 125]]
const COCKPIT = [[512, 214], [548, 240], [560, 305], [556, 370], [512, 396], [468, 370], [464, 305], [476, 240]]
const WING_L = [[488, 380], [425, 370], [256, 486], [260, 728], [488, 728]]
const WING_R = mx(WING_L)
const gond = (cx) => [[cx, 434], [cx + 48, 447], [cx + 66, 520], [cx + 69, 640], [cx + 65, 760], [cx + 51, 822], [cx, 829], [cx - 51, 822], [cx - 65, 760], [cx - 69, 640], [cx - 66, 520], [cx - 48, 447]]
const GL = 197, GR = 827
const FIN_L = [[446, 722], [390, 750], [338, 828], [314, 898, 0], [376, 902, 0], [412, 862], [446, 852]]
const FIN_R = mx(FIN_L)
const CLAMP_L = [395, 758, 34, 70]
const flame = (cx, w, tip) => [[cx - w, 850], [cx + w, 850], [cx + w * 0.95, 900], [cx + w * 0.55, tip - (tip - 900) * 0.35], [cx, tip, 0], [cx - w * 0.55, tip - (tip - 900) * 0.35], [cx - w * 0.95, 900]]
const TUR = [512, 458]

// ---------- esqueleto ----------
rig({ origin: [512, 590] })
bone("fuselaje", { at: [512, 590] })
bone("ala_izq", { parent: "fuselaje", at: [432, 388] })
bone("ala_der", { parent: "fuselaje", at: [592, 388] })
bone("gondola_izq", { parent: "ala_izq", at: [258, 600] })
bone("gondola_der", { parent: "ala_der", at: [766, 600] })
bone("llama_izq", { parent: "fuselaje", at: [435, 905] })
bone("llama_centro", { parent: "fuselaje", at: [512, 915] })
bone("llama_der", { parent: "fuselaje", at: [589, 905] })
bone("aleta_izq", { parent: "fuselaje", at: [420, 790] })
bone("aleta_der", { parent: "fuselaje", at: [604, 790] })
bone("torreta", { parent: "fuselaje", at: TUR })

// ---------- pasada 1: borde blanco de pegatina (detrás de todo) ----------
layer("borde", () => {
  const W = { fill: B, stroke: B, width: BW, join: "round" }
  part("llama_izq", () => blob(flame(435, 22, 958), { ...W, width: 40 }))
  part("llama_der", () => blob(flame(589, 22, 958), { ...W, width: 40 }))
  part("llama_centro", () => blob(flame(512, 30, 994), { ...W, width: 40 }))
  part("ala_izq", () => poly(WING_L, W))
  part("ala_der", () => poly(WING_R, W))
  part("gondola_izq", () => blob(gond(GL), W))
  part("gondola_der", () => blob(gond(GR), W))
  part("aleta_izq", () => { blob(FIN_L, W); rect(...CLAMP_L, { ...W, r: 12 }) })
  part("aleta_der", () => { blob(FIN_R, W); rect(1024 - CLAMP_L[0] - CLAMP_L[2], CLAMP_L[1], CLAMP_L[2], CLAMP_L[3], { ...W, r: 12 }) })
  part("fuselaje", () => {
    blob(HULL, W)
    rect(404, 832, 216, 92, { ...W, r: 12 })
  })
  part("torreta", () => {
    circle(...TUR, 50, W)
    rect(TUR[0] - 16, TUR[1] - 78, 32, 156, { ...W, r: 10 })
  })
})

// ---------- pasada 2: piezas en color, de atrás hacia delante ----------
layer("llamas", () => {
  const fl = (bn, cx, w, tip) => part(bn, () => {
    blob(flame(cx, w, tip), { fill: P.flame, glow: { blur: 18, color: "#ff8a3acc" } })
    blob(flame(cx, w * 0.5, tip - (tip - 850) * 0.32), { fill: P.flameL })
    blob(flame(cx - w * 0.15, w * 0.18, tip - (tip - 850) * 0.55), { fill: "#fff8d8", opacity: 0.9 })
  })
  fl("llama_izq", 435, 22, 958)
  fl("llama_der", 589, 22, 958)
  fl("llama_centro", 512, 30, 994)
})

const wing = (pts, s) => {
  const w = poly(pts, { fill: P.teal })
  const m = s < 0 ? (p) => p : mx
  clip(w, () => {
    poly(m([[240, 643], [500, 628], [500, 740], [240, 740]]), { fill: P.tealD })
    // franja de luz junto al borde de ataque
    poly(m([[425, 382], [256, 498], [256, 520], [425, 410]]), { fill: P.tealL, opacity: 0.85 })
    brush(m([[392, 440], [330, 488], [292, 516]]), { width: [2, 9, 2], color: P.tealL })
    brush(m([[378, 468], [330, 500], [302, 520]]), { width: [1, 5, 1], color: alpha(P.tealL, 0.7) })
    // juntas de panel
    brush(m([[270, 600], [405, 590]]), { width: 3, color: alpha(P.tealD, 0.55), taper: [0.1, 0.1] })
  })
  INK(w, 12)
}
layer("alas", () => {
  part("ala_izq", () => wing(WING_L, -1))
  part("ala_der", () => wing(WING_R, 1))
})

const gondola = (cx) => {
  const g = blob(gond(cx), { fill: P.white })
  clip(g, () => {
    blob([[cx + 18, 440], [cx + 90, 440], [cx + 90, 840], [cx + 10, 840], [cx + 22, 740], [cx + 24, 600], [cx + 16, 500]], { fill: P.lav })
    blob([[cx - 80, 748], [cx + 20, 744], [cx + 40, 735], [cx + 90, 760], [cx + 90, 850], [cx - 80, 850]], { fill: P.lav })
    rect(cx - 80, 690, 160, 26, { fill: P.teal })
    rect(cx + 20, 690, 60, 26, { fill: "#14968f" })
    brush([[cx - 50, 530], [cx - 54, 620], [cx - 52, 700]], { width: [1, 5, 1], color: "#ffffff" })
    // intake
    ellipse(cx - 1, 487, 37, 27, { fill: P.ink })
    const it = ellipse(cx - 1, 487, 27, 18, { fill: P.flame })
    clip(it, () => {
      ellipse(cx - 1, 498, 30, 14, { fill: P.intakeD })
      ellipse(cx - 6, 478, 14, 4, { fill: "#ffd0a0" })
    })
  })
  INK(g, 12)
}
layer("gondolas", () => {
  part("gondola_izq", () => gondola(GL))
  part("gondola_der", () => gondola(GR))
})
const fin = (s) => {
  const m = s < 0 ? (p) => p : mx
  const X = (x) => (s < 0 ? x : 1024 - x)
  // emisor trasero en la punta
  const e = rect(X(345) - 21, 868, 42, 54, { fill: P.noz, r: 11 })
  clip(e, () => {
    rect(X(345) - 15, 868, 9, 54, { fill: P.nozL, r: 3 })
    rect(X(345) - 21, 910, 42, 12, { fill: shade(P.noz, -0.35) })
  })
  INK(e, 9)
  ellipse(X(345), 920, 14, 6, { fill: P.flame, glow: { blur: 10, color: "#ff8a3a" } })
  ellipse(X(345), 919, 8, 3, { fill: P.flameL })
  const f = blob(m(FIN_L), { fill: P.acc })
  clip(f, () => {
    // sombra de un tono en la cara que se aparta de la luz
    poly(m([[450, 785], [384, 868], [376, 910], [470, 910]]), { fill: P.accD })
    brush(m([[436, 735], [388, 764], [346, 836], [328, 884]]), { width: [2, 9, 3], color: P.accL })
    brush(m([[442, 808], [380, 852]]), { width: 3, color: alpha(P.accD, 0.8), taper: [0.1, 0.1] })
    circle(X(398), 790, 6, { fill: P.accD })
    circle(X(397), 789, 2.5, { fill: P.accL })
    circle(X(360), 860, 5, { fill: shade(P.accD, -0.3) })
  })
  INK(f, 11)
}
layer("aletas", () => {
  part("aleta_izq", () => fin(-1))
  part("aleta_der", () => fin(1))
})

layer("fuselaje", () => {
  part("fuselaje", () => {
    // toberas: laterales detrás, central delante
    const noz = (x, y, w, h) => {
      const n = rect(x, y, w, h, { fill: P.noz, r: 10 })
      clip(n, () => {
        rect(x + 8, y, 9, h, { fill: P.nozL, r: 3 })
        rect(x, y + h - 14, w, 14, { fill: shade(P.noz, -0.35) })
      })
      INK(n, 10)
    }
    noz(404, 832, 62, 80)
    noz(558, 832, 62, 80)
    noz(460, 842, 104, 82)

    const h = blob(HULL, { fill: P.white })
    clip(h, () => {
      // morro verde azulado
      poly([[380, 208], [512, 184], [644, 208], [644, 140], [512, 106], [380, 140]], { fill: P.teal })
      poly([[540, 120], [644, 140], [644, 210], [540, 190]], { fill: "#14968f" })
      // sombra lavanda a la derecha y en la popa
      blob([[586, 250], [640, 240], [650, 860], [560, 860], [590, 780], [598, 600], [590, 420]], { fill: P.lav })
      blob([[390, 790], [470, 778], [560, 772], [600, 760], [640, 780], [640, 880], [390, 880]], { fill: P.lav })
      brush([[440, 330], [426, 470], [420, 620], [430, 760]], { width: [1, 6, 2], color: "#ffffff" })
      // juntas de módulos (encaje del arma)
      brush([[430, 728], [594, 728]], { width: 3, color: alpha(P.lav, 0.9), taper: [0.05, 0.05] })
      // ranuras de anclaje de las aletas
      rect(398, 764, 24, 58, { fill: P.noz, r: 8 })
      rect(602, 764, 24, 58, { fill: P.noz, r: 8 })
    })
    INK(h, 13)

    // cabina
    const c = blob(COCKPIT, { fill: P.glass })
    clip(c, () => {
      blob([[536, 230], [570, 240], [570, 380], [516, 400], [540, 370], [548, 300]], { fill: P.glassD })
      brush([[494, 262], [484, 290], [482, 316]], { width: [3, 10, 4], color: "#ffffff", taper: [0.3, 0.3] })
      circle(497, 345, 5, { fill: "#ffffff" })
    })
    INK(c, 12)

    // zócalo de la torreta (se ve cuando la torreta se separa)
    circle(...TUR, 38, { fill: P.noz, stroke: P.ink, width: 8 })
    circle(...TUR, 24, { fill: "none", stroke: P.nozL, width: 4 })
    circle(...TUR, 8, { fill: P.nozL })

    // núcleo amarillo
    circle(512, 590, 54, { fill: P.ink })
    circle(512, 590, 45, { fill: P.tealD })
    const k = circle(512, 590, 39, { fill: radial(500, 575, 48, [P.coreL, P.core, "#e07a08"]) })
    clip(k, () => {
      circle(512, 590, 37, { fill: "none", stroke: "#fff3b0", width: 3 })
    })
    circle(498, 576, 9, { fill: "#ffffff" })
    circle(527, 602, 4, { fill: "#ffffff" })
  })
})

layer("abrazaderas", () => {
  // placas que fijan las aletas sobre el casco
  const clamp = (bn, x) => part(bn, () => {
    const c = rect(x, CLAMP_L[1], CLAMP_L[2], CLAMP_L[3], { fill: P.acc, r: 12 })
    clip(c, () => {
      rect(x + CLAMP_L[2] - 10, CLAMP_L[1], 12, CLAMP_L[3], { fill: P.accD })
      rect(x + 5, CLAMP_L[1] + 6, 6, CLAMP_L[3] - 12, { fill: P.accL, r: 3 })
    })
    INK(c, 9)
    circle(x + CLAMP_L[2] / 2, CLAMP_L[1] + 16, 4.5, { fill: P.ink })
    circle(x + CLAMP_L[2] / 2, CLAMP_L[1] + CLAMP_L[3] - 16, 4.5, { fill: P.ink })
  })
  clamp("aleta_izq", CLAMP_L[0])
  clamp("aleta_der", 1024 - CLAMP_L[0] - CLAMP_L[2])
})

layer("torreta", () => {
  part("torreta", () => {
    const [cx, cy] = TUR
    // dos cañones opuestos: delante (arriba) y detrás (abajo)
    for (const d of [-1, 1]) {
      const L = 76
      const y0 = d < 0 ? cy - L : cy
      const b = rect(cx - 15, y0, 30, L, { fill: P.white, r: 9 })
      clip(b, () => {
        rect(cx + 4, y0, 14, L, { fill: P.lav })
        rect(cx - 16, d < 0 ? cy - L + 10 : cy + L - 20, 32, 10, { fill: P.acc })
      })
      INK(b, 10)
      const my = cy + d * (L - 5)
      ellipse(cx, my, 11, 6, { fill: P.flame, glow: { blur: 8, color: "#ff8a3a" } })
      ellipse(cx, my, 6, 3, { fill: P.flameL })
    }
    // anillo rojo anaranjado
    const r = circle(cx, cy, 48, { fill: P.acc })
    clip(r, () => {
      circle(cx + 14, cy + 14, 48, { fill: P.accD })
      circle(cx - 5, cy - 5, 44, { fill: P.acc })
      brush(arc(cx, cy, 40, 230, 330, 8), { width: [1, 6, 1], color: P.accL })
    })
    INK(r, 12)
    for (const a of [45, 135, 225, 315]) {
      const [bx, by] = polar(cx, cy, 37, a)
      circle(bx, by, 4, { fill: P.ink })
    }
    // cúpula central
    const dm = circle(cx, cy, 26, { fill: P.white })
    clip(dm, () => circle(cx + 10, cy + 10, 24, { fill: P.lav }))
    INK(dm, 8)
    circle(cx, cy, 10, { fill: P.teal, stroke: P.ink, width: 4 })
    circle(cx - 11, cy - 11, 4.5, { fill: "#ffffff" })
  })
})

// ---------- acciones ----------
// plegar: ala izquierda gira en sentido antihorario (negativo) para irse hacia atrás
action("plegar", { duration: 1, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, gondola_izq: 0, gondola_der: 0 }],
  [0.5, { ala_izq: -9, ala_der: 9, gondola_izq: 8, gondola_der: -8 }],
  [1, { ala_izq: -19, ala_der: 19, gondola_izq: 16, gondola_der: -16 }],
])

// motor: las llamas salen y se esconden bajo las toberas, desfasadas
action("motor", { duration: 0.4, loop: true }, range(9).map((i) => {
  const t = i / 8
  const f = (ph, a) => ({ y: Math.round(a * 0.5 + a * Math.sin(2 * Math.PI * (t + ph))) })
  return [t, { llama_izq: f(0, 20), llama_centro: f(0.33, 26), llama_der: f(0.66, 20) }]
}))

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, gondola_izq: 0, gondola_der: 0, llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 } }],
  [0.4, { ala_izq: -12, ala_der: 12, gondola_izq: 9, gondola_der: -9, llama_izq: { y: 34 }, llama_centro: { y: 46 }, llama_der: { y: 34 } }],
  [1, { ala_izq: -8, ala_der: 8, gondola_izq: 6, gondola_der: -6, llama_izq: { y: 40 }, llama_centro: { y: 52 }, llama_der: { y: 40 } }],
])

// acoplar: aleta izquierda, aleta derecha y por último la torreta, con rebote
const AI = { rot: 42, x: -130, y: 30 }, AD = { rot: -42, x: 130, y: 30 }
const TU = { rot: 180, x: 0, y: -440 }
action("acoplar", { duration: 0.7, loop: false }, [
  [0, { aleta_izq: AI, aleta_der: AD, torreta: TU }],
  [0.1, { aleta_izq: AI, aleta_der: AD, torreta: { rot: 180, x: 0, y: -420 } }],
  [0.3, { aleta_izq: { rot: -4, x: 6, y: -3 }, aleta_der: AD }],
  [0.4, { aleta_izq: { rot: 0, x: 0, y: 0 }, torreta: { rot: 180, x: 0, y: -260 } }],
  [0.52, { aleta_der: { rot: 4, x: -6, y: -3 } }],
  [0.62, { aleta_der: { rot: 0, x: 0, y: 0 } }],
  [0.85, { torreta: { rot: -8, x: 0, y: 5 } }],
  [1, { aleta_izq: { rot: 0, x: 0, y: 0 }, aleta_der: { rot: 0, x: 0, y: 0 }, torreta: { rot: 0, x: 0, y: 0 } }],
])
