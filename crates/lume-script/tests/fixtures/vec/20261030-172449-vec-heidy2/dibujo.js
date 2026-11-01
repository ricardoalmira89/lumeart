// heidy2: retrato semirrealista con auriculares
canvas({ width: 1024, height: 1024, background: null })
setLight(-0.6, -1)

const P = {
  skin: "#e9a898", skinL: "#f8cfbf", skinD: "#c07a70", skinDD: "#9c5a58",
  hair: "#c8a67f", hairD: "#7d6047", hairDD: "#4e3a2c", hairL: "#efe0c6",
  top: "#4a3530", topD: "#2e211f", cardi: "#1e1c22", cardiL: "#3a3742",
  set: "#24232a", setL: "#55535e", gold: "#d2ae68", goldD: "#8f6f35",
  lip: "#c66d76", eye: "#6d8494", line: "#5a3430",
}

// Pelo trasero: la gran masa detrás de la cara y los hombros
layer("pelo-detras", () => {
  const back = blob([
    [470, 28], [610, 40], [720, 110], [775, 250], [800, 420], [850, 560],
    [920, 700], [975, 860], [1000, 1010], [800, 1015], [700, 900], [640, 760],
    [360, 760], [250, 760], [190, 700], [205, 520], [225, 330], [270, 150], [350, 60],
  ], { fill: linear(250, 0, 950, 0, [P.hair, P.hairD, P.hair, P.hairL]) })
  clip(back, () => {
    ellipse(480, 90, 160, 60, { fill: P.hairL, opacity: 0.5, blur: 30 })
    ellipse(700, 520, 70, 220, { fill: P.hairDD, opacity: 0.6, blur: 40 })
    ellipse(860, 760, 70, 200, { fill: P.hairL, opacity: 0.6, blur: 40 })
    ellipse(250, 620, 50, 150, { fill: P.hairD, opacity: 0.5, blur: 30 })
    // raíces más oscuras en la coronilla
    ellipse(520, 120, 260, 110, { fill: "#8a6a4c", opacity: 0.75, blur: 40 })
  })
})

// Cuerpo: camiseta de cuello alto marrón y chaqueta negra
layer("cuerpo", () => {
  // chaqueta
  const ch = blob([
    [20, 735, 0.5], [180, 690], [330, 700], [500, 760], [700, 740], [870, 760],
    [960, 830], [1010, 950], [1015, 1015, 0], [10, 1015, 0], [5, 760, 0.5],
  ], { fill: P.cardi })
  clip(ch, () => {
    ellipse(160, 760, 160, 40, { fill: P.cardiL, opacity: 0.5, blur: 25 })
    ellipse(900, 820, 80, 40, { fill: P.cardiL, opacity: 0.4, blur: 25 })
  })
  // camiseta
  const t = blob([
    [330, 700], [430, 745], [520, 760], [590, 735], [680, 760], [760, 900],
    [780, 1015, 0], [215, 1015, 0], [235, 860], [280, 740],
  ], { fill: linear(300, 760, 720, 1000, ["#5a4038", P.top, P.topD]) })
  clip(t, () => {
    ellipse(240, 950, 60, 140, { fill: P.topD, opacity: 0.7, blur: 30 })
    ellipse(720, 900, 70, 160, { fill: P.topD, opacity: 0.7, blur: 30 })
    brush([[300, 900], [330, 960], [340, 1010]], { width: 6, color: alpha(P.topD, 0.6) })
    brush([[640, 880], [660, 950], [700, 1010]], { width: 8, color: alpha(P.topD, 0.6) })
  })
  // cuello alto
  const col = blob([
    [345, 690], [420, 720], [520, 735], [590, 715], [600, 760], [520, 785],
    [420, 775], [340, 740],
  ], { fill: linear(350, 720, 600, 760, ["#634840", P.top, P.topD]) })
  brush([[350, 735], [430, 770], [520, 780], [595, 755]], { width: [2, 5, 4, 2], color: P.topD })
})

// Cuello
layer("cuello", () => {
  const n = blob([
    [355, 600], [580, 610], [585, 700], [590, 745], [520, 760], [420, 750], [345, 715], [350, 650],
  ], { fill: linear(350, 0, 590, 0, [P.skinD, P.skin, P.skinD]) })
  clip(n, () => {
    // sombra que proyecta la mandíbula
    blob([[340, 600], [420, 680], [480, 712], [560, 680], [600, 610], [600, 600]], { fill: P.skinDD, opacity: 0.55, blur: 14, blend: "multiply" })
  })
})

// Cara
layer("cara", () => {
  const face = blob([
    [500, 170], [600, 185], [660, 250], [675, 360], [668, 450], [652, 540],
    [610, 620], [545, 680], [470, 705], [400, 675], [345, 610], [318, 520],
    [305, 420], [312, 320], [350, 230], [420, 180],
  ], { fill: radial(450, 380, 330, [P.skinL, P.skin, P.skinD], { fx: 440, fy: 330 }) })
  clip(face, () => {
    blob([[640, 250], [690, 350], [670, 560], [600, 650], [560, 690], [610, 560], [640, 420]], { fill: P.skinD, opacity: 0.6, blur: 25, blend: "multiply" })
    // sombra bajo el pelo en la frente y a los lados
    ellipse(500, 175, 210, 45, { fill: P.skinDD, opacity: 0.35, blur: 20, blend: "multiply" })
    ellipse(668, 400, 30, 180, { fill: P.skinDD, opacity: 0.35, blur: 18, blend: "multiply" })
    ellipse(310, 420, 22, 160, { fill: P.skinDD, opacity: 0.3, blur: 16, blend: "multiply" })
    // rubor
    ellipse(385, 500, 55, 35, { fill: "#e07f80", opacity: 0.3, blur: 20 })
    ellipse(600, 505, 50, 32, { fill: "#e07f80", opacity: 0.25, blur: 20 })
    // luz en frente y pómulo
    ellipse(470, 260, 80, 40, { fill: P.skinL, opacity: 0.5, blur: 25, blend: "screen" })
    ellipse(410, 455, 40, 18, { fill: P.skinL, opacity: 0.4, blur: 12, blend: "screen" })
  })
  ink(face, { width: 2.5, weight: 2, color: P.line, breaks: true })
})

// Rasgos: ojos, cejas, nariz, boca
layer("rasgos", () => {
  // cuencas de los ojos (sombra suave)
  ellipse(400, 380, 60, 30, { fill: P.skinD, opacity: 0.45, blur: 14, blend: "multiply" })
  ellipse(588, 388, 58, 30, { fill: P.skinD, opacity: 0.5, blur: 14, blend: "multiply" })

  function eye(cx, cy, w, tilt) {
    const top = [[cx - w, cy + 3], [cx - w * 0.3, cy - 13], [cx + w * 0.4, cy - 12], [cx + w, cy + 2]]
    const bot = [[cx + w, cy + 2], [cx + w * 0.3, cy + 9], [cx - w * 0.4, cy + 9]]
    const white = blob([[cx - w, cy + 3, 0], ...top.slice(1, 3), [cx + w, cy + 2, 0], ...bot.slice(1)], { fill: "#efe2dc", rotate: tilt })
    clip(white, () => {
      circle(cx + 2, cy - 1, 13, { fill: P.eye })
      circle(cx + 2, cy - 1, 13, { fill: "none", stroke: shade(P.eye, -0.5), width: 2 })
      circle(cx + 2, cy - 1, 5.5, { fill: "#1d1e26" })
      // párpado caído: mira hacia abajo
      ellipse(cx, cy - 14, w + 4, 10, { fill: mix(P.skin, P.skinD, 0.5) })
      ellipse(cx, cy - 4, w, 5, { fill: "#6a4a48", opacity: 0.5, blur: 3 })
      circle(cx - 3, cy - 5, 3, { fill: "#ffffff", opacity: 0.9 })
    })
    // párpado superior y pestañas
    brush(shift(top, 0, 4).map(([x, y], i) => [x, y + (i === 1 || i === 2 ? 2 : 0)]), { width: [2, 5, 5, 3], taper: [0.1, 0.25], color: "#3a2524" })
    brush([[cx + w * 0.6, cy - 6], [cx + w + 6, cy - 4]], { width: 2.5, color: "#3a2524" })
    // pliegue del párpado
    brush([[cx - w * 0.8, cy - 12], [cx, cy - 24], [cx + w * 0.8, cy - 16]], { width: 2.5, color: alpha(P.skinDD, 0.6) })
    // párpado inferior
    brush(bot, { width: [1.5, 2, 1], color: alpha(P.skinDD, 0.7) })
  }
  eye(402, 384, 38, -2)
  eye(588, 393, 36, 2)

  // cejas
  brush([[345, 345], [375, 330], [420, 325], [455, 333]], { width: [4, 10, 8, 3], taper: [0.2, 0.5], color: "#8a6650" })
  brush([[532, 343], [570, 333], [612, 335], [645, 350]], { width: [3, 9, 8, 3], taper: [0.2, 0.5], color: "#8a6650" })

  // nariz
  brush([[455, 400], [450, 440], [452, 480]], { width: [1, 4, 2], color: alpha(P.skinD, 0.7) })
  ellipse(498, 470, 22, 50, { fill: P.skinD, opacity: 0.35, blur: 12, blend: "multiply" })
  ellipse(478, 488, 18, 14, { fill: P.skinL, opacity: 0.6, blur: 6 })
  brush([[443, 497], [452, 510], [470, 512]], { width: [2, 4, 2], color: P.skinDD })
  brush([[497, 512], [514, 508], [520, 495]], { width: [2, 4, 2], color: P.skinDD })
  ellipse(481, 514, 12, 4, { fill: P.skinDD, opacity: 0.6, blur: 2 })

  // boca: sonrisa cerrada
  const lipU = blob([[402, 546, 0], [440, 535], [470, 538], [482, 541], [500, 537], [530, 538], [560, 548, 0], [520, 553], [480, 555], [440, 553]], { fill: shade(P.lip, -0.1) })
  const lipL = blob([[410, 550, 0], [480, 556], [552, 551, 0], [530, 568], [480, 576], [435, 568]], { fill: P.lip })
  clip(lipL, () => ellipse(470, 562, 30, 7, { fill: "#f2b0b0", opacity: 0.6, blur: 4 }))
  brush([[400, 545], [440, 553], [480, 556], [525, 553], [562, 546]], { width: [2, 4, 4, 4, 2], color: "#7e3c43" })
  ellipse(480, 590, 30, 8, { fill: P.skinD, opacity: 0.4, blur: 6 })
  // hoyuelos de la sonrisa
  brush([[395, 530], [390, 548], [398, 562]], { width: 3, color: alpha(P.skinD, 0.6) })
  brush([[568, 532], [572, 548], [565, 560]], { width: 3, color: alpha(P.skinD, 0.6) })
})

// Pelo delantero: mechones sobre la frente y los hombros
layer("pelo-delante", () => {
  seed(7)
  // mechón izquierdo: de la raya hacia la sien
  const L = blob([
    [520, 130], [470, 160], [400, 195], [345, 260], [318, 340], [308, 450], [318, 560],
    [300, 520], [272, 420], [268, 300], [300, 190], [380, 110], [470, 80],
  ], { fill: P.hair })
  clip(L, () => {
    ellipse(470, 110, 90, 40, { fill: P.hairD, opacity: 0.7, blur: 20 })
    ellipse(360, 230, 60, 50, { fill: P.hairL, opacity: 0.6, blur: 20 })
    ellipse(290, 420, 25, 120, { fill: P.hairD, opacity: 0.5, blur: 15 })
  })
  // mechón derecho: cae por la sien hasta la oreja
  const R = blob([
    [520, 130], [600, 110], [690, 160], [745, 260], [760, 380], [700, 440], [690, 360],
    [672, 270], [630, 205], [570, 165],
  ], { fill: P.hairD })
  clip(R, () => {
    ellipse(640, 170, 60, 35, { fill: P.hair, opacity: 0.8, blur: 18 })
    ellipse(700, 340, 30, 80, { fill: P.hairDD, opacity: 0.5, blur: 15 })
  })
  // mechones finos de la parte superior
  for (let i = 0; i < 26; i++) {
    const t = i / 25
    const x0 = 515 + rand(-10, 10)
    const side = t < 0.5 ? -1 : 1
    const k = side < 0 ? t * 2 : (t - 0.5) * 2
    const pts = side < 0
      ? [[x0, 70 + rand(0, 40)], [lerp(470, 340, k), lerp(150, 200, k)], [lerp(400, 290, k) + rand(-8, 8), lerp(210, 420, k)]]
      : [[x0, 70 + rand(0, 40)], [lerp(560, 700, k), lerp(140, 200, k)], [lerp(610, 740, k) + rand(-8, 8), lerp(200, 380, k)]]
    brush(pts, { width: rand(2, 5), color: pick([P.hairL, P.hairD, "#e2c9a2", P.hairDD]), taper: [0.2, 0.5] })
  }
  // cascada de pelo sobre el hombro derecho
  const C = blob([
    [660, 520], [720, 600], [800, 700], [880, 800], [930, 920], [950, 1015, 0], [640, 1015, 0],
    [590, 900], [600, 780], [640, 680],
  ], { fill: P.hair })
  clip(C, () => {
    ellipse(800, 800, 80, 220, { fill: P.hairL, opacity: 0.7, blur: 35 })
    ellipse(640, 760, 30, 150, { fill: P.hairD, opacity: 0.6, blur: 20 })
  })
  clip(C, () => {
    for (let i = 0; i < 45; i++) {
      const x = rand(600, 880), y = rand(520, 620)
      const dx = (x - 600) * 0.35 + rand(-10, 10)
      const pts = [[x, y], [x + dx * 0.4 + rand(-12, 12), y + 160], [x + dx * 0.7 + rand(-15, 15), y + 300], [x + dx, y + 450]]
      brush(pts, { width: rand(1.5, 3.5), color: pick([P.hairL, P.hairD, "#efe4cf", P.hairDD]), taper: [0.4, 0.4] })
    }
  })
  // pelo sobre el hombro izquierdo
  const S = blob([[200, 600], [250, 640], [300, 700], [270, 740], [215, 745], [185, 700]], { fill: P.hair })
  for (let i = 0; i < 10; i++) {
    const x = rand(200, 260)
    brush([[x, 560], [x - 10, 650], [x + rand(0, 30), 740]], { width: rand(2, 4), color: pick([P.hairL, P.hairD]), taper: [0.3, 0.6] })
  }
})

// Auriculares con micrófono
layer("auriculares", () => {
  // diadema transparente por arriba
  curve([[300, 130], [370, 85], [460, 58], [560, 50], [640, 62]], { stroke: alpha("#c9c9d4", 0.55), width: 7 })
  // diadema negra del lado izquierdo
  brush([[238, 360], [230, 280], [250, 200], [300, 132]], { width: [16, 15, 13, 10], taper: [0, 0.2], color: P.set })
  brush([[234, 330], [232, 260], [255, 195]], { width: 3, color: P.setL, taper: [0.3, 0.3] })
  // lado derecho
  brush([[760, 400], [770, 330], [762, 260], [740, 200]], { width: [14, 14, 10, 4], taper: [0, 0.4], color: P.set })
  // auricular izquierdo
  const cl = blob([[215, 345], [270, 330], [300, 360], [305, 450], [290, 495], [240, 495], [212, 470]], { fill: P.set })
  clip(cl, () => {
    ellipse(240, 380, 25, 40, { fill: P.setL, opacity: 0.6, blur: 10 })
    rect(260, 330, 50, 170, { fill: "#141318", opacity: 0.8 })
  })
  ink(cl, { width: 2, weight: 2, color: "#0d0c10" })
  rect(225, 315, 25, 40, { fill: P.set, r: 6 })
  // micrófono
  brush([[232, 470], [228, 530], [250, 580], [290, 610]], { width: 12, taper: [0, 0.1], color: P.set })
  brush([[236, 490], [236, 540], [255, 578]], { width: 3, color: P.setL })
  ellipse(305, 612, 18, 14, { fill: "#2c2b33", rotate: 30 })
  circle(298, 606, 5, { fill: P.setL, opacity: 0.6 })
  // auricular derecho
  // reflejo del izquierdo (x' = 975 - x), algo más bajo
  const cr = blob([[760, 365], [705, 350], [675, 380], [670, 470], [685, 515], [735, 515], [763, 490]], { fill: P.set })
  clip(cr, () => {
    ellipse(735, 400, 22, 40, { fill: P.setL, opacity: 0.45, blur: 10 })
    rect(665, 350, 50, 170, { fill: "#141318", opacity: 0.8 })
  })
  ink(cr, { width: 2, weight: 2, color: "#0d0c10" })
  rect(728, 340, 25, 40, { fill: P.set, r: 6 })
})

// Collar dorado
layer("collar", () => {
  const ch = [[324, 710], [338, 790], [372, 865], [420, 925], [468, 975]]
  const ch2 = [[575, 830], [560, 890], [520, 945], [478, 978]]
  curve(ch, { stroke: P.goldD, width: 6 })
  curve(ch, { stroke: P.gold, width: 3.5 })
  curve(ch2, { stroke: P.goldD, width: 6 })
  curve(ch2, { stroke: P.gold, width: 3.5 })
  curve(shift(ch, -1, -1), { stroke: "#fff1c4", width: 1, opacity: 0.8 })
  poly([[474, 978], [490, 1000], [474, 1015], [458, 1000]], { fill: P.gold, stroke: P.goldD, width: 2 })
  circle(470, 995, 3, { fill: "#fff6d8" })
})

// Retoques y grano
layer("acabado", () => {
  // grano solo sobre la figura
  const sil = blob([
    [470, 28], [610, 40], [720, 110], [775, 250], [800, 420], [850, 560], [920, 700],
    [955, 820], [1005, 950], [1012, 1015, 0], [12, 1015, 0], [5, 760, 0.5], [190, 690], [205, 520],
    [225, 330], [270, 150], [350, 60],
  ], { hidden: true })
  clip(sil, () => grain({ opacity: 0.06, size: 1, blend: "overlay" }))
})
