// Nave con módulo Taladro: vista desde arriba, mirando hacia arriba, con esqueleto para Godot.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  ink: "#1c2341",      // contorno
  white: "#f4f7fc",    // casco
  shade: "#a9b5d6",    // sombra del casco
  teal: "#19b3a7",     // alas
  tealD: "#0e7180",    // banda oscura del ala
  tealM: "#139092",
  tealL: "#94eede",    // brillos del ala
  glass: "#5fe5fc", glassD: "#2189d0",
  gold: "#fdc52a", orange: "#fc7f17", ember: "#9a3a0e", cream: "#ffecdd",
  nozzle: "#2c3156", nozzleL: "#4f5989",
  turq: "#22f0d8",     // turquesa del arma
  turqD: "#0fb3ae",
  metal: "#2b3258", metalL: "#46508a",
}
const W = 12          // grosor del contorno
const B = 34          // grosor del borde blanco de pegatina
const OUT = { stroke: P.ink, width: W, join: "round" }
const M = (pts, s) => (s > 0 ? pts.map((p) => [1024 - p[0], ...p.slice(1)]) : pts)

// ---------- geometría compartida (pieza y borde de pegatina) ----------
const FUS = [[480, 172, 0], [462, 240], [445, 320], [428, 410], [413, 520], [406, 640], [410, 750], [428, 815], [468, 845], [512, 852], [556, 845], [596, 815], [614, 750], [618, 640], [611, 520], [596, 410], [579, 320], [562, 240], [544, 172, 0]]
const WING = [[470, 360, 0], [425, 372, 0], [252, 492, 0], [252, 728, 0], [470, 728, 0]]
const GOND = (cx) => [[cx, 435], [cx + 54, 455], [cx + 66, 520], [cx + 62, 700], [cx + 52, 812, 0.5], [cx, 822], [cx - 51, 812, 0.5], [cx - 62, 700], [cx - 66, 520], [cx - 54, 455]]
const PLACA = [[436, 368, 0.4], [326, 444, 0.5], [312, 468, 0.5], [310, 580, 0.5], [328, 598, 0.5], [430, 598, 0.4]]
const ARM = [[430, 196, 0.5], [462, 196, 0.5], [447, 320], [432, 420], [423, 482], [408, 498], [393, 482], [401, 420], [415, 320]]
const DRILL = [[512, 34, 0], [558, 112, 0.7], [584, 204, 0], [440, 204, 0], [466, 112, 0.7]]
const FLAME = (cx, top, len, w) => [[cx - w, top], [cx + w, top], [cx + w * 0.9, top + len * 0.55], [cx, top + len, 0], [cx - w * 0.9, top + len * 0.55]]
const COLLAR = [438, 158, 148, 60]

// ---------- esqueleto ----------
rig({ origin: [512, 540] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [432, 450] })
bone("ala_der", { parent: "fuselaje", at: [592, 450] })
bone("gondola_izq", { parent: "ala_izq", at: [196, 600] })
bone("gondola_der", { parent: "ala_der", at: [828, 600] })
bone("llama_izq", { parent: "fuselaje", at: [437, 905] })
bone("llama_centro", { parent: "fuselaje", at: [512, 915] })
bone("llama_der", { parent: "fuselaje", at: [587, 905] })
bone("broca", { parent: "fuselaje", at: [512, 200] })
bone("collarin", { parent: "fuselaje", at: [512, 188] })
bone("brazo_izq", { parent: "fuselaje", at: [446, 205] })
bone("brazo_der", { parent: "fuselaje", at: [578, 205] })
bone("placa_izq", { parent: "fuselaje", at: [432, 450] })
bone("placa_der", { parent: "fuselaje", at: [592, 450] })

// ---------- borde blanco de pegatina (detrás de todo) ----------
layer("pegatina", () => {
  const S = { fill: "#ffffff", stroke: "#ffffff", width: W + B, join: "round" }
  part("ala_izq", () => blob(WING, S))
  part("ala_der", () => blob(M(WING, 1), S))
  part("gondola_izq", () => blob(GOND(196), S))
  part("gondola_der", () => blob(GOND(828), S))
  part("fuselaje", () => {
    blob(FUS, S)
    rect(400, 826, 224, 100, { ...S, r: 14 })
  })
  part("broca", () => blob(DRILL, S))
  part("collarin", () => rect(...COLLAR, { ...S, r: 22 }))
  part("brazo_izq", () => blob(ARM, S))
  part("brazo_der", () => blob(M(ARM, 1), S))
  part("placa_izq", () => blob(PLACA, S))
  part("placa_der", () => blob(M(PLACA, 1), S))
})

// ---------- llamas (detrás de las toberas) ----------
layer("llamas", () => {
  const flame = (bn, cx, top, len, w) => part(bn, () => {
    blob(FLAME(cx, top, len, w), { fill: P.orange, glow: { blur: 18, color: alpha(P.orange, 0.7) } })
    blob(FLAME(cx, top, len * 0.72, w * 0.55), { fill: P.gold })
    blob(FLAME(cx, top, len * 0.42, w * 0.25), { fill: "#fff6c8" })
  })
  flame("llama_izq", 437, 850, 112, 22)
  flame("llama_der", 587, 850, 112, 22)
  flame("llama_centro", 512, 860, 132, 30)
})

// ---------- alas y góndolas ----------
layer("alas", () => {
  const wing = (s) => {
    const w = blob(M(WING, s), { fill: P.teal })
    clip(w, () => {
      poly(M([[200, 636], [520, 628], [520, 760], [200, 760]], s), { fill: P.tealD })
      // brillos del borde de ataque
      blob(M([[420, 398, 0], [262, 512, 0], [300, 515, 0], [418, 425, 0]], s), { fill: P.tealL, opacity: 0.9 })
      blob(M([[388, 452, 0], [300, 520, 0], [318, 520, 0], [395, 462, 0]], s), { fill: P.tealL, opacity: 0.7 })
    })
    blob(M(WING, s), { fill: "none", ...OUT })
  }
  const gond = (cx) => {
    const g = blob(GOND(cx), { fill: P.white })
    clip(g, () => {
      blob([[cx + 14, 445], [cx + 80, 445], [cx + 80, 840], [cx - 80, 840], [cx - 80, 748], [cx - 30, 745], [cx + 18, 728], [cx + 22, 620], [cx + 18, 520]], { fill: P.shade })
      rect(cx - 80, 690, 160, 26, { fill: P.teal })
      rect(cx + 20, 690, 60, 26, { fill: P.tealM })
      brush([[cx - 46, 520], [cx - 50, 600], [cx - 48, 680]], { width: 6, color: "#ffffff", taper: [0.4, 0.4] })
    })
    blob(GOND(cx), { fill: "none", ...OUT })
    // toma de aire
    ellipse(cx, 486, 36, 26, { fill: P.orange, ...OUT, width: 10 })
    clip(ellipse(cx, 486, 31, 21, { hidden: true }), () => {
      ellipse(cx, 498, 34, 14, { fill: P.ember })
      brush([[cx - 18, 474], [cx, 469], [cx + 18, 474]], { width: 5, color: P.cream })
    })
  }
  part("ala_izq", () => wing(-1))
  part("gondola_izq", () => gond(196))
  part("ala_der", () => wing(1))
  part("gondola_der", () => gond(828))
})

// ---------- placas de blindaje del arma ----------
layer("placas", () => {
  const placa = (s) => {
    const p = blob(M(PLACA, s), { fill: P.white })
    clip(p, () => {
      // cara inferior en sombra y bisel
      poly(M([[300, 562], [460, 562], [460, 620], [300, 620]], s), { fill: P.shade })
      // líneas de energía turquesa
      const glow = { glow: { blur: 8, color: alpha(P.turq, 0.8) } }
      brush(M([[400, 410], [338, 454]], s), { width: 9, color: P.turq, taper: [0.15, 0.15], ...glow })
      brush(M([[334, 476], [334, 540]], s), { width: 9, color: P.turq, taper: [0.2, 0.2], ...glow })
      brush(M([[354, 484], [392, 484]], s), { width: 7, color: P.turq, taper: [0.1, 0.4], ...glow })
      brush(M([[354, 512], [384, 512]], s), { width: 7, color: P.turq, taper: [0.1, 0.4], ...glow })
      brush(M([[354, 540], [376, 540]], s), { width: 7, color: P.turq, taper: [0.1, 0.4], ...glow })
      line(...M([[318, 560], [404, 560]], s).flat(), { stroke: alpha(P.ink, 0.6), width: 4 })
    })
    blob(M(PLACA, s), { fill: "none", ...OUT, width: 11 })
    for (const [x, y] of [[336, 580], [376, 580]]) circle(...M([[x, y]], s)[0], 6, { fill: P.ink })
  }
  part("placa_izq", () => placa(-1))
  part("placa_der", () => placa(1))
})

// ---------- fuselaje: casco, cabina, núcleo y toberas ----------
layer("fuselaje", () => part("fuselaje", () => {
  // toberas: las laterales detrás de la central
  const noz = (x, y, w, h) => {
    rect(x, y, w, h, { fill: P.nozzle, ...OUT, r: 12 })
    rect(x + 10, y + 8, 9, h - 22, { fill: P.nozzleL, r: 4 })
  }
  noz(405, 826, 66, 84)
  noz(553, 826, 66, 84)
  noz(463, 840, 98, 80)
  // casco
  const f = blob(FUS, { fill: P.white })
  clip(f, () => {
    blob([[572, 200], [598, 400], [610, 600], [600, 740], [560, 772], [470, 778], [396, 796], [396, 900], [640, 900], [640, 190]], { fill: P.shade })
    brush([[430, 450], [420, 600], [428, 740]], { width: 7, color: "#ffffff", taper: [0.4, 0.5] })
  })
  blob(FUS, { fill: "none", ...OUT })
  // cabina
  const c = ellipse(512, 312, 47, 94, { fill: P.glass, ...OUT, width: 11 })
  clip(ellipse(512, 312, 41, 88, { hidden: true }), () => {
    ellipse(560, 318, 26, 96, { fill: P.glassD })
    brush([[492, 262], [484, 290], [486, 312]], { width: 11, color: "#ffffff", taper: [0.3, 0.4] })
    circle(498, 346, 5, { fill: "#ffffff" })
  })
  // núcleo
  circle(512, 540, 52, { fill: P.ink })
  circle(512, 540, 41, { fill: radial(500, 525, 50, [P.gold, P.orange]), stroke: P.cream, width: 3 })
  circle(498, 526, 10, { fill: "#ffffff" })
  circle(526, 554, 4, { fill: "#ffffff" })
}))

// ---------- taladro: broca, brazos y collarín ----------
layer("taladro", () => {
  part("broca", () => {
    const d = blob(DRILL, { fill: P.white })
    clip(d, () => {
      // mitad derecha en sombra
      blob([[512, 20, 0], [566, 110], [596, 214, 0], [534, 214, 0], [526, 110]], { fill: P.shade })
      // espiras: cada estría es una sombra bajo un filo turquesa
      for (let k = 0; k < 5; k++) {
        const yl = 222 - k * 36, yr = yl - 50
        const edge = [[428, yl], [478, yl - 8], [540, yl - 28], [598, yr]]
        blob([...edge, ...shift(edge, 0, 16).reverse()], { fill: P.shade, opacity: 0.85 })
        curve(edge, { stroke: P.turq, width: 10, cap: "round" })
        curve(shift(edge, 0, -6), { stroke: P.ink, width: 4, cap: "round" })
        circle(478 + k * 2, yl - 9, 2.5, { fill: "#ffffff" })
      }
    })
    blob(DRILL, { fill: "none", ...OUT, width: 11 })
    circle(504, 66, 4, { fill: "#ffffff" })
  })
  const arm = (s) => {
    const a = blob(M(ARM, s), { fill: P.white })
    clip(a, () => {
      curve(M([[456, 196], [442, 320], [428, 420], [418, 490]], s), { stroke: P.shade, width: 12 })
      curve(M([[443, 210], [429, 320], [414, 420], [407, 476]], s), { stroke: P.turq, width: 8, cap: "round", glow: { blur: 6, color: alpha(P.turq, 0.7) } })
    })
    blob(M(ARM, s), { fill: "none", ...OUT, width: 10 })
    circle(...M([[408, 482]], s)[0], 6, { fill: P.ink })
  }
  part("brazo_izq", () => arm(-1))
  part("brazo_der", () => arm(1))
  part("collarin", () => {
    const [x, y, w, h] = COLLAR
    const c = rect(x, y, w, h, { fill: P.metal, r: 22 })
    clip(c, () => {
      rect(x, y, w, 18, { fill: P.metalL })
      for (let i = 0; i < 7; i++) line(x + 20 + i * 18, y + 22, x + 20 + i * 18, y + h - 8, { stroke: alpha(P.ink, 0.7), width: 4 })
      rect(x, y + h - 12, w, 12, { fill: alpha(P.ink, 0.5) })
    })
    rect(x, y, w, h, { fill: "none", ...OUT, r: 22 })
    // ranuras de energía
    for (const sx of [x + 22, x + w - 22]) {
      rect(sx - 9, y + 17, 18, 28, { fill: P.turqD, r: 6 })
      rect(sx - 6, y + 20, 12, 22, { fill: P.turq, r: 4 })
      rect(sx - 3, y + 23, 4, 10, { fill: "#ffffff", r: 2 })
    }
    brush([[x + 40, y + 10], [x + 80, y + 9]], { width: 4, color: "#ffffff", taper: [0.3, 0.3] })
  })
})

// ---------- acciones ----------
// fusiona pistas por hueso { hueso: [[t, valor], ...] } en claves [[t, pose], ...]
const tracks = (T) => {
  const keys = {}
  for (const [b, list] of Object.entries(T)) for (const [t, v] of list) (keys[t] = keys[t] || {})[b] = v
  return Object.keys(keys).map(Number).sort((a, b) => a - b).map((t) => [t, keys[t]])
}

const FOLD = 30
action("plegar", { duration: 1, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, placa_izq: 0, placa_der: 0, gondola_izq: 0, gondola_der: 0 }],
  [0.5, { ala_izq: -FOLD * 0.5, ala_der: FOLD * 0.5, placa_izq: -FOLD * 0.5, placa_der: FOLD * 0.5, gondola_izq: 2, gondola_der: -2 }],
  [1, { ala_izq: -FOLD, ala_der: FOLD, placa_izq: -FOLD, placa_der: FOLD, gondola_izq: 0, gondola_der: 0 }],
])

// llamas desfasadas un tercio de ciclo; la broca vibra a un lado y a otro
const N8 = 8
action("motor", { duration: 0.4, loop: true }, range(N8 + 1).map((i) => {
  const t = i / N8, w = (ph) => Math.round(8 + 18 * Math.sin(2 * Math.PI * (t + ph)))
  return [t, {
    llama_izq: { y: w(0) }, llama_centro: { y: w(1 / 3) }, llama_der: { y: w(2 / 3) },
    broca: [0, 3, 0, -3, 0, 3, 0, -3, 0][i],
  }]
}))

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, placa_izq: 0, placa_der: 0, llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 } }],
  [0.4, { ala_izq: -16, ala_der: 16, placa_izq: -16, placa_der: 16, llama_izq: { y: 55 }, llama_centro: { y: 70 }, llama_der: { y: 55 } }],
  [1, { ala_izq: -12, ala_der: 12, placa_izq: -12, placa_der: 12, llama_izq: { y: 48 }, llama_centro: { y: 62 }, llama_der: { y: 48 } }],
])

// las piezas encajan una a una: placas, brazos, collarín y broca, cada una con su rebote
action("acoplar", { duration: 0.7, loop: false }, tracks({
  placa_izq: [[0, { rot: -12, x: -90, y: 0 }], [0.22, { rot: 2, x: 6, y: 0 }], [0.32, { rot: 0, x: 0, y: 0 }]],
  placa_der: [[0, { rot: 12, x: 90, y: 0 }], [0.22, { rot: -2, x: -6, y: 0 }], [0.32, { rot: 0, x: 0, y: 0 }]],
  brazo_izq: [[0, { rot: 32, x: -60, y: -10 }], [0.14, { rot: 32, x: -60, y: -10 }], [0.38, { rot: -4, x: 4, y: 0 }], [0.5, { rot: 0, x: 0, y: 0 }]],
  brazo_der: [[0, { rot: -32, x: 60, y: -10 }], [0.14, { rot: -32, x: 60, y: -10 }], [0.38, { rot: 4, x: -4, y: 0 }], [0.5, { rot: 0, x: 0, y: 0 }]],
  collarin: [[0, { rot: 0, x: 0, y: -90 }], [0.3, { rot: 0, x: 0, y: -90 }], [0.55, { rot: 0, x: 0, y: 7 }], [0.68, { rot: 0, x: 0, y: 0 }]],
  broca: [[0, { rot: 0, x: 0, y: -170 }], [0.45, { rot: 0, x: 0, y: -160 }], [0.75, { rot: 0, x: 0, y: 10 }], [0.88, { rot: 0, x: 0, y: -4 }], [1, { rot: 0, x: 0, y: 0 }]],
}))
