// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 720x1280: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 720, height: 1280, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  sun: "#f2ac3a",     // disco mostaza
  sunL: "#f8c862",
  skin: "#e9a27a",    // piel
  skinD: "#c8714f",
  dark: "#24262b",    // pelo, barba, camiseta
  slate: "#4c5552",   // chaqueta
  cream: "#f6ecd8",   // ojos y luces
  rose: "#d97a63",    // mejillas / acento
}

let sunShape, beardShape
layer("disco", () => {
  sunShape = circle(360, 660, 340, { fill: radial(300, 520, 420, [P.sunL, P.sun, shade(P.sun, -0.12)]) })
  clip(sunShape, () => {
    // anillos concéntricos suaves estilo póster
    for (let i = 0; i < 4; i++) circle(360, 660, 300 - i * 60, { fill: "none", stroke: alpha(P.sunL, 0.25), width: 14 })
    grain({ opacity: 0.18, size: 1.4, blend: "overlay" })
  })
})

layer("cuerpo", () => {
  // cuello
  const neck = blob([[300, 660], [410, 660], [418, 800], [360, 830], [292, 800]], { fill: P.skin })
  clip(neck, () => {
    ellipse(355, 700, 80, 70, { fill: shade(P.skin, -0.35), opacity: 0.5, blend: "multiply", blur: 14 })
  })
  const base = path("M 0 700 L 720 700 L 720 975 Q 360 1030 0 975 Z", { hidden: true })
  clip(base, () => {
  // camiseta negra
  poly([[250, 790], [300, 770], [360, 800], [420, 770], [470, 790], [480, 1040], [240, 1040]], { fill: P.dark })
  // cuello redondo de la camiseta
  curve([[296, 776], [330, 812], [360, 818], [392, 812], [424, 776]], { stroke: shade(P.dark, 0.25), width: 9, cap: "round" })
  // chaqueta: dos mitades
  const jacketL = blob([[290, 740, 0], [300, 840], [334, 1040, 0], [70, 1040, 0], [86, 900], [140, 820], [230, 780]], { fill: P.slate })
  const jacketR = blob([[420, 740, 0], [470, 780], [570, 820], [628, 900], [648, 1040, 0], [396, 1040, 0], [430, 860]], { fill: shade(P.slate, -0.12) })
  // sombras y luces de la chaqueta
  clip(jacketL, () => {
    blob([[80, 1000], [100, 880], [160, 860], [200, 1000]], { fill: shade(P.slate, -0.4), opacity: 0.5, blend: "multiply" })
    brush([[150, 830], [220, 800], [270, 790]], { width: 10, color: shade(P.slate, 0.35), opacity: 0.6 })
    grain({ opacity: 0.2, size: 1.2, blend: "overlay" })
  })
  clip(jacketR, () => {
    blob([[540, 820], [640, 900], [640, 1000], [560, 1000]], { fill: shade(P.slate, -0.45), opacity: 0.5, blend: "multiply" })
    grain({ opacity: 0.2, size: 1.2, blend: "overlay" })
  })
  // solapas
  const lapL = blob([[286, 728, 0], [320, 800], [306, 880, 0], [260, 830], [222, 780, 0]], { fill: shade(P.slate, 0.12) })
  const lapR = blob([[424, 728, 0], [492, 776, 0], [452, 832], [412, 880, 0], [404, 800]], { fill: shade(P.slate, 0.02) })
  brush([[222, 780], [262, 830], [306, 880]], { width: 4, color: shade(P.slate, -0.5), taper: [0.2, 0.3] })
  brush([[492, 776], [452, 832], [412, 880]], { width: 4, color: shade(P.slate, -0.5), taper: [0.2, 0.3] })
  // costura central y bolsillo
  brush([[330, 1000], [314, 900], [306, 880]], { width: 5, color: shade(P.slate, -0.5) })
  brush([[400, 1000], [408, 900], [412, 880]], { width: 5, color: shade(P.slate, -0.5) })
  rect(470, 900, 80, 70, { fill: "none", stroke: shade(P.slate, -0.4), width: 3, r: 4 })
  line(470, 918, 550, 918, { stroke: shade(P.slate, -0.4), width: 3 })
  circle(440, 930, 5, { fill: shade(P.slate, -0.5) })
  circle(436, 980, 5, { fill: shade(P.slate, -0.5) })
  })
})

layer("orejas", () => {
  mirror(347, () => {
    const ear = blob([[232, 455], [205, 470], [200, 515], [218, 555], [246, 555]], { fill: P.skin })
    clip(ear, () => {
      brush([[226, 480], [214, 505], [228, 535]], { width: 6, color: P.skinD })
    })
  })
})

let face
layer("cara", () => {
  face = blob([[345, 330], [430, 345], [462, 420], [468, 520], [440, 620], [345, 660], [252, 620], [226, 520], [232, 420], [262, 345]], { fill: P.skin })
  freeform(face, [[290, 400, shade(P.skin, 0.25), 0.4], [345, 500, P.skin, 0.4], [450, 480, P.skinD, 0.3], [345, 640, P.skinD, 0.2]])
  clip(face, () => {
    // mejillas
    ellipse(272, 540, 26, 16, { fill: P.rose, opacity: 0.45, blur: 6 })
    ellipse(425, 540, 24, 15, { fill: P.rose, opacity: 0.45, blur: 6 })
    // ceño fruncido: sombra y pliegues verticales entre las cejas
    ellipse(350, 448, 34, 30, { fill: P.skinD, opacity: 0.45, blend: "multiply", blur: 10 })
    brush([[338, 418], [342, 440], [340, 462]], { width: [2, 4, 2], color: shade(P.skinD, -0.35), opacity: 0.85 })
    brush([[362, 416], [358, 440], [360, 460]], { width: [2, 4, 2], color: shade(P.skinD, -0.35), opacity: 0.85 })
    brush([[300, 400], [325, 410], [338, 420]], { width: 2.5, color: P.skinD, opacity: 0.55 })
    brush([[400, 398], [376, 408], [362, 418]], { width: 2.5, color: P.skinD, opacity: 0.55 })
    // ojeras oscuras
    ellipse(300, 506, 30, 12, { fill: "#6a3a52", opacity: 0.35, blend: "multiply", blur: 6 })
    ellipse(400, 504, 30, 12, { fill: "#6a3a52", opacity: 0.4, blend: "multiply", blur: 6 })
    // barba incipiente gris en las sienes rapadas
    blob([[440, 360], [475, 400], [470, 500], [446, 470]], { fill: shade(P.dark, 0.35), opacity: 0.75 })
    blob([[250, 360], [226, 400], [228, 480], [244, 460]], { fill: shade(P.dark, 0.35), opacity: 0.6 })
    grain({ opacity: 0.14, size: 1.2, blend: "overlay" })
  })
})

layer("pelo", () => {
  seed(3)
  const hair = blob([[236, 425, 0.5], [236, 345], [272, 300], [345, 284], [420, 292], [462, 332], [462, 410, 0.5], [436, 362], [380, 348], [310, 352], [262, 372]], { fill: P.dark })
  // mechones en punta, peinados hacia arriba y algo a la derecha
  for (let i = 0; i < 22; i++) {
    const t = i / 21
    const x = lerp(246, 456, t) + rand(-6, 6)
    const y = 330 - Math.sin(t * Math.PI) * 35
    const len = rand(70, 130) * (0.55 + Math.sin(t * Math.PI) * 0.6)
    const ang = lerp(-60, 35, t) + rand(-14, 14)
    const tip = polar(x, y, len, ang)
    const mid = polar(x, y, len * 0.5, ang - 12)
    brush([[x, y + 20], mid, tip], { width: rand(18, 30), color: P.dark, taper: [0, 0.95] })
  }
  // brillos del pelo
  for (let i = 0; i < 9; i++) {
    const x = rand(270, 430), y = rand(300, 330)
    brush([[x, y + 20], [x + rand(-4, 10), y - rand(25, 45)]], { width: 3, color: "#5b6168", opacity: 0.8 })
  }
})

layer("ojos", () => {
  seed(21)
  // ojos entornados y furiosos; s apunta hacia el centro de la cara
  for (const [x, y, s] of [[300, 488, 1], [400, 486, -1]]) {
    const eyePts = [[x - 28, y + 2, 0], [x - 10, y - 15], [x + 12, y - 15], [x + 28, y + 2, 0], [x + 10, y + 16], [x - 10, y + 16]]
    const eye = blob(eyePts, { fill: "#f1e3c8" })
    clip(eye, () => {
      // venas rojas desde el borde hacia el iris
      for (let i = 0; i < 18; i++) {
        const a = i * 20 + rand(-8, 8), p0 = polar(x, y + 2, 30, a), p1 = polar(x, y + 2, 20, a + rand(-12, 12)), p2 = polar(x, y + 2, rand(11, 14), a + rand(-20, 20))
        brush([p0, p1, p2], { width: rand(1.8, 2.8), color: "#c3242f", opacity: 0.9, taper: [0, 0.9] })
        brush([p1, polar(p1[0], p1[1], rand(5, 8), a + 180 + pick([-50, 50]))], { width: 1, color: "#c3242f", opacity: 0.7, taper: [0, 1] })
      }
      ellipse(x, y + 2, 30, 18, { fill: "none", stroke: "#d0404a", width: 6, opacity: 0.35, blur: 3 })
      // iris rojo con pupila pequeña
      circle(x + s * 3, y + 3, 10.5, { fill: radial(x + s * 3, y + 1, 11, ["#f0503a", "#a3121f", "#5a0a14"]) })
      circle(x + s * 3, y + 3, 3.6, { fill: "#140608" })
      circle(x + s * 3 - 4, y - 1, 2.2, { fill: P.cream })
      // párpado superior caído en diagonal (más bajo hacia el centro)
      poly([[x - s * 40, y - 40], [x + s * 40, y - 40], [x + s * 40, y - 1], [x - s * 40, y - 13]], { fill: mix(P.skin, "#9c5a4a", 0.35) })
      // sombra del párpado sobre el ojo
      brush([[x - s * 34, y - 9], [x + s * 34, y + 4]], { width: 6, color: "#000000", opacity: 0.25, blur: 2 })
    })
    // línea del párpado y contorno inferior
    brush([[x - s * 30, y - 12], [x, y - 7], [x + s * 30, y + 1]], { width: [3, 6, 4], color: shade(P.skinD, -0.75), taper: [0.2, 0.1] })
    brush([[x - 28, y + 3], [x, y + 17], [x + 28, y + 3]], { width: [1, 2.5, 1], color: shade(P.skinD, -0.5) })
    brush([[x - 22, y + 24], [x, y + 28], [x + 22, y + 24]], { width: 2.5, color: P.skinD, opacity: 0.7 })
  }
  // cejas fruncidas: bajan hacia el centro
  brush([[254, 436], [292, 444], [334, 466]], { width: [9, 18, 14], color: P.dark, taper: [0.35, 0.05] })
  brush([[366, 464], [408, 442], [446, 434]], { width: [14, 18, 9], color: P.dark, taper: [0.05, 0.35] })
})

layer("nariz", () => {
  const nose = blob([[350, 468], [372, 512], [382, 546], [354, 566], [322, 560], [314, 538], [336, 512]], { fill: P.skin })
  clip(nose, () => {
    ellipse(374, 535, 16, 32, { fill: P.skinD, opacity: 0.6, blur: 5 })
    ellipse(336, 536, 8, 6, { fill: P.cream, opacity: 0.5, blur: 2 })
  })
  brush([[354, 478], [374, 514], [386, 546], [368, 566]], { width: [2, 5, 4], color: shade(P.skinD, -0.35) })
  brush([[320, 544], [324, 560], [344, 566]], { width: 4, color: shade(P.skinD, -0.35), taper: [0.4, 0.4] })
})

layer("barba", () => {
  seed(9)
  const bPts = [[234, 495, 0.5], [262, 560], [300, 582], [345, 592], [392, 582], [432, 558], [464, 495, 0.5], [462, 590], [440, 662], [398, 724], [352, 756], [304, 724], [264, 664], [240, 590]]
  const beard = blob(bPts, { fill: P.dark })
  beardShape = beard
  // flecos irregulares en la parte baja de la barba
  for (let i = 0; i < 18; i++) {
    const t = i / 17, x = lerp(262, 440, t)
    const y = 660 + Math.sin(t * Math.PI) * 92 - 30
    brush([[x, y], [x + rand(-5, 5), y + rand(18, 32)]], { width: rand(8, 13), color: P.dark, taper: [0, 0.9] })
  }
  clip(beard, () => {
    // canas: mechones claros
    for (let i = 0; i < 70; i++) {
      const x = rand(240, 460), y = rand(600, 740)
      brush([[x, y], [x + rand(-6, 6), y + rand(14, 30)]], { width: rand(1.5, 3.5), color: pick([P.cream, "#9aa0a3", "#6b7174"]), opacity: rand(0.4, 0.85) })
    }
    ellipse(470, 640, 50, 140, { fill: "#000000", opacity: 0.35, blend: "multiply", blur: 20 })
  })
  // boca de vampiro abierta
  group({ move: [0, -7] }, () => {
  const lips = blob([[312, 600, 0], [350, 594], [388, 600, 0], [380, 628], [350, 648], [320, 628]], { fill: shade(P.skinD, -0.1) })
  const mouth = blob([[318, 602, 0], [350, 598], [382, 602, 0], [374, 624], [350, 640], [326, 624]], { fill: "#3a1018" })
  clip(mouth, () => {
    ellipse(352, 640, 22, 12, { fill: "#a8343c" }) // lengua
    rect(310, 596, 80, 9, { fill: P.cream }) // dientes de arriba
    for (let i = 1; i < 6; i++) line(318 + i * 11, 598, 318 + i * 11, 605, { stroke: "#c9b9a0", width: 1.2 })
    ellipse(350, 640, 30, 6, { fill: P.cream, opacity: 0.9 }) // dientes de abajo
    ellipse(380, 620, 14, 30, { fill: "#000000", opacity: 0.35, blend: "multiply", blur: 6 })
  })
  // colmillos
  mirror(350, () => {
    blob([[322, 600], [334, 600], [330, 628, 0]], { fill: P.cream })
    brush([[333, 602], [331, 616], [330, 626]], { width: 2, color: "#c9b9a0", taper: [0, 0.8] })
  })
  // gota de sangre en la punta del colmillo
  blob([[370, 626], [373, 633], [370, 638], [367, 633]], { fill: "#b8202c" })
  circle(369.5, 633, 1, { fill: P.cream })
  // luz del labio inferior y contorno
  brush([[330, 638], [350, 645], [370, 638]], { width: [1, 3, 1], color: shade(P.skin, 0.3), opacity: 0.8 })
  brush([[318, 626], [350, 650], [382, 626]], { width: 2, color: shade(P.skinD, -0.5), opacity: 0.8 })
  })
  // bigote
  blob([[288, 608], [312, 574], [348, 566], [386, 574], [410, 606], [376, 600], [348, 596], [322, 602]], { fill: P.dark })
  for (let i = 0; i < 10; i++) {
    const x = lerp(304, 396, i / 9)
    brush([[x, 578], [x + (x - 350) * 0.12, 593]], { width: 2, color: "#6b7174", opacity: 0.7 })
  }
})

layer("textura", () => {
  clip(beardShape, () => grain({ opacity: 0.25, size: 1.4, blend: "screen" }))
})
