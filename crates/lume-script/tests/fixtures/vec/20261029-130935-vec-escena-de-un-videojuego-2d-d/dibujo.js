// Escena "Los Anillos": atardecer en superficie, abismo con un anillo que brilla y un leviatán dormido.
canvas({ width: 1920, height: 1080, background: "#030a18" })

const W = 1920, H = 1080
const HZ = 300      // horizonte
const SURF = 400    // línea de superficie delantera
const SUN = [470, 288]

const P = {
  skyTop: "#23255e", skyMid: "#7a4a8c", skyRose: "#d0607e", skyLow: "#f7a65a", sun: "#fff1c8",
  farLand: "#a87f9f", seaFar: "#c99bb0", seaMid: "#7f86b0", surf: "#3f7f9e",
  w0: "#3a8f9a", w1: "#1a5574", w2: "#0b2b4b", w3: "#04132a", w4: "#01050d",
  cyan: "#7dffe0", amber: "#ffb347", buoy: "#e0503a",
  stone: "#2c5d73", rock: "#081a2a", kelp: "#0b2c2e"
}

// ola: puntos de una línea ondulada entre x0 y x1
function waveLine(y, amp, len, x0 = -20, x1 = W + 20, ph = 0, step = 20) {
  const pts = []
  for (let x = x0; x <= x1; x += step) pts.push([x, y + Math.sin(x / len * Math.PI * 2 + ph) * amp + Math.sin(x / (len * 0.43) + ph * 2) * amp * 0.35])
  return pts
}

layer("cielo", { parallax: 0.02 }, () => {
  rect(0, 0, W, SURF, { fill: linear(0, 0, 0, HZ + 10, [[0, P.skyTop], [0.38, P.skyMid], [0.7, P.skyRose], [1, P.skyLow]]) })
  // halo del sol
  clip(rect(0, 0, W, SURF, { hidden: true }), () => {
    circle(SUN[0], SUN[1], 520, { fill: radial(SUN[0], SUN[1], 520, [[0, alpha("#ffd89a", 0.75)], [0.35, alpha("#ff9a6a", 0.3)], [1, alpha("#ff9a6a", 0)]]) })
  })
  element("sol", () => circle(SUN[0], SUN[1], 46, { fill: P.sun, glow: { blur: 30, color: "#ffd27a" } }))
  clip(rect(0, 0, W, SURF, { hidden: true }), () => grain({ opacity: 0.05, size: 1 }))
  // primeras estrellas arriba
  seed(3)
  for (let i = 0; i < 60; i++) {
    const x = rand(0, W), y = rand(0, 120)
    if (Math.hypot(x - SUN[0], y - SUN[1]) < 500) continue
    circle(x, y, rand(0.8, 2), { fill: "#fff6e8", opacity: rand(0.25, 0.8) * (1 - y / 140) })
  }
})

// cúmulo: bóvedas con base plana a la altura by; luz cálida del lado del sol
function cloud(cx, by, w, h, c, sunSide = -1) {
  const shapes = []
  const n = Math.max(4, Math.round(w / (h * 0.7)))
  clip(rect(cx - w, by - h * 2.5, w * 2, h * 2.5, { hidden: true }), () => {
    for (let i = 0; i < n; i++) {
      const t = i / (n - 1)
      const x = cx - w / 2 + t * w
      const peak = Math.sin(t * Math.PI) * 0.75 + 0.25
      const r = h * (0.45 + peak * 0.55) * rand(0.8, 1.15)
      shapes.push(ellipse(x, by - r * 0.55, r * 1.15, r, { fill: c.base }))
    }
    clip(shapes, () => {
      rect(cx - w, by - h * 1.2, w * 2, h * 1.3, { fill: linear(0, by - h * 1.2, 0, by, [alpha(c.under, 0), alpha(c.under, 0.9)]) })
      const lx = cx + sunSide * w * 0.6
      circle(lx, by, w * 0.8, { fill: radial(lx, by, w * 0.8, [[0, alpha(c.lit, 0.9)], [0.5, alpha(c.lit, 0.35)], [1, alpha(c.lit, 0)]]) })
      rect(cx - w, by - 6, w * 2, 8, { fill: c.edge, opacity: 0.7, blur: 3 })
    })
  })
}

layer("nubes lejanas", { parallax: 0.08 }, () => {
  seed(11)
  const c = { base: "#b77095", under: "#9a5a88", lit: "#f6b07a", edge: "#ffcf96" }
  // franjas aplanadas cerca del horizonte
  for (const [i, [x, y, w]] of [[180, 262, 300], [880, 270, 420], [1350, 258, 360], [1760, 266, 260], [620, 248, 200]].entries())
    element("nube_lejana_" + (i + 1), () => cloud(x, y, w, w * 0.09, c, x < SUN[0] ? 1 : -1))
  // cirros finos
  for (let i = 0; i < 9; i++) {
    const y = rand(60, 190), x = rand(0, W), l = rand(160, 420)
    brush([[x, y], [x + l * 0.5, y - rand(2, 8)], [x + l, y + rand(-4, 4)]], { width: rand(3, 8), color: alpha("#f3a6a0", rand(0.25, 0.5)) })
  }
})

layer("nubes cercanas", { parallax: 0.15 }, () => {
  seed(21)
  const c = { base: "#8a4f86", under: "#5e3a74", lit: "#ff9e6e", edge: "#ffd08a" }
  element("nube_1", () => cloud(1150, 205, 520, 95, c))
  element("nube_2", () => cloud(1660, 160, 380, 80, c))
  element("nube_3", () => cloud(90, 150, 300, 60, c, 1))
})

layer("horizonte", { parallax: 0.2 }, () => {
  const land = "#9c7499"
  seed(31)
  // islas lejanas (izquierda) y un acantilado al fondo derecha
  blob(roughen([[0, HZ + 2], [0, HZ - 22], [60, HZ - 30], [140, HZ - 18], [210, HZ - 12], [260, HZ + 2]], 4, { detail: 3 }), { fill: land })
  blob([[1600, HZ + 2], [1650, HZ - 16, 0.5], [1730, HZ - 34], [1820, HZ - 40], [1930, HZ - 28], [1930, HZ + 2]], { fill: mix(land, P.seaFar, 0.15) })
  // el anillo medio hundido en el horizonte
  element("anillo", () => {
  const rc = [1440, HZ + 6]
  const ringO = circle(rc[0], rc[1], 46, { fill: mix(land, "#5d4a7a", 0.25) })
  cut(ringO, [circle(rc[0], rc[1], 33, { hidden: true }), rect(rc[0] - 60, HZ + 1, 120, 60, { hidden: true }), poly([[rc[0] + 30, rc[1] - 50], [rc[0] + 50, rc[1] - 50], [rc[0] + 50, rc[1] - 20]], { hidden: true })])
  // pequeño brillo de runa
  circle(rc[0] - 38, rc[1] - 22, 2.2, { fill: P.cyan, glow: { blur: 6, color: P.cyan }, opacity: 0.9 })
  })
  // bruma
  rect(0, HZ - 30, W, 34, { fill: linear(0, HZ - 30, 0, HZ + 4, [alpha("#f7b07a", 0), alpha("#f7b07a", 0.35)]) })
})

layer("mar lejano", { parallax: 0.3 }, () => {
  rect(0, HZ, W, 50, { fill: linear(0, HZ, 0, HZ + 45, [P.seaFar, mix(P.seaFar, P.seaMid, 0.45)]) })
  // reflejo del sol: columna de destellos
  seed(41)
  element("destellos_sol", { parallax: 0.02 }, () => {  // siempre bajo el sol: se mueve como el cielo
    for (let i = 0; i < 26; i++) {
      const y = HZ + 3 + i * 1.6 + rand(0, 1.5)
      const w = (90 - i * 2) * rand(0.5, 1.1)
      rect(SUN[0] - w / 2 + rand(-12, 12), y, w, 1.6, { fill: "#fff1c8", opacity: rand(0.4, 0.9) * (1 - i / 30) })
    }
  })
  for (let i = 0; i < 40; i++) {
    const y = rand(HZ + 6, HZ + 44), x = rand(0, W)
    rect(x, y, rand(20, 60), 1.2, { fill: "#f4d0d0", opacity: 0.35 })
  }
})

layer("mar medio", { parallax: 0.5 }, () => {
  const top = HZ + 34
  blob([[-20, top + 6], ...waveLine(top, 2, 140, -20, W + 20, 0.5, 30), [W + 20, top + 6], [W + 20, SURF + 10], [-20, SURF + 10]].map((p, i, a) => i === 0 || i === a.length - 1 || i >= a.length - 3 ? [...p, 0] : p),
    { fill: linear(0, top, 0, SURF, [P.seaMid, mix(P.seaMid, P.surf, 0.6)]) })
  seed(51)
  // reflejo del sol más ancho y roto
  element("reflejo_sol", { parallax: 0.02 }, () => {
    for (let i = 0; i < 22; i++) {
      const y = top + 4 + i * 2.6
      const w = (140 + i * 4) * rand(0.4, 1)
      brush([[SUN[0] - w / 2 + rand(-25, 25), y], [SUN[0] + w / 2 + rand(-25, 25), y]], { width: rand(1.5, 3), color: alpha("#ffe0a8", rand(0.4, 0.85)), taper: [0.4, 0.4] })
    }
  })
  // crestas de olas
  for (let i = 0; i < 70; i++) {
    const y = rand(top + 6, SURF - 8), x = rand(0, W), l = rand(25, 70) * (0.6 + (y - top) / 60)
    brush([[x, y], [x + l / 2, y - 1.5], [x + l, y]], { width: 2, color: alpha("#d9b6d2", 0.45) })
    brush([[x + 4, y + 3], [x + l - 4, y + 3]], { width: 2, color: alpha("#3e4f86", 0.35) })
  }
})

layer("agua", { parallax: 0.6 }, () => {
  const water = rect(0, SURF - 6, W, H - SURF + 6, { fill: linear(0, SURF, 0, H, [[0, P.w0], [0.16, P.w1], [0.45, P.w2], [0.75, P.w3], [1, P.w4]]) })
  clip(water, () => {
    // luz del sol que entra por arriba a la izquierda
    circle(SUN[0] + 120, SURF - 40, 620, { fill: radial(SUN[0] + 120, SURF - 40, 620, [[0, alpha("#9fe6cf", 0.4)], [0.5, alpha("#9fe6cf", 0.12)], [1, alpha("#9fe6cf", 0)]]) })
    // cáusticas: redes onduladas bajo la superficie
    seed(61)
    for (let r = 0; r < 4; r++) {
      const y0 = SURF + 14 + r * 22
      for (let x = -40; x < W; x += rand(50, 90)) {
        const l = rand(40, 80)
        curve([[x, y0 + rand(-6, 6)], [x + l * 0.5, y0 + rand(-10, 10)], [x + l, y0 + rand(-6, 6)]], { stroke: "#c8fff0", width: 2 - r * 0.4, opacity: 0.22 - r * 0.045, blend: "screen" })
      }
    }
    // el abismo: viñeta oscura abajo
    rect(0, H - 380, W, 380, { fill: linear(0, H - 380, 0, H, [alpha(P.w4, 0), alpha(P.w4, 0.9)]) })
    grain({ opacity: 0.07, size: 1.2 })
  })
})

layer("fondo lejano", { parallax: 0.45 }, () => {
  const ink = "#0a2a48"
  // ruinas lejanas: columnas y arcos casi perdidos
  seed(71)
  element("ruinas", () => {
    for (const [x, h, w] of [[1580, 300, 34], [1660, 220, 30], [1760, 360, 40], [1850, 260, 34]]) {
      const top = 860 - h
      poly([[x, 860], [x + w * 0.1, top + 10], [x + w * 0.4, top], [x + w * 0.9, top + 14], [x + w, 860]], { fill: mix(P.w2, ink, 0.4), opacity: 0.7 })
    }
    path("M 1594 640 Q 1625 590 1662 650 L 1662 668 Q 1625 616 1598 662 Z", { fill: mix(P.w2, ink, 0.4), opacity: 0.6 })
  })
  // lecho lejano
  blob([[-20, 1100, 0], [-20, 860], [300, 830], [700, 870], [1100, 840], [1500, 870], [1940, 840], [1940, 1100, 0]], { fill: mix(P.w3, ink, 0.3), opacity: 0.85 })

  // el leviatán: un cuerpo colosal enroscado, apenas más oscuro que el agua
  element("leviatan", () => group({ move: [190, 80] }, () => {
  const body = "#051a33"
  const lev = blob([
    [140, 820], [260, 740], [430, 705], [610, 720], [760, 770], [930, 800], [1150, 790], [1340, 830], [1480, 900],
    [1440, 930], [1260, 890], [1060, 880], [880, 900], [700, 880], [520, 860], [380, 900], [250, 940], [150, 910]
  ], { fill: body, opacity: 0.92 })
  clip(lev, () => {
    // luz cenital muy tenue sobre el lomo
    blob([[200, 780], [430, 712], [620, 728], [930, 808], [1340, 838], [1340, 860], [930, 832], [600, 760], [420, 745], [220, 805]], { fill: "#2a6488", opacity: 0.45, blur: 12 })
    // placas del lomo
    for (let i = 0; i < 14; i++) {
      const t = i / 13, x = lerp(260, 1380, t)
      const y = 760 + Math.sin(t * 3.1) * -40 + t * 70
      brush([[x - 20, y + 18], [x, y], [x + 22, y + 16]], { width: 4, color: alpha("#2a5f80", 0.25) })
    }
  })
  // mandíbula: una línea larga que insinúa la boca
  brush([[160, 870], [260, 880], [380, 868]], { width: [2, 5, 2], color: alpha("#000814", 0.6) })
  // el ojo: lo único que se ve con claridad
  circle(300, 812, 46, { fill: radial(300, 812, 46, [[0, alpha("#7fd4ff", 0.45)], [1, alpha("#7fd4ff", 0)]]) })
  ellipse(300, 812, 16, 9, { fill: "#cdeeff", opacity: 0.9 })
  ellipse(302, 812, 3, 8, { fill: "#061d38" })
  // fotóforos del flanco
  for (let i = 0; i < 16; i++) {
    const t = i / 15, x = lerp(420, 1300, t)
    const y = 815 + Math.sin(t * 3.1) * -30 + t * 75
    circle(x, y, 2.4, { fill: "#8fe9ff", opacity: 0.55 - t * 0.25, glow: { blur: 8, color: "#5fd0ff" } })
  }
  }))
  // velo de agua por encima: lo empuja hacia lo hondo
  rect(0, 560, W, 520, { fill: linear(0, 560, 0, H, [alpha(P.w2, 0), alpha(P.w3, 0.3)]) })
})

layer("zona media", { parallax: 0.75 }, () => {
  const stone = "#1c4458"
  const RC = [1330, 690], RO = 160, RI = 112
  seed(81)

  // lecho medio derecho donde descansa el anillo
  const bed = blob(roughen([[960, 1100, 0], [1000, 900], [1120, 850], [1260, 835], [1420, 830], [1560, 855], [1700, 880], [1940, 870], [1940, 1100, 0]], 8, { detail: 3 }),
    { fill: linear(0, 830, 0, 1000, ["#123448", "#030c18"]) })
  clip(bed, () => {
    brush([[1020, 895], [1200, 845], [1420, 836], [1700, 884]], { width: 6, color: alpha("#5fb0b8", 0.35), blur: 2 })
  })

  // rocas sobre el lecho
  element("rocas", () => {
  for (const [x, w, h] of [[1200, 120, 70], [1330, 160, 60], [1460, 110, 80]]) {
    const rk = blob(roughen([[x - w / 2, 860], [x - w / 2 + 10, 860 - h * 0.7], [x - w * 0.1, 860 - h], [x + w * 0.3, 860 - h * 0.85], [x + w / 2, 860]], 6), { fill: "#163b52" })
    clip(rk, () => rect(x - w / 2, 860 - h * 0.4, w, h, { fill: "#0a2236", opacity: 0.6, blur: 10 }))
    brush([[x - w / 2 + 8, 860 - h * 0.6], [x - w * 0.1, 860 - h + 4], [x + w * 0.25, 860 - h * 0.85]], { width: 4, color: alpha("#6fd8d0", 0.4) })
  }
  })

  // terraza
  const ter = blob(roughen([[-20, 600], [120, 560], [300, 568], [360, 600], [330, 680], [150, 720], [-20, 760]], 10, { detail: 3 }), { fill: linear(0, 560, 0, 760, ["#1f5a6e", "#0d2c44"]) })
  clip(ter, () => brush([[0, 600], [140, 566], [300, 572], [356, 600]], { width: 8, color: alpha("#7ad6c8", 0.35), blur: 2 }))
  // mantiene la secuencia aleatoria de lo que viene después
  for (let i = 0; i < 18; i++) rand()
  // banco de peces que gira hacia el anillo
  element("banco_peces", () => {
  for (let i = 0; i < 26; i++) {
    const t = i / 25, a = t * 200 - 40
    const x = 1010 + Math.cos(a * Math.PI / 180) * 120 + rand(-20, 20)
    const y = 560 + Math.sin(a * Math.PI / 180) * 45 + rand(-14, 14)
    const s = rand(0.7, 1.2), dir = Math.sin(a * Math.PI / 180) > 0 ? -1 : 1
    blob([[x - 10 * s * dir, y], [x, y - 4 * s], [x + 8 * s * dir, y, 0.5], [x, y + 4 * s]], { fill: "#123f58", opacity: 0.85 })
    poly([[x + 7 * s * dir, y], [x + 13 * s * dir, y - 4 * s], [x + 13 * s * dir, y + 4 * s]], { fill: "#123f58", opacity: 0.85 })
    line(x - 6 * s * dir, y - 1, x + 2 * s * dir, y - 2.5 * s, { stroke: "#6fc6c8", width: 1, opacity: 0.6 })
  }
  })
  // la grieta: hacia donde baja todo
  blob([[760, 1100, 0], [800, 1030], [900, 1000], [1000, 1010], [1060, 1060], [1080, 1100, 0]], { fill: "#01040b", opacity: 0.9, blur: 10 })
})

layer("rayos de luz", { parallax: 0.9 }, () => {
  seed(91)
  for (let i = 0; i < 9; i++) {
    const x = rand(80, 1500) + (i < 3 ? 0 : 0), w = rand(30, 110), drift = 260
    const len = rand(450, 680)
    element("rayo_" + (i + 1), { pivot: [x + w / 2, SURF] }, () => poly([[x, SURF], [x + w, SURF], [x + w * 2.2 + drift, SURF + len], [x + drift - w * 0.3, SURF + len]],
      { fill: linear(0, SURF, 0, SURF + len, [alpha("#d8fff0", 0.17), alpha("#d8fff0", 0)]), blend: "screen", blur: 10 }))
  }
})

layer("particulas", { parallax: 1 }, () => {
  seed(101)
  for (let i = 0; i < 260; i++) {
    const y = SURF + Math.pow(rand(), 1.4) * (H - SURF)
    const x = rand(0, W)
    const d = (y - SURF) / (H - SURF)
    circle(x, y, rand(0.8, 2.6), { fill: mix("#e6fff6", "#5fb8d0", d), opacity: rand(0.2, 0.7) * (1 - d * 0.7) })
  }
  // burbujas que suben desde la grieta
  element("burbujas", () => {
    for (let i = 0; i < 14; i++) {
      const y = rand(560, 1000), x = 900 + Math.sin(y / 60) * 20 + rand(-20, 20)
      circle(x, y, rand(2, 5), { fill: "none", stroke: "#a8f0ff", width: 1.2, opacity: 0.45 })
    }
  })
})

layer("superficie", { parallax: 1 }, () => {
  // banda de la superficie delantera: lámina de agua vista de canto
  const top = waveLine(SURF - 10, 3, 220, -20, W + 20, 1.3, 16)
  const bot = waveLine(SURF + 8, 2.5, 220, -20, W + 20, 1.3, 16).reverse()
  const band = blob([...top, ...bot].map(p => [...p, 0.6]), { fill: linear(0, SURF - 14, 0, SURF + 10, ["#5aa6b4", "#2f8592"]) })
  clip(band, () => {
    // brillo del sol sobre la lámina
    element("brillo_sol", { parallax: 0.02 }, () => ellipse(SUN[0], SURF - 4, 380, 20, { fill: "#ffe6b0", opacity: 0.55, blur: 8 }))
  })
  // borde aire-agua: línea nítida y brillante
  curve(top, { stroke: "#e9fff6", width: 2.5, opacity: 0.9 })
  brush(top.filter(p => Math.abs(p[0] - SUN[0]) < 360), { width: 4, color: alpha("#fff2c8", 0.9), taper: [0.4, 0.4] })
  // menisco inferior: el espejo plateado bajo la superficie
  curve(bot, { stroke: "#bff6ee", width: 2, opacity: 0.55 })
  seed(111)
  for (let i = 0; i < 60; i++) {
    const x = rand(0, W), y = SURF + rand(10, 22), l = rand(20, 70)
    brush([[x, y], [x + l, y + rand(-2, 2)]], { width: rand(1, 2.2), color: alpha("#b6f2ea", rand(0.15, 0.4)) })
  }
})

// roca con luz de borde arriba-izquierda (cenital)
function rock(pts, base, rim, k = 8) {
  const rp = roughen(pts, k, { detail: 3 })
  const r = blob(rp, { fill: rim })
  clip(r, () => {
    blob(shift(rp, 3, 5), { fill: base })
    blob(shift(grow(rp, 0.85), 30, 60), { fill: shade(base, -0.5), opacity: 0.6, blur: 20 })
  })
  return r
}

function kelp(x, y, h, sway, wmax, col) {
  const n = 8, pts = []
  for (let i = 0; i <= n; i++) {
    const t = i / n
    pts.push([x + Math.sin(t * 3 + sway) * 24 * t + sway * 30 * t * t, y - t * h])
  }
  brush(pts, { width: u => wmax * (0.4 + Math.sin(u * Math.PI) * 0.8), color: col, taper: [0, 0.5] })
  // hojitas
  for (let i = 2; i < n; i++) {
    const [px, py] = pts[i], s = i % 2 ? 1 : -1
    brush([[px, py], [px + s * 14, py - 10], [px + s * 26, py - 4]], { width: [1, 6, 1], color: col })
  }
  return pts
}

function coralFan(x, y, s, col) {
  const branch = (bx, by, ang, len, d) => {
    const ex = bx + Math.cos(ang) * len, ey = by + Math.sin(ang) * len
    brush([[bx, by], [(bx + ex) / 2 + rand(-4, 4), (by + ey) / 2], [ex, ey]], { width: [d * 1.6, d], color: col, taper: [0, 0.3] })
    if (d > 1.4) {
      branch(ex, ey, ang - rand(0.25, 0.5), len * 0.75, d * 0.7)
      branch(ex, ey, ang + rand(0.25, 0.5), len * 0.75, d * 0.7)
    } else circle(ex, ey, 2.2, { fill: mix(col, "#ff7fb0", 0.6), glow: { blur: 4, color: "#ff5fa0" }, opacity: 0.8 })
  }
  branch(x, y, -Math.PI / 2, 40 * s, 6 * s)
}

layer("primer plano", { parallax: 1.3 }, () => {
  seed(121)
  const base = "#06121f", rim = "#2f7f8c"
  // macizo izquierdo
  element("roca_izquierda", () => {
    rock([[-30, 470, 0], [40, 455], [110, 500], [150, 560], [140, 640], [210, 700], [250, 790], [230, 880], [300, 960], [420, 1010], [560, 1030], [700, 1060], [720, 1100, 0], [-30, 1100, 0]], base, rim, 10)
    rock([[40, 760], [150, 728], [235, 765], [250, 850], [200, 900], [40, 900]], "#081726", rim, 6)
  })
  // macizo derecho
  element("roca_derecha", () => {
    rock([[1960, 500, 0], [1880, 520], [1800, 580], [1770, 660], [1720, 720], [1700, 820], [1640, 900], [1520, 960], [1380, 1010], [1180, 1050], [1120, 1100, 0], [1960, 1100, 0]], base, rim, 10)
    rock([[1560, 1000], [1640, 930], [1740, 915], [1800, 960], [1790, 1040], [1600, 1050]], "#081726", rim, 6)
  })
  // algas que se mecen
  const kc = "#0a2a2c"
  // cada alga gira desde su base, para mecerla
  for (const [i, [x, y, h, s, w]] of [[40, 1090, 420, 0.4, 14], [110, 1080, 330, -0.3, 11], [190, 1085, 260, 0.6, 9], [330, 1050, 200, 0.2, 8],
                                [1860, 1090, 460, -0.5, 14], [1780, 1080, 340, 0.3, 11], [1700, 1085, 240, -0.2, 9], [1480, 1030, 170, 0.5, 7]].entries())
    element("alga_" + (i + 1), { pivot: [x, y] }, () => kelp(x, y, h, s, w, kc))
  // corales con puntas luminosas
  element("coral_1", { pivot: [380, 1000] }, () => coralFan(380, 1000, 1.1, "#1a0f2a"))
  element("coral_2", { pivot: [600, 1055] }, () => coralFan(600, 1055, 0.8, "#1a0f2a"))
  element("coral_3", { pivot: [1330, 1035] }, () => coralFan(1330, 1035, 1.0, "#1a0f2a"))
  element("coral_4", { pivot: [1620, 960] }, () => coralFan(1620, 960, 0.9, "#1a0f2a"))
  // anémonas: pequeños acentos de vida
  for (const [n, [x, y]] of [[250, 965], [640, 1050], [1250, 1050], [1560, 975]].entries())
    element("anemona_" + (n + 1), { pivot: [x, y] }, () => {
      for (let i = 0; i < 9; i++) {
        const a = -160 + i * 17.5
        brush([[x, y], polar(x, y, rand(14, 22), a + 90)], { width: [3, 1], color: "#4f2a5e" })
      }
      ellipse(x, y + 2, 9, 5, { fill: "#2a1636" })
    })
  // conchas y piedras sueltas con luz cenital
  for (let i = 0; i < 18; i++) {
    const x = rand() < 0.5 ? rand(300, 700) : rand(1150, 1650)
    const y = x < 900 ? lerp(990, 1060, (x - 300) / 400) : lerp(1050, 960, (x - 1150) / 500)
    ellipse(x, y + rand(0, 20), rand(4, 10), rand(3, 5), { fill: "#0d2232" })
  }
  // viñeta inferior propia
  rect(0, 980, W, 100, { fill: linear(0, 980, 0, H, [alpha("#000308", 0), alpha("#000308", 0.7)]) })
})
