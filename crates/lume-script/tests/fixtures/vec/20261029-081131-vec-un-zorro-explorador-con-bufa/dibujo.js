// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: linear(0, 0, 0, 1024, ["#3a4278", [0.45, "#8a6fa6"], [0.7, "#e8a98e"], "#f3c99a"]) })

const P = {
  noche: "#3a4278",
  lejos: "#7a7fb0",
  medio: "#4f5f8a",
  cerca: "#2f4a5a",
  hierba: "#5b8a5e",
  roca: "#9a96a8",
  musgo: "#86ad52",
  zorro: "#e8803e",
  crema: "#fbecd6",
  oscuro: "#7a3418",
  bufanda: "#d93a3e",
  mochila: "#a8723f",
  luz: "#fff3a6",
}
seed(7)
const FL = [778, 300] // luciérnaga

layer("cielo", () => {
  // estrellas y luna suave
  for (let i = 0; i < 40; i++) {
    const x = rand(20, 1004), y = rand(20, 380)
    circle(x, y, rand(1, 2.6), { fill: "#fff6e0", opacity: rand(0.4, 0.9) })
  }
  circle(190, 170, 110, { fill: "#ffe8c0", opacity: 0.25, blur: 40 })
  circle(190, 170, 52, { fill: "#fff1d8", opacity: 0.95 })
  circle(208, 160, 46, { fill: "#f8dcc0", opacity: 0.5 })
})

layer("bosque lejano", () => {
  // colinas y pinos azulados
  blob([[0, 620], [200, 560], [420, 590], [650, 545], [860, 580], [1024, 560], [1024, 760], [0, 760]], { fill: mix(P.lejos, "#e8a98e", 0.25) })
  for (let i = 0; i < 16; i++) {
    const x = i * 68 + rand(-20, 20), h = rand(130, 220), b = 640 + rand(-20, 20)
    blob([[x, b - h, 0], [x + 34, b - h * 0.45], [x + 46, b, 0], [x - 46, b, 0], [x - 34, b - h * 0.45]], { fill: P.lejos, opacity: 0.9 })
  }
  for (let i = 0; i < 9; i++) {
    const x = i * 130 + rand(-30, 30) - 20, h = rand(240, 330), b = 720 + rand(-15, 15)
    if (x > 300 && x < 760) continue
    blob([[x, b - h, 0], [x + 50, b - h * 0.5], [x + 70, b, 0], [x - 70, b, 0], [x - 50, b - h * 0.5]], { fill: P.medio })
  }
})

layer("suelo", () => {
  blob([[0, 740], [300, 715], [700, 725], [1024, 705], [1024, 1024], [0, 1024]], { fill: linear(0, 720, 0, 1024, [P.hierba, shade(P.hierba, -0.45)]) })
  for (let i = 0; i < 70; i++) {
    const x = rand(0, 1024), y = rand(760, 1010), h = rand(18, 45)
    brush([[x, y], [x + rand(-8, 8), y - h * 0.6], [x + rand(-14, 14), y - h]], { width: rand(3, 6), color: pick([shade(P.hierba, 0.25), shade(P.hierba, -0.3), "#7aa565"]), taper: [0, 1] })
  }
  // arbustos y setas cercanos
  blob([[-20, 900], [40, 820], [130, 830], [180, 900], [140, 1030], [-20, 1030]], { fill: P.cerca })
  blob([[860, 900], [930, 810], [1010, 790], [1050, 1030], [860, 1030]], { fill: P.cerca })
  const seta = (x, y, s) => {
    rect(x - 7 * s, y - 30 * s, 14 * s, 32 * s, { fill: P.crema, r: 6 * s })
    blob([[x - 30 * s, y - 26 * s], [x, y - 52 * s], [x + 30 * s, y - 26 * s]], { fill: "#d65a4a", stroke: "#8a2a2a", width: 3 * s, behind: true })
    circle(x - 10 * s, y - 36 * s, 4 * s, { fill: P.crema }); circle(x + 9 * s, y - 32 * s, 3 * s, { fill: P.crema })
  }
  seta(830, 905, 1.2); seta(875, 925, 0.8); seta(170, 945, 0.9)
})

layer("roca", () => {
  ellipse(495, 885, 300, 34, { fill: "#1e2d3a", opacity: 0.35, blur: 10 })
  const R = blob([[230, 810], [262, 705], [350, 652], [490, 632], [630, 642], [728, 695], [768, 800], [712, 872], [490, 892], [290, 876]], { fill: P.roca })
  clip(R, () => {
    // sombra fría abajo/izquierda, luz cálida de la luciérnaga a la derecha
    ellipse(420, 900, 340, 120, { fill: shade(P.roca, -0.45), blend: "multiply", blur: 20, opacity: 0.8 })
    ellipse(720, 700, 160, 120, { fill: "#ffd98a", blend: "screen", blur: 40, opacity: 0.5 })
    brush([[330, 760], [370, 800], [360, 840]], { width: 5, color: shade(P.roca, -0.35) })
    brush([[600, 790], [640, 820], [700, 830]], { width: 4, color: shade(P.roca, -0.3) })
    // musgo encima
    const M = blob([[220, 740], [270, 680], [360, 640], [500, 618], [640, 628], [745, 690], [760, 735], [700, 712], [640, 730], [560, 705], [470, 725], [380, 708], [300, 740]], { fill: P.musgo })
    clip(M, () => {
      ellipse(330, 720, 140, 50, { fill: shade(P.musgo, -0.4), blend: "multiply", opacity: 0.6, blur: 12 })
      ellipse(650, 660, 120, 40, { fill: shade(P.musgo, 0.5), blend: "screen", opacity: 0.6, blur: 12 })
    })
    for (let i = 0; i < 40; i++) {
      const x = rand(250, 740), y = rand(640, 715)
      circle(x, y, rand(4, 9), { fill: pick([shade(P.musgo, 0.25), shade(P.musgo, -0.2), "#a3c464"]), opacity: 0.8 })
    }
    grain({ opacity: 0.12, size: 1.5 })
  })
  blob([[230, 810], [262, 705], [350, 652], [490, 632], [630, 642], [728, 695], [768, 800], [712, 872], [490, 892], [290, 876]], { fill: "none", stroke: shade(P.roca, -0.55), width: 5 })
  // flecos de musgo colgando
  for (const [x, y] of [[262, 745], [300, 742], [705, 718], [745, 735], [470, 728]])
    brush([[x, y - 6], [x + 2, y + 10], [x - 1, y + rand(18, 30)]], { width: 8, color: shade(P.musgo, -0.15), taper: [0, 0.8] })
})

const ZO = shade(P.zorro, -0.55) // contorno del zorro
layer("cola", () => {
  const T = blob([[420, 640], [330, 668], [240, 650], [175, 590], [168, 515], [200, 470], [238, 488], [250, 545], [290, 590], [360, 600], [430, 600]], { fill: P.zorro, stroke: ZO, width: 9, behind: true })
  clip(T, () => {
    blob([[205, 470], [168, 515], [172, 560], [205, 545], [238, 500], [240, 480]], { fill: P.crema })
    ellipse(320, 660, 140, 40, { fill: shade(P.zorro, -0.4), blend: "multiply", opacity: 0.6, blur: 14 })
    ellipse(190, 520, 30, 90, { fill: "#a9a4e0", blend: "screen", opacity: 0.35, blur: 10 })
    for (let i = 0; i < 6; i++) brush([[260 + i * 25, 615 + rand(-10, 10)], [280 + i * 25, 630]], { width: 3, color: shade(P.zorro, -0.3) })
  })
})

const CU = [[515, 395], [580, 425], [615, 495], [612, 590], [590, 660], [470, 668], [395, 640], [380, 545], [410, 455]]
layer("cuerpo", () => {
  // muslo trasero y cuerpo
  const B = blob(CU, { fill: P.zorro, stroke: ZO, width: 9, behind: true })
  clip(B, () => {
    blob([[560, 420], [612, 480], [612, 580], [580, 600], [555, 520]], { fill: P.crema })
    ellipse(400, 560, 60, 140, { fill: shade(P.zorro, -0.45), blend: "multiply", opacity: 0.6, blur: 20 })
    ellipse(620, 520, 40, 120, { fill: "#ffe08a", blend: "screen", opacity: 0.45, blur: 18 })
  })
  const H = ellipse(462, 600, 80, 66, { fill: P.zorro })
  brush([[400, 555], [430, 545], [475, 548], [520, 575], [540, 620]], { width: [3, 7, 4], color: ZO, taper: [0.3, 0.5] })
  clip(H, () => ellipse(440, 650, 90, 40, { fill: shade(P.zorro, -0.4), blend: "multiply", opacity: 0.6, blur: 12 }))
  // pata trasera
  ellipse(515, 662, 40, 15, { fill: "#5e3628", stroke: "#3a1e18", width: 5 })
  // patas delanteras
  for (const [x, c] of [[548, "#5e3628"], [585, "#4a2a20"]]) {
    rect(x - 17, 540, 34, 120, { fill: c, r: 16, stroke: shade(c, -0.4), width: 5 })
    ellipse(x + 6, 662, 24, 13, { fill: c, stroke: shade(c, -0.4), width: 5 })
  }
  brush([[585, 560], [588, 640]], { width: 4, color: "#7a4a36", opacity: 0.8 })
})
layer("mochila", () => {
  group({ rotate: -12, around: [420, 490] }, () => {
    const M = rect(365, 425, 105, 130, { fill: P.mochila, r: 30, stroke: shade(P.mochila, -0.5), width: 8, behind: true })
    clip(M, () => {
      rect(360, 420, 40, 140, { fill: shade(P.mochila, -0.4), blend: "multiply", opacity: 0.5, blur: 10 })
      rect(380, 500, 75, 45, { fill: shade(P.mochila, -0.15), r: 12 })
    })
    rect(385, 503, 69, 40, { fill: "none", stroke: shade(P.mochila, -0.45), width: 4, r: 12, dash: [8, 6] })
    blob([[362, 470], [365, 430], [400, 415], [445, 418], [474, 440], [474, 470], [418, 480]], { fill: shade(P.mochila, 0.1), stroke: shade(P.mochila, -0.5), width: 7, behind: true })
    rect(410, 462, 22, 28, { fill: "#e6b84a", r: 5, stroke: "#8a6420", width: 3 })
    // rollo de manta encima
    rect(368, 395, 104, 32, { fill: "#5f8fb0", r: 16, stroke: shade("#5f8fb0", -0.5), width: 6, behind: true })
    brush([[400, 398], [400, 425]], { width: 4, color: shade("#5f8fb0", -0.4) })
    brush([[440, 398], [440, 425]], { width: 4, color: shade("#5f8fb0", -0.4) })
  })
  // correa sobre el hombro
  brush([[470, 430], [520, 445], [570, 470], [605, 500]], { width: 14, color: shade(P.mochila, -0.2), taper: [0, 0] })
  brush([[472, 438], [525, 452], [572, 477], [604, 508]], { width: 3, color: shade(P.mochila, -0.55), taper: [0, 0] })
})

layer("cabeza", () => {
  const OR = (b1, tip, b2, c) => {
    blob([b1, tip, b2], { fill: c, stroke: ZO, width: 9, behind: true })
  }
  // oreja lejana
  OR([505, 285], [522, 178, 0], [572, 258], shade(P.zorro, -0.25))
  blob([[518, 270], [526, 200, 0], [556, 258]], { fill: "#4a2a20", opacity: 0.85 })
  // cabeza
  const C = blob([[505, 335], [515, 280], [565, 248], [630, 255], [680, 292], [728, 324], [736, 344], [705, 366], [640, 388], [560, 395], [515, 375]], { fill: P.zorro, stroke: ZO, width: 9, behind: true })
  clip(C, () => {
    blob([[535, 395], [585, 345], [645, 342], [700, 345], [738, 342], [705, 375], [620, 400]], { fill: P.crema })
    ellipse(520, 330, 50, 80, { fill: shade(P.zorro, -0.45), blend: "multiply", opacity: 0.5, blur: 16 })
    ellipse(510, 320, 14, 70, { fill: "#a9a4e0", blend: "screen", opacity: 0.45, blur: 8 })
    ellipse(720, 320, 70, 60, { fill: "#ffe48a", blend: "screen", opacity: 0.6, blur: 20 })
  })
  // mechón de mejilla
  blob([[540, 380], [575, 360], [600, 392], [560, 410], [520, 400, 0]], { fill: P.crema })
  // oreja cercana
  OR([580, 255], [612, 168, 0], [655, 278], P.zorro)
  blob([[594, 258], [612, 192, 0], [638, 268]], { fill: "#4a2a20" })
  blob([[600, 262], [614, 215, 0], [630, 268]], { fill: shade(P.crema, -0.1) })
  // nariz
  ellipse(733, 333, 13, 11, { fill: "#3a2228" })
  circle(737, 329, 3.5, { fill: "#fff", opacity: 0.8 })
  // boca sonriente
  brush([[722, 352], [705, 360], [692, 352]], { width: 3.5, color: ZO, taper: [0.3, 0.3] })
  // ojo mirando a la luciérnaga
  ellipse(648, 300, 12, 15, { fill: "#2e1c22", rotate: 15 })
  circle(653, 293, 4.5, { fill: "#fff" })
  circle(644, 307, 2, { fill: "#fff", opacity: 0.7 })
  brush([[650, 283], [662, 281], [674, 288]], { width: [2, 4, 2], color: ZO })
  // rubor
  ellipse(640, 335, 18, 9, { fill: "#f06a6a", opacity: 0.4, blur: 3 })
})

layer("bufanda", () => {
  const B = shade(P.bufanda, -0.5)
  // puntas ondeando hacia atrás
  const T1 = blob([[520, 400], [470, 405], [420, 425], [380, 418, 0], [392, 445], [430, 452], [480, 440], [525, 425]], { fill: shade(P.bufanda, -0.15), stroke: B, width: 7, behind: true })
  const T2 = blob([[525, 410], [480, 450], [455, 490, 0], [480, 492], [515, 460], [540, 430]], { fill: P.bufanda, stroke: B, width: 7, behind: true })
  // banda del cuello
  const N = blob([[505, 372], [560, 388], [612, 375], [628, 400], [598, 428], [535, 432], [498, 410]], { fill: P.bufanda, stroke: B, width: 8, behind: true })
  clip([N, T1, T2], () => {
    for (let i = 0; i < 6; i++) brush([[410 + i * 40, 360], [400 + i * 40, 500]], { width: 9, color: "#f3c26a", opacity: 0.55, taper: [0, 0] })
    ellipse(600, 380, 40, 25, { fill: "#ffd98a", blend: "screen", opacity: 0.5, blur: 10 })
    ellipse(520, 430, 50, 20, { fill: B, blend: "multiply", opacity: 0.5, blur: 8 })
  })
  // flecos
  for (let i = 0; i < 4; i++) brush([[385 + i * 3, 425 + i * 6], [365 + i * 2, 430 + i * 7]], { width: 4, color: P.bufanda, taper: [0, 0.6] })
  for (let i = 0; i < 3; i++) brush([[458 + i * 8, 488], [450 + i * 9, 506]], { width: 4, color: P.bufanda, taper: [0, 0.6] })
  // nudo
  ellipse(530, 418, 20, 17, { fill: shade(P.bufanda, 0.1), stroke: B, width: 6 })
})

layer("luciérnaga", () => {
  const [x, y] = FL
  circle(x, y + 10, 150, { fill: radial(x, y + 10, 150, [[0, "#fff3a6aa"], [0.4, "#ffd36a44"], [1, "#ffd36a00"]]), blend: "screen" })
  circle(x, y + 12, 40, { fill: "#fff6b0", opacity: 0.5, blur: 14 })
  // estela
  curve([[900, 180], [860, 230], [820, 210], [790, 255], [x + 12, y - 2]], { stroke: "#fff3a6", width: 2.5, dash: [3, 10], cap: "round", opacity: 0.8 })
  // alas
  ellipse(x - 9, y - 12, 9, 14, { fill: "#e8f4ff", opacity: 0.7, rotate: -30, stroke: "#9ab0d0", width: 1.5 })
  ellipse(x + 9, y - 12, 9, 14, { fill: "#e8f4ff", opacity: 0.7, rotate: 30, stroke: "#9ab0d0", width: 1.5 })
  // cuerpo y farolillo
  ellipse(x, y - 4, 8, 10, { fill: "#4a3a52" })
  ellipse(x, y + 12, 10, 12, { fill: "#fff7c0", glow: { blur: 16, color: "#ffe066" } })
  circle(x - 2, y - 10, 2, { fill: "#fff" })
  brush([[x - 3, y - 12], [x - 10, y - 24]], { width: 2, color: "#4a3a52", taper: [0, 0.5] })
  brush([[x + 3, y - 12], [x + 10, y - 24]], { width: 2, color: "#4a3a52", taper: [0, 0.5] })
  // chispas
  for (let i = 0; i < 6; i++) { const [px, py] = polar(x, y + 10, rand(30, 60), rand(0, 360)); circle(px, py, rand(1.5, 3), { fill: "#fff3a6", opacity: 0.8 }) }
})
layer("retoques", () => {
  // luz cálida de la luciérnaga sobre la escena y viñeta
  circle(FL[0] - 60, FL[1] + 80, 260, { fill: "#ffd98a", blend: "soft-light", opacity: 0.5, blur: 60 })
  rect(0, 0, 1024, 1024, { fill: radial(560, 480, 720, [[0.55, "#00000000"], [1, "#1e1a4060"]]) })
  // luciérnagas lejanas
  for (let i = 0; i < 7; i++) {
    const x = rand(60, 980), y = rand(420, 760)
    if (x > 220 && x < 780) continue
    circle(x, y, 10, { fill: "#ffe680", opacity: 0.35, blur: 6 }); circle(x, y, 2.5, { fill: "#fff6c0" })
  }
  grain({ opacity: 0.08, size: 1.2 })
})
