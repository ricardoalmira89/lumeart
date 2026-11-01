// Faro rojo sobre la colina, entre nubes y montañas (estilo cómic a tinta)
canvas({ width: 720, height: 1280, background: linear(0, 0, 0, 900, ["#1a6070", "#22788a"]) })

const P = {
  ink: "#2b2327",
  sky: "#1d6878",
  cloud: "#f4ecc6",
  cloudSh: "#a9cbbf",
  cloudSh2: "#7fb0ab",
  red: "#d9352d",
  redDk: "#8f1e22",
  redLt: "#ef5a45",
  mount: "#8fb4bd",
  mountDk: "#5f8597",
  snow: "#eef0e4",
  grass: "#7d873c",
  grassDk: "#566229",
  rock: "#6e6966",
  rockLt: "#9a948c",
  dirt: "#a89b80",
  wood: "#7a6a5c",
  glass: "#3c4f57",
}
seed(7)

// Nube: círculos con contorno de tinta unificado, sombra interior y rizos
function puff(x, y, r, dark = 0) {
  const base = mix(P.cloud, "#e4e6c8", dark)
  const sh = mix(P.cloudSh, P.cloudSh2, dark)
  const c = blob(jitter(ring(x, y, r, r * 0.88, 11, rand(0, 40)), r * 0.1), { fill: base, stroke: alpha(P.ink, 0.85), width: 1.8 })
  clip(c, () => {
    blob(jitter(ring(x + r * 0.3, y + r * 0.75, r * 1.05, r * 0.65, 8), r * 0.12), { fill: sh, opacity: 0.8 })
    blob(jitter(ring(x + r * 0.5, y + r * 1.0, r * 0.75, r * 0.5, 7), r * 0.1), { fill: mix(sh, P.cloudSh2, 0.7), opacity: 0.5 + dark * 0.5 })
  })
  if (rand() < 0.7) brush(arc(x + r * 0.05, y + r * 0.05, r * 0.5, 190 + rand(-30, 20), 300 + rand(-20, 40), 6), { width: 1.5, color: alpha(P.ink, 0.55) })
  if (rand() < 0.4) brush(arc(x + r * 0.3, y + r * 0.35, r * 0.3, 120, 260, 5), { width: 1.2, color: alpha(P.ink, 0.45) })
}

// Nube: puffs dentro de una región, de arriba abajo (los de abajo encima)
function cloud(back, edge, region, n, rmin, rmax, darkY0, darkY1) {
  blob(grow(back, 0.94), { fill: linear(0, darkY0, 0, darkY1, [P.cloudSh, P.cloudSh2]) })
  const pts = [...edge]
  for (let i = 0; i < n; i++) {
    const [x0, y0, x1, y1] = region
    pts.push([rand(x0, x1), rand(y0, y1), rand(rmin, rmax)])
  }
  // racimos de puffs pequeños alrededor de cada puff grande (textura de cúmulo)
  const extra = []
  for (const [x, y, r] of pts) {
    if (rand() < 0.6) for (let k = 0; k < 2; k++) {
      const a = rand(200, 340)
      const [px, py] = polar(x, y, r * 0.75, a)
      extra.push([px, py + r * 0.4, r * rand(0.35, 0.55)])
    }
  }
  pts.push(...extra)
  pts.sort((a, b) => a[1] - b[1])
  for (const [x, y, r] of pts) puff(x, y, r, clamp((y - darkY0) / (darkY1 - darkY0), 0, 1))
}

layer("cielo", () => {
  // pájaros
  for (const [x, y, s] of [[80, 80, 1], [110, 110, 1.1], [150, 108, 0.9], [200, 455, 0.6], [215, 465, 0.5]]) {
    brush([[x - 11 * s, y + 2 * s], [x - 5 * s, y - 4 * s], [x, y], [x + 5 * s, y - 4 * s], [x + 11 * s, y + 1 * s]], { width: 2.4 * s, color: P.snow, taper: [0.4, 0.4] })
  }
})

layer("nubes", () => {
  // masa grande detrás del faro (derecha, alta)
  cloud([[345, 190], [420, 55], [560, 75], [720, 120], [720, 700], [600, 700], [470, 600], [400, 470], [350, 300]],
    [[330, 190, 32], [345, 150, 34], [380, 105, 40], [420, 60, 42], [470, 40, 40], [520, 60, 38],
    [560, 85, 36], [610, 100, 38], [660, 115, 38], [705, 140, 42], [330, 240, 34]],
    [370, 120, 720, 640], 95, 26, 48, 200, 700)
  // torre de nube a la izquierda del faro
  cloud([[215, 310], [300, 275], [400, 310], [400, 430], [240, 360]],
    [[215, 310, 30], [245, 280, 34], [290, 270, 30], [205, 350, 28]],
    [225, 300, 400, 430], 22, 24, 38, 280, 480)
  // banda baja de la izquierda
  cloud([[0, 520], [100, 475], [300, 480], [430, 520], [440, 720], [300, 750], [100, 720], [0, 720]],
    [[55, 495, 34], [105, 475, 38], [160, 470, 36], [215, 485, 34], [260, 470, 30], [20, 540, 34]],
    [10, 510, 430, 735], 70, 24, 44, 520, 760)
  // nubes a la derecha, bajas
  cloud([[600, 610], [720, 580], [720, 800], [610, 780]],
    [[640, 590, 34], [690, 575, 36], [610, 640, 30]],
    [590, 610, 720, 790], 22, 24, 40, 600, 800)
})

layer("montañas", () => {
  // bruma turquesa en el horizonte
  rect(0, 700, 720, 140, { fill: linear(0, 700, 0, 840, [alpha("#7fbcb4", 0), "#86c2b8"]) })
  // cordillera lejana con pico nevado a la izquierda
  const far = poly([[0, 790], [40, 745], [95, 712], [141, 676], [168, 700], [210, 735], [250, 772], [300, 792], [400, 802],
    [560, 796], [600, 776], [630, 760], [665, 785], [700, 770], [720, 760], [720, 1100], [0, 1100]],
    { fill: linear(0, 680, 0, 900, ["#cfe0dc", P.mount]), stroke: alpha(P.ink, 0.7), width: 2, join: "round" })
  clip(far, () => {
    // nieve del pico
    poly([[60, 760], [100, 712], [141, 676], [175, 708], [215, 745], [190, 760], [170, 745], [150, 770], [128, 748], [100, 775], [80, 760]],
      { fill: P.snow })
    // laderas en sombra (derecha de cada pico)
    poly([[141, 676], [175, 708], [250, 772], [200, 800], [160, 760]], { fill: P.mountDk, opacity: 0.35 })
    poly([[630, 760], [665, 785], [720, 800], [720, 860], [650, 820]], { fill: P.mountDk, opacity: 0.35 })
    for (let i = 0; i < 26; i++) {
      const x = rand(0, 720), y = rand(700, 860)
      brush([[x, y], [x + rand(-14, 14), y + rand(14, 34)]], { width: 1.3, color: alpha(P.mountDk, 0.6) })
    }
  })
  // cordillera media, más azul y con más contraste
  const mid = poly([[0, 860], [45, 830], [90, 845], [150, 805], [190, 838], [240, 868], [300, 890], [420, 905], [600, 890],
    [640, 850], [680, 832], [720, 845], [720, 1100], [0, 1100]],
    { fill: linear(0, 800, 0, 1050, ["#9fc2c8", "#5e8597"]), stroke: alpha(P.ink, 0.8), width: 2.2, join: "round" })
  clip(mid, () => {
    poly([[150, 805], [190, 838], [240, 868], [200, 900], [165, 850]], { fill: P.mountDk, opacity: 0.4 })
    poly([[45, 830], [90, 845], [70, 900], [40, 870]], { fill: P.mountDk, opacity: 0.35 })
    poly([[680, 832], [720, 845], [720, 900], [690, 870]], { fill: P.mountDk, opacity: 0.4 })
    // crestas y barrancos
    for (let i = 0; i < 70; i++) {
      const x = rand(0, 720), y = rand(830, 1060)
      if (x > 190 && x < 560 && y < 1000) continue
      const dir = pick([-1, 1]), len = rand(18, 50)
      brush([[x, y], [x + dir * len * 0.4, y + len * 0.5], [x + dir * len * 0.7, y + len]], { width: [0.8, 1.8, 0.6], color: alpha(rand() < 0.65 ? "#33536a" : "#e8f2ee", 0.7) })
    }
    poly([[0, 930], [60, 900], [130, 960], [80, 1060], [0, 1060]], { fill: "#3f6680", opacity: 0.35 })
    for (const [x, y] of [[150, 805], [45, 830], [680, 832], [640, 850]]) {
      brush([[x, y], [x + 8, y + 40], [x + 2, y + 90]], { width: [1, 2.4, 1], color: alpha(P.ink, 0.6) })
      brush([[x, y], [x - 12, y + 30], [x - 30, y + 70]], { width: [1, 2, 1], color: alpha("#ffffff", 0.5) })
    }
  })
})

layer("colina", () => {
  // suelo de hierba
  // el terreno sube hasta el zócalo de la casa (montículo) y cae en talud hacia la izquierda
  const ground = blob([[0, 1068, 0], [90, 1062], [150, 1046], [186, 1018, 0.5], [300, 1010], [440, 1012], [520, 990], [590, 905], [720, 912, 0], [720, 1280, 0], [0, 1280, 0]],
    { fill: P.grass, stroke: P.ink, width: 2.5 })
  clip(ground, () => {
    rect(0, 880, 720, 400, { fill: radial(560, 930, 520, ["#97a24c", "#7d873c", "#626d2e"]) })
    // manchas de hierba clara y oscura
    for (let i = 0; i < 26; i++) {
      const x = rand(0, 720), y = rand(960, 1280)
      ellipse(x, y, rand(30, 70), rand(8, 18), { fill: pick(["#99a650", "#5c6a2a", "#6f7a34"]), opacity: 0.5, blur: 4 })
    }
    // talud rocoso bajo la casa
    poly([[150, 1046], [186, 1016], [460, 1012], [470, 1040], [380, 1050], [260, 1052], [170, 1060]], { fill: "#5f5a52", opacity: 0.85 })
    for (const [x, y, w, h, t] of [[176, 1040, 34, 26, 0.2], [214, 1030, 44, 26, 0], [262, 1034, 40, 24, 0.3], [330, 1030, 48, 26, 0.1],
      [392, 1032, 44, 24, 0.2], [446, 1028, 36, 24, 0]]) rock(x, y, w, h, t)
    // sombra de la casa sobre la ladera
    ellipse(330, 1060, 230, 55, { fill: "#2e3a2a", opacity: 0.45, blend: "multiply", blur: 14 })
    // camino de tierra
    blob([[240, 1100], [330, 1095], [420, 1110], [390, 1170], [440, 1230], [470, 1280, 0], [180, 1280, 0], [230, 1210], [190, 1160]],
      { fill: P.dirt, stroke: alpha(P.ink, 0.6), width: 1.5 })
    // matas de hierba
    for (let i = 0; i < 160; i++) {
      const x = rand(0, 720), y = rand(960, 1280)
      if (x > 180 && x < 450 && y > 1110) continue
      const h = rand(6, 16) * (0.6 + (y - 960) / 400)
      brush([[x, y], [x + rand(-4, 4), y - h]], { width: 1.6, color: pick([P.grassDk, "#a3ad55", "#4f5a24"]), taper: [0, 0.8] })
    }
    // piedrecitas del camino
    for (let i = 0; i < 40; i++) {
      const x = rand(200, 440), y = rand(1110, 1280)
      ellipse(x, y, rand(2, 6), rand(1.5, 3.5), { fill: pick(["#8a7f6c", "#c3b89f", P.rock]), stroke: alpha(P.ink, 0.4), width: 0.8 })
    }
  })
})

// Roca facetada: cuerpo, cara iluminada arriba-izquierda y lado en sombra
function rock(x, y, w, h, tone = 0) {
  const base = mix(P.rock, "#8a8478", tone)
  const pts = roughen([[x - w * 0.5, y + h * 0.2, 0], [x - w * 0.3, y - h * 0.4, 0], [x + w * 0.15, y - h * 0.5, 0], [x + w * 0.5, y - h * 0.1, 0], [x + w * 0.45, y + h * 0.4, 0], [x - w * 0.4, y + h * 0.45, 0]], Math.min(w, h) * 0.08, { sharp: true })
  const r = poly(pts, { fill: base, stroke: P.ink, width: 2.2, join: "round" })
  clip(r, () => {
    poly([[x - w * 0.55, y + h * 0.1], [x - w * 0.3, y - h * 0.6], [x + w * 0.2, y - h * 0.6], [x + w * 0.1, y - h * 0.05]], { fill: P.rockLt, opacity: 0.75 })
    poly([[x + w * 0.15, y], [x + w * 0.6, y - h * 0.2], [x + w * 0.6, y + h * 0.6], [x - w * 0.5, y + h * 0.6], [x - w * 0.4, y + h * 0.3]], { fill: "#3f3a44", opacity: 0.4, blend: "multiply" })
    brush([[x - w * 0.2, y - h * 0.1], [x + w * 0.1, y + h * 0.15]], { width: 1.2, color: alpha(P.ink, 0.6) })
  })
  return r
}

layer("faro", () => {
  const cx = 489
  const redSide = (x0, x1) => linear(x0, 0, x1, 0, [[0, "#c32a26"], [0.2, P.redLt], [0.45, P.red], [0.8, "#b0262a"], [1, P.redDk]])
  // fuste troncocónico
  const tower = path(`M 408 440 L 568 440 L 584 858 Q 491 878 399 858 Z`, { fill: redSide(399, 584), stroke: P.ink, width: 3.5, join: "round" })
  clip(tower, () => {
    // costuras de las virolas (curvas por la perspectiva)
    for (const y of [560, 690, 790]) {
      const k = (y - 440) / 420, l = lerp(408, 399, k), r = lerp(568, 584, k)
      brush([[l, y], [cx, y + 9], [r, y]], { width: 2, color: alpha(P.ink, 0.7), taper: [0.05, 0.05] })
      brush([[l, y + 3], [cx, y + 12], [r, y + 3]], { width: 1.5, color: alpha(P.redLt, 0.6), taper: [0.05, 0.05] })
    }
    // manchas de óxido y desgaste
    for (let i = 0; i < 40; i++) {
      const y = rand(450, 850), x = rand(410, 575)
      ellipse(x, y, rand(1.5, 4), rand(1, 2.5), { fill: pick([P.redDk, "#7a2a22", P.redLt]), opacity: 0.7 })
    }
    for (let i = 0; i < 10; i++) {
      const x = rand(420, 560), y = rand(460, 820)
      brush([[x, y], [x + rand(-2, 2), y + rand(20, 50)]], { width: 2, color: alpha(P.redDk, 0.4) })
    }
    // sombra propia a la derecha y luz reflejada en el borde
    path(`M 530 440 L 556 440 L 572 860 L 545 860 Z`, { fill: "#5a1420", opacity: 0.35, blend: "multiply", blur: 6 })
    path(`M 562 440 L 568 440 L 584 860 L 577 860 Z`, { fill: "#ff9a80", opacity: 0.35 })
    // franja de luz a la izquierda
    path(`M 418 440 L 436 440 L 430 860 L 410 860 Z`, { fill: "#ff8a6a", opacity: 0.35, blend: "screen" })
  })
  // escalera de mano en el lado derecho
  for (const dx of [0, 10]) line(562 + dx, 470, 577 + dx, 855, { stroke: alpha(P.ink, 0.85), width: 1.6 })
  for (let y = 480; y < 850; y += 14) {
    const k = (y - 470) / 385
    line(562 + 15 * k, y, 572 + 15 * k, y, { stroke: alpha(P.ink, 0.8), width: 1.4 })
  }
  // ventanas pequeñas
  const smallWin = (x, y, w, h) => {
    rect(x - 2, y - 2, w + 4, h + 4, { fill: "#d8cfc0", stroke: P.ink, width: 1.8 })
    rect(x, y, w, h, { fill: P.glass })
    line(x, y, x + w, y + h * 0.6, { stroke: alpha("#ffffff", 0.35), width: 2 })
  }
  smallWin(434, 468, 14, 18)
  smallWin(432, 516, 18, 20)
  smallWin(425, 620, 14, 22)
  // ventana grande con tejadillo
  poly([[485, 588], [550, 588], [546, 597], [489, 597]], { fill: "#b8aa98", stroke: P.ink, width: 2 })
  rect(492, 598, 48, 58, { fill: "#d8cfc0", stroke: P.ink, width: 2 })
  rect(497, 603, 38, 48, { fill: P.glass })
  rect(497, 603, 38, 48, { fill: linear(497, 603, 535, 651, ["#6d8a91", P.glass]), opacity: 0.7 })
  line(516, 603, 516, 651, { stroke: "#d8cfc0", width: 2.5 })
  line(497, 627, 535, 627, { stroke: "#d8cfc0", width: 2.5 })
  rect(488, 655, 56, 6, { fill: "#d8cfc0", stroke: P.ink, width: 1.8 })
  // zócalo de piedra
  path(`M 396 852 Q 491 872 587 852 L 592 875 Q 491 895 392 875 Z`, { fill: "#c8c3b6", stroke: P.ink, width: 2.5 })

  // --- galería ---
  // plataforma: anillo que rodea el faro (canto y suelo)
  ellipse(cx, 448, 126, 17, { fill: "#8d8a80", stroke: P.ink, width: 3 })
  ellipse(cx, 441, 126, 15, { fill: "#d8d0bd", stroke: P.ink, width: 3 })
  // tornapuntas bajo la plataforma
  for (const [x0, x1] of [[380, 420], [600, 560]]) brush([[x0, 455], [x1, 500]], { width: 2, color: P.ink, taper: [0, 0] })
  // cuerpo cilíndrico que sube desde el suelo de la galería: su base curva tapa la mitad trasera del anillo
  const upper = path(`M 415 366 L 566 366 L 566 441 Q 490 458 415 441 Z`, { fill: redSide(415, 566), stroke: P.ink, width: 3, join: "round" })
  clip(upper, () => {
    brush([[415, 400], [cx, 409], [566, 400]], { width: 1.5, color: alpha(P.ink, 0.6) })
    path(`M 415 436 Q 490 452 566 436 L 566 446 Q 490 462 415 446 Z`, { fill: "#4a1018", opacity: 0.45, blend: "multiply" })
  })
  // puerta de servicio a la galería
  rect(430, 386, 18, 50, { fill: P.redDk, stroke: P.ink, width: 1.6, r: 2 })
  // barandilla: barra trasera, postes y barra delantera
  const railY = 402
  for (let i = 0; i <= 10; i++) {
    const a = (i / 10) * Math.PI
    const px = cx - Math.cos(a) * 122, py = 441 + Math.sin(a) * 13
    line(px, py, px, py - 39, { stroke: i % 5 ? P.redDk : P.ink, width: i % 5 ? 2.4 : 3.2 })
  }
  for (const off of [0, 20]) {
    const pts = range(21).map((i) => {
      const a = (i / 20) * Math.PI
      return [cx - Math.cos(a) * 123, railY + off + Math.sin(a) * 13]
    })
    curve(pts, { stroke: off ? P.redDk : P.red, width: off ? 2.2 : 4, cap: "round" })
    if (!off) curve(pts, { stroke: P.ink, width: 1, opacity: 0.6 })
  }

  // --- linterna ---
  rect(425, 300, 127, 68, { fill: "#2f4249", stroke: P.ink, width: 3 })
  clip(rect(425, 300, 127, 68, { fill: linear(425, 0, 552, 0, ["#4d6870", "#2c3d44", "#55737a"]) }), () => {
    circle(470, 345, 16, { fill: "#e8e2cf" })
    rect(460, 345, 22, 23, { fill: "#cfc6b0" })
    path("M 430 300 L 470 300 L 438 368 L 425 368 Z", { fill: "#ffffff", opacity: 0.18 })
    for (const x of [445, 470, 500, 528]) line(x, 300, x, 368, { stroke: "#1e2a2f", width: 3 })
    line(425, 334, 552, 334, { stroke: "#1e2a2f", width: 2.5 })
  })
  rect(422, 362, 133, 8, { fill: P.redDk, stroke: P.ink, width: 2 })
  // cúpula
  ellipse(cx, 304, 82, 10, { fill: P.redDk, stroke: P.ink, width: 3 })
  path(`M 412 300 Q 418 262 470 254 Q 489 248 508 254 Q 560 262 566 300 Z`, { fill: linear(412, 0, 566, 0, [P.red, P.redLt, P.red, P.redDk]), stroke: P.ink, width: 3 })
  brush([[430, 292], [445, 268], [470, 258]], { width: 3, color: alpha("#ffc0a0", 0.7) })
  circle(cx, 248, 8, { fill: "#c8c3b6", stroke: P.ink, width: 2 })
  brush([[cx, 242], [cx, 200], [cx + 1, 162]], { width: [4, 2.5, 1], color: P.redDk, taper: [0, 0.4] })
})

layer("cable", () => {
  // poste y cable que cuelga de la galería
  brush([[210, 758], [211, 850]], { width: [2.5, 3.5], color: "#4a3f38", taper: [0, 0] })
  line(204, 768, 217, 766, { stroke: "#4a3f38", width: 2 })
  curve([[385, 452], [330, 560], [262, 680], [210, 762]], { stroke: alpha(P.ink, 0.85), width: 1.6 })
  curve([[210, 762], [130, 870], [60, 960], [0, 1010]], { stroke: alpha(P.ink, 0.85), width: 1.6 })
  curve([[392, 452], [360, 520], [340, 600]], { stroke: alpha(P.ink, 0.6), width: 1.2 })
})

layer("casa", () => {
  const trim = "#d6cbb6"
  // chimenea
  rect(287, 748, 11, 40, { fill: "#6a5a52", stroke: P.ink, width: 2 })
  rect(284, 744, 17, 7, { fill: "#4e423e", stroke: P.ink, width: 2 })
  // muro largo (de tablas)
  const wall = poly([[200, 858], [453, 846], [453, 1006], [200, 1000]], { fill: linear(200, 0, 453, 0, ["#c8302b", P.red, "#c22b28"]), stroke: P.ink, width: 3 })
  clip(wall, () => {
    for (let x = 206; x < 453; x += 6.5) line(x, 840, x + rand(-0.5, 0.5), 1010, { stroke: alpha(P.redDk, rand(0.35, 0.7)), width: 1 })
    for (let i = 0; i < 8; i++) { const x = rand(205, 450); line(x, 850, x, 1006, { stroke: alpha(P.redLt, 0.4), width: 1.5 }) }
    // sombra del alero
    poly([[200, 858], [453, 846], [453, 866], [200, 874]], { fill: P.redDk, opacity: 0.55, blend: "multiply" })
    // base sucia
    rect(200, 985, 260, 25, { fill: "#5b2a22", opacity: 0.35 })
  })
  // hastial (lado corto, en sombra suave)
  // vértice a media profundidad entre la esquina delantera (453) y la trasera (522)
  const gable = poly([[453, 850], [488, 760], [522, 846], [522, 998], [453, 1006]], { fill: "#b4272a", stroke: P.ink, width: 3, join: "round" })
  clip(gable, () => {
    for (let x = 456; x < 522; x += 6) line(x, 760, x, 1010, { stroke: alpha(P.redDk, 0.6), width: 1 })
    poly([[453, 850], [488, 760], [522, 846], [522, 866], [488, 782], [453, 870]], { fill: P.redDk, opacity: 0.6 })
  })
  // ventanas del muro largo
  const win = (x, y, w, h) => {
    rect(x - 4, y - 4, w + 8, h + 8, { fill: trim, stroke: P.ink, width: 2 })
    rect(x, y, w, h, { fill: linear(x, y, x + w, y + h, ["#5a7279", "#26343a"]) })
    line(x + w / 2, y, x + w / 2, y + h, { stroke: trim, width: 2 })
    for (let k = 1; k < 3; k++) line(x, y + (h * k) / 3, x + w, y + (h * k) / 3, { stroke: trim, width: 1.8 })
    line(x + 2, y + h * 0.7, x + w * 0.4, y + 2, { stroke: alpha("#ffffff", 0.25), width: 2 })
    rect(x - 6, y + h + 3, w + 12, 4, { fill: trim, stroke: P.ink, width: 1.5 })
  }
  win(226, 892, 26, 46)
  win(307, 893, 34, 36)
  win(357, 888, 26, 46)
  // puerta del muro largo
  rect(262, 884, 30, 116, { fill: trim, stroke: P.ink, width: 2 })
  rect(266, 888, 22, 112, { fill: "#6d6156", stroke: P.ink, width: 1.2 })
  rect(268, 892, 18, 18, { fill: "#2f3d42" })
  brush([[268, 960], [286, 990]], { width: 1.5, color: alpha(P.ink, 0.6) })
  // entrada lateral: cuerpo saliente del hastial con tejadillo a dos aguas.
  // La profundidad va en la fuga del muro largo (hacia la izquierda, -12 px).
  const dz = -12
  // lado visible del cuerpo (mira hacia donde mira el muro largo)
  poly([[462 + dz, 884], [462, 884], [462, 1000], [462 + dz, 1003]], { fill: shade(trim, -0.35), stroke: P.ink, width: 1.8 })
  // frente: pilarillos y dintel
  rect(462, 884, 44, 116, { fill: trim, stroke: P.ink, width: 2 })
  rect(462, 884, 6, 116, { fill: shade(trim, 0.15), stroke: alpha(P.ink, 0.6), width: 1 })
  rect(500, 884, 6, 116, { fill: shade(trim, -0.15), stroke: alpha(P.ink, 0.6), width: 1 })
  rect(470, 894, 28, 106, { fill: "#5f564d", stroke: P.ink, width: 1.2 })
  rect(474, 898, 20, 16, { fill: "#2f3d42" })
  for (const y of [926, 958]) rect(474, y, 20, 26, { stroke: alpha(P.ink, 0.5), width: 1 })
  // sombra del tejadillo sobre el frente
  rect(462, 884, 44, 10, { fill: "#3a2a28", opacity: 0.45, blend: "multiply" })
  // tejadillo: faldón izquierdo (en fuga) y frontón con vuelo
  const pk = [484, 852], eL = [455, 888], eR = [513, 888]
  poly([[eL[0] + dz, eL[1]], eL, pk, [pk[0] + dz, pk[1]]], { fill: "#7d8279", stroke: P.ink, width: 2, join: "round" })
  poly([eL, pk, eR, [eR[0] - 4, eR[1]], [pk[0], pk[1] + 7], [eL[0] + 4, eL[1]]], { fill: P.red, stroke: P.ink, width: 1.8, join: "round" })
  poly([[eL[0] + 5, eL[1] - 1], [pk[0], pk[1] + 8], [eR[0] - 5, eR[1] - 1]], { fill: shade(trim, -0.1), stroke: alpha(P.ink, 0.6), width: 1.2 })
  // tejado de chapa: cumbrera paralela al alero delantero (misma fuga)
  const eaveA = [186, 866], eaveB = [446, 858], ridgeB = [490, 750]
  const ridgeA = [ridgeB[0] - (eaveB[0] - eaveA[0]), ridgeB[1] - (eaveB[1] - eaveA[1])]
  const roof = poly([eaveA, ridgeA, ridgeB, eaveB], { fill: "#8f958c", stroke: P.ink, width: 3, join: "round" })
  clip(roof, () => {
    // ondas de la chapa
    for (let i = 0; i <= 34; i++) {
      const t = i / 34
      brush([[lerp(ridgeA[0], ridgeB[0], t), lerp(ridgeA[1], ridgeB[1], t)], [lerp(eaveA[0], eaveB[0], t), lerp(eaveA[1], eaveB[1], t)]], { width: 1.4, color: alpha(P.ink, 0.45), taper: [0, 0] })
    }
    // pintura roja que queda sobre la chapa oxidada
    poly(roughen([[400, 756], [492, 750], [446, 860], [415, 860], [434, 818], [408, 798]], 6, { sharp: true }), { fill: P.red, opacity: 0.92 })
    for (const [x, y, w] of [[245, 838, 30], [295, 798, 22], [345, 822, 18], [215, 855, 14], [360, 782, 12]])
      poly(roughen([[x - w, y + 4], [x, y - 6], [x + w, y - 3], [x + w * 0.4, y + 8]], 3, { sharp: true }), { fill: P.red, opacity: 0.85 })
    for (let i = 0; i < 18; i++) { const x = rand(200, 440), y = rand(775, 858); brush([[x, y], [x - 6, y + rand(6, 16)]], { width: 2, color: alpha("#8b4a2a", 0.6) }) }
    // luz en la parte alta
    poly([ridgeA, ridgeB, [ridgeB[0] - 5, ridgeB[1] + 14], [ridgeA[0] - 5, ridgeA[1] + 14]], { fill: "#ffffff", opacity: 0.15 })
  })
  // cumbrera y tablas de canto del hastial (con vuelo sobre el muro)
  brush([ridgeA, ridgeB], { width: 4, color: P.redDk, taper: [0, 0] })
  for (const w of [6, 1.5]) {
    const c = w > 2 ? P.red : P.ink
    brush([ridgeB, [511, 804], [532, 858]], { width: w, color: c, taper: [0, 0] })
    brush([ridgeB, [468, 804], eaveB], { width: w, color: c, taper: [0, 0] })
  }
  // canalón y bajante
  brush([[eaveA[0], eaveA[1] + 2], [eaveB[0], eaveB[1] + 2]], { width: 4, color: P.redDk, taper: [0, 0] })
  curve([[195, 868], [192, 880], [196, 940]], { stroke: P.red, width: 3 })

  // cimiento de piedra
  poly([[198, 1000], [453, 1006], [520, 998], [522, 1012], [453, 1020], [198, 1012]], { fill: "#7c766c", stroke: P.ink, width: 2, join: "round" })
  // sombra de contacto del zócalo sobre el terreno
  brush([[194, 1016], [453, 1024], [524, 1015]], { width: 9, color: "#231c22", taper: [0.05, 0.05], opacity: 0.55, blur: 3 })
  // porche (plataforma sobre pilotes)
  for (const [x, y1] of [[172, 1072], [210, 1078]]) line(x, 1000, x, y1, { stroke: "#4a3f38", width: 4 })
  line(172, 1010, 210, 1060, { stroke: "#4a3f38", width: 2.5 })
  line(210, 1010, 248, 1060, { stroke: "#4a3f38", width: 2.5 })
  poly([[166, 990], [292, 992], [300, 1003], [168, 1002]], { fill: "#6e8a80", stroke: P.ink, width: 2 })
  rect(166, 1001, 134, 6, { fill: "#5a4c42", stroke: P.ink, width: 1.5 })
  // barandilla roja
  polyline([[168, 991], [168, 955], [262, 955]], { stroke: P.red, width: 3.5, join: "round" })
  line(245, 955, 245, 991, { stroke: P.red, width: 3 })
  // escalera: dos largueros paralelos (38 px) que bajan del borde del porche al suelo
  const run = [60, 108]
  const stairL = [[262, 1002], [262 + run[0], 1002 + run[1]]]
  const stairR = [[300, 1002], [300 + run[0], 1002 + run[1]]]
  // larguero trasero primero (queda detrás de los peldaños)
  brush(stairR, { width: 6, color: "#5a4a3e", taper: [0, 0] })
  for (let i = 1; i <= 8; i++) {
    const t = i / 9
    const a = [lerp(stairL[0][0], stairL[1][0], t), lerp(stairL[0][1], stairL[1][1], t)]
    const b = [a[0] + 38, a[1]]
    poly([a, b, [b[0], b[1] + 5], [a[0], a[1] + 5]], { fill: "#8b7a68", stroke: P.ink, width: 1.4 })
  }
  brush(stairL, { width: 6, color: "#6b5a4c", taper: [0, 0] })
  brush(stairL, { width: 1.3, color: P.ink, taper: [0, 0] })
  // pasamanos sobre el larguero derecho, con poste arriba y abajo
  const railTop = [300, 958], railBot = [356, 1062]
  line(railTop[0], railTop[1], 300, 1002, { stroke: P.red, width: 3 })
  line(railBot[0], railBot[1], railBot[0], 1106, { stroke: P.red, width: 3 })
  polyline([[262, 955], railTop, railBot], { stroke: P.red, width: 3.2, join: "round" })
  line(300, 980, 356, 1084, { stroke: P.red, width: 1.8 })
  // trastos bajo el porche
  ellipse(176, 1085, 10, 8, { fill: "#c9c5ba", stroke: P.ink, width: 1.5 })
  ellipse(268, 1092, 9, 7, { fill: "#d0ccbf", stroke: P.ink, width: 1.5 })
  circle(238, 1090, 7, { fill: "#2e4a7a", stroke: P.ink, width: 1.5 })
  circle(252, 1085, 6, { fill: P.red, stroke: P.ink, width: 1.5 })
  circle(282, 1100, 5, { fill: P.red, stroke: P.ink, width: 1.2 })
})

layer("primer plano", () => {
  // escalones de piedra hacia la puerta del hastial
  for (const [x, y, w] of [[492, 1000, 50], [505, 1018, 58], [520, 1036, 64], [535, 1055, 70]]) {
    poly([[x - w / 2, y], [x + w / 2, y - 3], [x + w / 2 + 4, y + 10], [x - w / 2 - 2, y + 12]], { fill: "#8e887e", stroke: P.ink, width: 1.8, join: "round" })
    line(x - w / 2, y + 1, x + w / 2, y - 2, { stroke: alpha("#ffffff", 0.3), width: 2 })
  }
  // losas sueltas junto a la escalera
  poly([[360, 1092], [440, 1072], [470, 1090], [390, 1115]], { fill: "#9a948a", stroke: P.ink, width: 2, join: "round" })
  poly([[430, 1105], [520, 1080], [545, 1095], [455, 1125]], { fill: "#8e887e", stroke: P.ink, width: 2, join: "round" })
  // rocas que pisan la casa y el faro (no tapan el hueco bajo el porche)
  for (const [x, y, w, h, t] of [[580, 885, 70, 40, 0.3], [645, 900, 70, 44, 0], [700, 925, 60, 40, 0.2], [610, 960, 80, 46, 0.1],
    [560, 1000, 70, 40, 0.4], [670, 990, 90, 50, 0], [620, 1060, 110, 60, 0.2], [700, 1080, 70, 50, 0], [535, 1095, 60, 30, 0.3],
    [235, 1062, 60, 34, 0.2], [140, 1085, 60, 36, 0]]) rock(x, y, w, h, t)
  // primer plano
  for (const [x, y, w, h, t] of [[30, 1200, 110, 80, 0.1], [110, 1235, 80, 60, 0.3], [690, 1200, 110, 80, 0], [620, 1250, 100, 70, 0.2],
    [520, 1150, 60, 40, 0.3], [470, 1265, 70, 40, 0.1], [60, 1120, 70, 40, 0.2]]) rock(x, y, w, h, t)
  // hierba alta delante
  for (let i = 0; i < 60; i++) {
    const x = rand(0, 720), y = rand(1150, 1290)
    if (x > 200 && x < 450) continue
    for (let k = 0; k < 3; k++) brush([[x, y], [x + rand(-10, 10), y - rand(14, 30)]], { width: 2, color: pick([P.grassDk, "#9aa650", "#4b5622"]), taper: [0, 0.9] })
  }
  grain({ opacity: 0.08, size: 1.2 })
})
