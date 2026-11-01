// dibujo.js: faro rojo en la montaña, estilo cómic de línea clara.
canvas({ width: 720, height: 1280, background: linear(0, 0, 0, 900, ["#175f6c", "#2a7c84", "#7fb8b5"]) })

const P = {
  sky: "#1f6d7a",
  cloud: "#f3ecd2",
  cloudSh: "#cfd2bc",
  red: "#d8352c",
  redDk: "#8e1e22",
  redLt: "#ee5a44",
  mtn: "#a9c6cf",
  mtnDk: "#6f93a3",
  snow: "#eef3f1",
  grass: "#7d8a45",
  grassDk: "#56602f",
  rock: "#6f6a66",
  rockLt: "#a29b92",
  path: "#b6ab98",
  ink: "#2a3138",
  wood: "#7a6a5a",
  glass: "#3c5560",
}

// Nube de cúmulo: bolas con contorno de tinta, sombra abajo a la derecha
function cloud(cx, cy, rx, ry, n, rmin, rmax) {
  const puffs = []
  for (let i = 0; i < n; i++) {
    const a = rand(0, Math.PI * 2), d = Math.sqrt(rand())
    puffs.push([cx + Math.cos(a) * rx * d, cy + Math.sin(a) * ry * d, rand(rmin, rmax)])
  }
  puffs.sort((a, b) => a[1] - b[1])
  // silueta de tinta común
  for (const [x, y, r] of puffs) circle(x, y, r + 1.8, { fill: P.ink })
  for (const [x, y, r] of puffs) {
    const t = clamp((y - cy) / ry * 0.5 + 0.5, 0, 1)
    const c = circle(x, y, r, { fill: mix(P.cloud, "#e6e2c8", t * 0.6) })
    clip(c, () => {
      ellipse(x + r * 0.3, y + r * 0.95, r * 1.1, r * 0.7, { fill: P.cloudSh, opacity: 0.35 + t * 0.5 })
      ellipse(x + r * 0.5, y + r * 1.15, r * 0.9, r * 0.5, { fill: mix(P.cloudSh, P.mtnDk, 0.4), opacity: 0.2 + t * 0.5 })
      circle(x - r * 0.35, y - r * 0.4, r * 0.35, { fill: "#fffbe8", opacity: 0.7, blur: r * 0.15 })
      // rizos de sombreado a pluma
      const k = rand(0.3, 0.6)
      brush([[x - r * 0.5, y + r * k], [x, y + r * (k + 0.15)], [x + r * 0.5, y + r * k]], { width: 0.9, color: alpha(P.ink, 0.45) })
    })
    // arco de contorno superior: línea interior del coliflor
    const a0 = rand(170, 200), a1 = rand(340, 375)
    const pts = range(9).map(i => {
      const a = (a0 + (a1 - a0) * i / 8) * Math.PI / 180
      return [x + r * Math.cos(a), y + r * Math.sin(a)]
    })
    brush(pts, { width: 1.8, color: P.ink, taper: [0.35, 0.35] })
  }
}

layer("cielo", () => {
  // pájaros lejanos
  for (const [x, y, s] of [[58, 82, 1], [80, 112, 1.2], [128, 106, 0.9], [190, 452, 0.6], [200, 462, 0.5]]) {
    brush([[x - 10 * s, y - 3 * s], [x - 3 * s, y], [x, y + 2 * s], [x + 4 * s, y - 1 * s], [x + 11 * s, y - 4 * s]], { width: 2.2 * s, color: "#f4f1e6" })
  }
})

layer("nubes", () => {
  seed(3)
  cloud(500, 140, 130, 100, 55, 16, 38)    // torre alta
  cloud(590, 300, 150, 110, 55, 18, 40)
  cloud(310, 380, 120, 90, 50, 16, 36)
  cloud(150, 590, 170, 70, 60, 16, 36)     // banda izquierda
  cloud(640, 600, 110, 200, 60, 18, 40)    // columna derecha baja
  cloud(330, 650, 220, 80, 70, 16, 38)     // banda media
})

layer("montañas", () => {
  const far = mix(P.mtn, "#9fd0cc", 0.4)
  // cordillera lejana a la derecha
  poly([[500, 900], [560, 830], [600, 850], [650, 805], [690, 830], [720, 815], [720, 960], [500, 960]], { fill: far, stroke: alpha(P.ink, 0.6), width: 1.2 })
  // gran pico nevado a la izquierda
  const peak = poly([[0, 780], [50, 735], [108, 682], [140, 712], [175, 735], [230, 778], [290, 800], [360, 830], [420, 845], [520, 860], [520, 1000], [0, 1000]], { fill: P.snow, stroke: P.ink, width: 1.4 })
  clip(peak, () => {
    // cara en sombra a la derecha del pico
    poly([[108, 682], [140, 712], [175, 735], [230, 778], [290, 800], [360, 830], [420, 845], [520, 860], [520, 1000], [170, 1000], [150, 850], [120, 760]], { fill: P.mtn })
    // aristas y canales de nieve
    seed(11)
    for (let i = 0; i < 26; i++) {
      const x0 = rand(20, 500), y0 = 700 + (x0 > 108 ? (x0 - 108) * 0.32 : (108 - x0) * 0.9) + rand(5, 30)
      brush([[x0, y0], [x0 + rand(-25, 10), y0 + rand(40, 80)], [x0 + rand(-45, 0), y0 + rand(100, 160)]], { width: rand(1, 2.2), color: alpha(P.mtnDk, 0.85) })
    }
    brush([[108, 684], [118, 740], [140, 800], [150, 880]], { width: 2, color: P.mtnDk })
  })
  // estribaciones azules más cercanas
  const mid = poly([[0, 880], [40, 850], [90, 870], [150, 840], [210, 875], [270, 900], [330, 950], [330, 1100], [0, 1100]], { fill: mix(P.mtnDk, P.sky, 0.25), stroke: P.ink, width: 1.4 })
  clip(mid, () => {
    seed(12)
    for (let i = 0; i < 18; i++) {
      const x0 = rand(0, 320), y0 = rand(860, 960)
      brush([[x0, y0], [x0 + rand(-20, 20), y0 + rand(40, 90)]], { width: rand(1, 2), color: alpha(P.ink, 0.5) })
    }
    poly([[150, 840], [210, 875], [270, 900], [330, 950], [330, 1100], [170, 1100]], { fill: P.ink, opacity: 0.18 })
  })
  // neblina en la base
  rect(0, 930, 720, 120, { fill: linear(0, 930, 0, 1050, [alpha("#a8d4cf", 0), alpha("#a8d4cf", 0.7)]) })
})

layer("ladera", () => {
  const hill = blob([[-20, 1045], [90, 1022, 0.5], [200, 1012], [330, 1008], [430, 990], [530, 900], [620, 862], [740, 850, 0], [740, 1300, 0], [-20, 1300, 0]], { fill: P.grass, stroke: P.ink, width: 1.6 })
  clip(hill, () => {
    rect(0, 840, 720, 460, { fill: linear(0, 860, 0, 1280, [alpha(P.grassDk, 0), alpha(P.grassDk, 0.5)]) })
    // sendero de tierra
    blob([[270, 1080], [380, 1060], [470, 1050], [560, 1040], [520, 1090], [420, 1130], [420, 1200], [470, 1290, 0], [280, 1290, 0], [300, 1190], [260, 1130]], { fill: P.path, stroke: alpha(P.ink, 0.6), width: 1.2 })
    seed(21)
    for (let i = 0; i < 70; i++) {
      const x = rand(280, 520), y = rand(1060, 1280)
      ellipse(x, y, rand(2, 6), rand(1.5, 3.5), { fill: pick([P.rockLt, "#c9bfae", "#8d857b"]), stroke: alpha(P.ink, 0.5), width: 0.6 })
    }
    // matas de hierba
    for (let i = 0; i < 160; i++) {
      const x = rand(0, 720), y = rand(880, 1280)
      if (x > 290 && x < 500 && y > 1080) continue
      const h = rand(6, 16) * (0.6 + (y - 880) / 600)
      brush([[x, y], [x + rand(-4, 4), y - h]], { width: 1.6, color: pick([P.grassDk, "#9aa65a", "#4b5426"]) })
    }
  })
  // cable del faro hasta el poste y hacia el valle
  line(190, 760, 190, 1000, { stroke: P.wood, width: 3 })
  curve([[392, 452], [300, 610], [190, 762]], { stroke: P.ink, width: 1.4 })
  curve([[190, 762], [100, 880], [0, 1010]], { stroke: P.ink, width: 1.4 })
  curve([[190, 790], [80, 960], [0, 1080]], { stroke: P.ink, width: 1.2 })
})

layer("faro", () => {
  // zócalo de piedra
  const base = rect(392, 850, 236, 30, { fill: "#b9b3a8", stroke: P.ink, width: 1.6, r: 3 })
  clip(base, () => rect(560, 850, 80, 30, { fill: P.mtnDk, opacity: 0.4, blend: "multiply" }))
  // fuste rojo
  const body = poly([[412, 436], [606, 436], [618, 860], [400, 860]], { fill: linear(400, 0, 620, 0, [P.redLt, P.red, [0.6, P.red], [1, P.redDk]]), stroke: P.ink, width: 2 })
  clip(body, () => {
    // sombra propia a la derecha
    poly([[560, 430], [620, 430], [630, 870], [572, 870]], { fill: "#5a1a30", opacity: 0.35, blend: "multiply" })
    // luz a la izquierda
    poly([[412, 430], [438, 430], [430, 870], [400, 870]], { fill: "#ffd0a0", opacity: 0.25, blend: "screen" })
    // juntas de las secciones
    for (const y of [580, 720]) curve([[396, y - 6], [460, y + 4], [560, y + 4], [622, y - 6]], { stroke: alpha(P.ink, 0.7), width: 1.2 })
    // manchas y desconchones
    seed(31)
    for (let i = 0; i < 50; i++) {
      const x = rand(410, 610), y = rand(450, 850)
      ellipse(x, y, rand(1, 3.5), rand(0.8, 2), { fill: pick([P.redDk, "#f07a62"]), opacity: 0.7 })
    }
    for (let i = 0; i < 10; i++) {
      const x = rand(420, 600), y = rand(480, 840)
      brush([[x, y], [x + rand(-2, 2), y + rand(15, 40)]], { width: 1.5, color: alpha(P.redDk, 0.6) })
    }
  })
  // ventanucos
  const win = (x, y, w, h) => {
    rect(x - 2, y - 2, w + 4, h + 4, { fill: "#d8d0bf", stroke: P.ink, width: 1.2 })
    rect(x, y, w, h, { fill: P.glass, stroke: P.ink, width: 1 })
    line(x + w / 2, y, x + w / 2, y + h, { stroke: "#d8d0bf", width: 1.2 })
    line(x, y + h / 2, x + w, y + h / 2, { stroke: "#d8d0bf", width: 1.2 })
    line(x + 2, y + 2, x + w * 0.4, y + 2, { stroke: "#9cc4c8", width: 1 })
  }
  rect(447, 468, 14, 16, { fill: P.glass, stroke: "#d8d0bf", width: 2.5 })
  rect(441, 513, 16, 14, { fill: P.glass, stroke: "#d8d0bf", width: 2.5 })
  rect(434, 624, 12, 18, { fill: P.glass, stroke: "#d8d0bf", width: 2.5 })
  // ventana grande con tejadillo
  poly([[492, 582], [560, 582], [556, 590], [496, 590]], { fill: "#c9c2b4", stroke: P.ink, width: 1.2 })
  win(502, 596, 48, 50)
  rect(496, 650, 60, 6, { fill: "#d8d0bf", stroke: P.ink, width: 1.2 })
  // escalerilla a la derecha
  for (const dx of [588, 600]) line(dx, 450, dx + 6, 855, { stroke: P.redDk, width: 1.6 })
  for (let y = 460; y < 850; y += 14) line(588 + (y - 450) * 0.015, y, 600 + (y - 450) * 0.015, y, { stroke: P.redDk, width: 1.2 })

  // galería: plataforma y ménsulas
  for (const x of [378, 420, 465, 545, 590, 632]) line(x, 444, lerp(x, 505, 0.42), 488, { stroke: P.ink, width: 1.8 })
  ellipse(505, 438, 140, 14, { fill: "#8f8b84", stroke: P.ink, width: 1.8 })
  ellipse(505, 434, 140, 12, { fill: "#c8c2b6", stroke: P.ink, width: 1.4 })
  // parapeto rojo
  const par = rect(414, 384, 182, 50, { fill: linear(414, 0, 596, 0, [P.redLt, P.red, P.redDk]), stroke: P.ink, width: 1.8 })
  clip(par, () => line(414, 410, 596, 410, { stroke: alpha(P.ink, 0.6), width: 1 }))
  // barandilla
  curve([[368, 398], [430, 406], [505, 408], [580, 406], [642, 398]], { stroke: P.red, width: 3 })
  curve([[368, 416], [430, 422], [505, 424], [580, 422], [642, 416]], { stroke: P.redDk, width: 2 })
  for (const x of [368, 395, 614, 642]) line(x, 398, x, 432, { stroke: P.redDk, width: 2.4 })

  // linterna acristalada
  const lan = rect(432, 300, 146, 86, { fill: P.glass, stroke: P.ink, width: 2 })
  clip(lan, () => {
    rect(432, 300, 146, 86, { fill: linear(432, 0, 578, 0, ["#5d7f88", "#2d434c"]) })
    // óptica interior
    ellipse(500, 345, 24, 32, { fill: "#e9e4cf", opacity: 0.85 })
    ellipse(500, 345, 12, 20, { fill: "#fff6c8", glow: { blur: 8, color: "#fff2a0" } })
    for (const x of [462, 492, 522, 552]) line(x, 300, x, 386, { stroke: "#c8c2b6", width: 2.4 })
    line(432, 344, 578, 344, { stroke: "#c8c2b6", width: 1.6 })
    brush([[440, 380], [458, 305]], { width: 4, color: alpha("#bfe0e4", 0.5) })
  })
  rect(428, 372, 154, 14, { fill: P.red, stroke: P.ink, width: 1.4 })
  // cúpula
  const roof = path("M 412 304 Q 424 268 470 260 L 540 260 Q 586 268 598 304 Z", { fill: linear(412, 0, 598, 0, [P.redLt, P.red, P.redDk]), stroke: P.ink, width: 2 })
  clip(roof, () => brush([[440, 296], [458, 274], [480, 264]], { width: 4, color: alpha("#ffb08a", 0.7) }))
  ellipse(505, 304, 96, 7, { fill: P.redDk, stroke: P.ink, width: 1.4 })
  path("M 478 262 Q 482 240 505 238 Q 528 240 532 262 Z", { fill: P.red, stroke: P.ink, width: 1.6 })
  circle(505, 234, 6, { fill: "#8f8b84", stroke: P.ink, width: 1.2 })
  // pararrayos
  line(505, 230, 505, 160, { stroke: P.redDk, width: 2.4 })
  circle(505, 194, 3, { fill: P.redDk })
})

layer("casa", () => {
  // cimientos de piedra
  rect(172, 1000, 292, 18, { fill: "#8a837a", stroke: P.ink, width: 1.4 })
  // pared principal de tablas
  const wall = poly([[176, 858], [462, 858], [462, 1004], [178, 1004]], { fill: P.red, stroke: P.ink, width: 1.8 })
  clip(wall, () => {
    for (let x = 180; x < 462; x += 7) line(x, 858, x, 1004, { stroke: alpha(P.redDk, 0.55), width: 1 })
    rect(170, 858, 300, 22, { fill: "#4a1020", opacity: 0.45, blend: "multiply" })
    rect(170, 960, 300, 50, { fill: linear(0, 960, 0, 1004, [alpha("#4a1020", 0), alpha("#4a1020", 0.4)]), blend: "multiply" })
  })
  // cuerpo del hastial con la puerta
  // faldón derecho del hastial, que se aleja hacia el faro
  const sideRoof = poly([[496, 772], [522, 760], [588, 858], [562, 872]], { fill: shade("#8fa29c", -0.25), stroke: P.ink, width: 1.6 })
  clip(sideRoof, () => {
    for (let i = 1; i < 6; i++) { const t = i / 6; line(lerp(496, 562, t), lerp(772, 872, t), lerp(522, 588, t), lerp(760, 858, t), { stroke: alpha(P.ink, 0.35), width: 1 }) }
  })
  const bay = poly([[456, 856], [496, 790], [550, 856], [550, 996], [456, 996]], { fill: shade(P.red, -0.12), stroke: P.ink, width: 1.8 })
  clip(bay, () => {
    for (let x = 458; x < 552; x += 7) line(x, 790, x, 1000, { stroke: alpha(P.redDk, 0.55), width: 1 })
    rect(520, 790, 40, 210, { fill: "#4a1020", opacity: 0.3, blend: "multiply" })
    // sombra del alero sobre el frontón
    poly([[440, 870], [496, 780], [566, 876], [566, 892], [496, 800], [440, 890]], { fill: "#4a1020", opacity: 0.45, blend: "multiply" })
  })
  // puerta del hastial
  rect(474, 884, 38, 104, { fill: "#d8d0bf", stroke: P.ink, width: 1.4 })
  rect(480, 892, 26, 96, { fill: "#6e675f", stroke: P.ink, width: 1.2 })
  rect(484, 896, 18, 22, { fill: P.glass, stroke: P.ink, width: 1 })
  rect(470, 986, 46, 8, { fill: "#b9b3a8", stroke: P.ink, width: 1.2 })
  // tejado de chapa
  const roof = poly([[158, 868], [246, 776], [492, 772], [446, 868]], { fill: "#8fa29c", stroke: P.ink, width: 1.8 })
  clip(roof, () => {
    for (let i = 0; i <= 34; i++) {
      const t = i / 34
      line(lerp(158, 470, t), lerp(868, 868, t), lerp(246, 482, t), lerp(776, 764, t), { stroke: alpha(P.ink, 0.35), width: 1 })
    }
    // restos de pintura roja
    seed(41)
    for (let i = 0; i < 9; i++) {
      const cx = rand(220, 460), cy = rand(780, 850)
      blob(jitter(ring(cx, cy, rand(14, 34), rand(5, 12), 8, -10), 4), { fill: P.red, opacity: 0.9, stroke: alpha(P.redDk, 0.8), width: 0.8 })
    }
    poly([[158, 868], [246, 776], [270, 776], [190, 868]], { fill: "#ffffff", opacity: 0.12 })
  })
  line(158, 868, 446, 868, { stroke: P.ink, width: 2.4 })
  // aleros del hastial (tablas de remate con algo de grosor)
  poly([[436, 872], [496, 768], [568, 874], [556, 880], [496, 786], [448, 878]], { fill: "#b3ab9e", stroke: P.ink, width: 1.6 })
  line(496, 786, 556, 880, { stroke: shade("#9a9387", -0.3), width: 2 })
  // chimenea
  rect(322, 724, 12, 50, { fill: "#6e675f", stroke: P.ink, width: 1.2 })
  rect(318, 718, 20, 8, { fill: "#4b4640", stroke: P.ink, width: 1.2 })
  // ventanas de la fachada
  const win = (x, y, w, h) => {
    rect(x - 4, y - 4, w + 8, h + 8, { fill: "#e3dccb", stroke: P.ink, width: 1.2 })
    rect(x, y, w, h, { fill: P.glass, stroke: P.ink, width: 1 })
    line(x + w / 2, y, x + w / 2, y + h, { stroke: "#e3dccb", width: 1.6 })
    line(x, y + h / 2, x + w, y + h / 2, { stroke: "#e3dccb", width: 1.6 })
    brush([[x + 3, y + h / 2 - 3], [x + w / 2 - 4, y + 3]], { width: 2, color: alpha("#a9d0d4", 0.6) })
    rect(x - 6, y + h + 3, w + 12, 5, { fill: "#cfc7b6", stroke: P.ink, width: 1 })
  }
  win(218, 900, 38, 48)
  win(330, 902, 34, 40)
  win(396, 900, 36, 46)
  // puerta frontal
  rect(272, 892, 36, 112, { fill: "#d8d0bf", stroke: P.ink, width: 1.4 })
  rect(278, 900, 24, 104, { fill: "#77706a", stroke: P.ink, width: 1.2 })
  rect(282, 904, 16, 20, { fill: P.glass, stroke: P.ink, width: 1 })
  // bajante
  line(180, 870, 180, 1000, { stroke: P.redDk, width: 3 })

  // porche sobre pilotes
  for (const x of [146, 200, 252]) line(x, 1006, x - 3, 1086, { stroke: P.wood, width: 4 })
  line(146, 1030, 250, 1080, { stroke: P.wood, width: 2.5 })
  rect(140, 1000, 124, 8, { fill: "#8f7e6c", stroke: P.ink, width: 1.4 })
  polyline([[142, 1000], [142, 958], [256, 958], [256, 1000]], { stroke: P.red, width: 3, fill: "none" })
  line(142, 975, 256, 975, { stroke: P.redDk, width: 1.6 })
  // escalera de madera
  brush([[258, 1004], [330, 1110]], { width: 6, color: "#6a5a4a", taper: [0, 0] })
  brush([[288, 1004], [358, 1104]], { width: 6, color: "#6a5a4a", taper: [0, 0] })
  for (let i = 0; i < 7; i++) {
    const t = (i + 0.5) / 7
    const x1 = lerp(258, 330, t), y1 = lerp(1004, 1110, t), x2 = lerp(288, 358, t), y2 = lerp(1004, 1104, t)
    poly([[x1, y1], [x2, y2], [x2 + 2, y2 + 5], [x1 + 2, y1 + 5]], { fill: "#a08c74", stroke: P.ink, width: 1 })
  }
  curve([[262, 960], [300, 1020], [335, 1078]], { stroke: P.red, width: 2.5 })
  line(335, 1078, 335, 1105, { stroke: P.red, width: 2.5 })
})

layer("primer plano", () => {
  // roca facetada con luz arriba a la izquierda
  const rock = (cx, cy, rx, ry, s) => {
    const pts = roughen(ring(cx, cy, rx, ry, 9, rand(0, 40)), rx * 0.12, { sharp: 0.5 })
    const r = blob(pts, { fill: P.rock, stroke: P.ink, width: 1.6 })
    clip(r, () => {
      ellipse(cx - rx * 0.3, cy - ry * 0.5, rx * 0.8, ry * 0.6, { fill: P.rockLt, opacity: 0.8 })
      poly([[cx + rx * 0.1, cy - ry * 1.2], [cx + rx * 1.2, cy - ry * 0.4], [cx + rx * 1.2, cy + ry * 1.2], [cx - rx * 0.6, cy + ry * 1.2]], { fill: "#3c3a48", opacity: 0.45, blend: "multiply" })
      brush([[cx - rx * 0.2, cy - ry * 0.7], [cx + rx * 0.05, cy], [cx - rx * 0.1, cy + ry * 0.6]], { width: 1.2, color: alpha(P.ink, 0.7) })
      for (let i = 0; i < 6; i++) circle(cx + rand(-rx, rx) * 0.7, cy + rand(-ry, ry) * 0.7, rand(0.8, 1.8), { fill: P.ink, opacity: 0.5 })
    })
    // hierba al pie
    for (let i = 0; i < 6 * s; i++) {
      const x = cx + rand(-rx, rx), y = cy + ry * rand(0.6, 0.95)
      brush([[x, y], [x + rand(-5, 5), y - rand(8, 18)]], { width: 1.8, color: pick([P.grassDk, "#9aa65a"]) })
    }
  }
  seed(51)
  // escalones de piedra y rocas junto al faro
  for (const [x, y, rx, ry] of [[640, 885, 40, 20], [690, 900, 30, 18], [580, 945, 34, 14], [620, 975, 42, 16], [560, 1010, 38, 16], [520, 1045, 30, 12], [700, 980, 30, 22]]) rock(x, y, rx, ry, 1)
  // rocas cercanas grandes
  for (const [x, y, rx, ry] of [[50, 1180, 70, 45], [150, 1240, 60, 38], [-10, 1260, 50, 40], [640, 1120, 55, 35], [680, 1220, 70, 48], [570, 1260, 55, 30], [230, 1100, 28, 16], [420, 1095, 24, 12]]) rock(x, y, rx, ry, 2)
  // cubos y trastos bajo el porche
  rect(170, 1072, 18, 20, { fill: "#d8d0bf", stroke: P.ink, width: 1.2, r: 2 })
  rect(214, 1078, 20, 16, { fill: "#2f5c7a", stroke: P.ink, width: 1.2, r: 2 })
  circle(250, 1088, 7, { fill: P.red, stroke: P.ink, width: 1.2 })
  // hierbas de primer plano con peso
  for (let i = 0; i < 90; i++) {
    const x = rand(0, 720), y = rand(1130, 1290)
    if (x > 300 && x < 470) continue
    const h = rand(14, 34)
    brush([[x, y], [x + rand(-6, 6), y - h * 0.5], [x + rand(-12, 12), y - h]], { width: rand(1.5, 2.8), color: pick([P.grassDk, "#a3ae62", "#4b5426", P.grass]) })
  }
  // grano de papel
  grain({ opacity: 0.08, size: 1.2, blend: "overlay" })
})
