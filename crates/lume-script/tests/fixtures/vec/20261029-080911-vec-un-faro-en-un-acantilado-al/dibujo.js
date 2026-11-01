// Póster retro: faro en un acantilado al atardecer
canvas({ width: 768, height: 1024, background: null })

const P = {
  cream: "#f6e6c4",
  mustard: "#f2b544",
  orange: "#e8803a",
  coral: "#c9503b",
  teal: "#2f6f73",
  navy: "#1c3247",
}
const H = 600 // horizonte
seed(7)

layer("cielo", () => {
  rect(0, 0, 768, H + 5, { fill: linear(0, 0, 0, H, [[0, P.teal], [0.35, mix(P.teal, P.coral, 0.6)], [0.7, P.orange], [1, P.mustard]]) })
  // sol
  circle(230, 585, 120, { fill: alpha(P.cream, 0.18) })
  circle(230, 585, 78, { fill: P.cream })
  // bandas de nubes planas
  const nubes = [[40, 300, 260, 14], [120, 330, 200, 10], [430, 250, 280, 12], [500, 278, 200, 9], [0, 440, 180, 8], [300, 470, 220, 7]]
  for (const [x, y, w, h] of nubes) rect(x, y, w, h, { r: h / 2, fill: alpha(P.cream, 0.35) })
})

layer("cabos lejanos", () => {
  // el más lejano, casi del color del cielo
  blob([[-20, H + 2], [-20, 545], [40, 535], [110, 548], [150, 572], [170, H + 2]], { fill: mix(P.orange, P.coral, 0.35) })
  blob([[300, H + 2], [330, 560], [380, 548, 0.5], [430, 560], [460, H + 2]], { fill: mix(P.orange, P.coral, 0.55) })
})

layer("mar", () => {
  rect(0, H, 768, 424, { fill: linear(0, H, 0, 1024, [[0, P.orange], [0.35, P.coral], [1, P.navy]]) })
  // reflejo del sol
  for (let i = 0; i < 14; i++) {
    const y = H + 8 + i * 14 + i * i * 0.6
    const w = 150 - i * 7 + rand(-15, 15)
    rect(230 - w / 2 + rand(-10, 10), y, w, 4 + i * 0.3, { r: 3, fill: alpha(P.cream, 0.7 - i * 0.04) })
  }
  // líneas de oleaje
  for (let i = 0; i < 26; i++) {
    const y = rand(H + 20, 960)
    const x = rand(-40, 700)
    const w = rand(30, 90) * (1 + (y - H) / 300)
    rect(x, y, w, 3, { r: 2, fill: alpha(P.mustard, 0.35) })
  }
})

layer("acantilado", () => {
  // cabo intermedio detrás, color medio
  poly([[380, 720], [420, 640], [470, 612], [520, 600], [560, 640], [600, 720]], { fill: mix(P.coral, P.navy, 0.35) })
  // acantilado principal
  const cliff = poly([
    [430, 1024], [445, 860], [470, 800], [462, 740], [490, 690], [500, 610], [520, 560],
    [545, 520], [560, 470], [590, 452], [660, 446], [720, 450], [768, 455], [768, 1024],
  ], { fill: P.navy })
  clip(cliff, () => {
    // estratos de roca en tono teal
    poly([[500, 610], [520, 560], [545, 520], [600, 540], [610, 620], [560, 700], [490, 690]], { fill: mix(P.navy, P.teal, 0.45) })
    poly([[470, 800], [462, 740], [490, 690], [560, 700], [600, 780], [540, 860], [445, 860]], { fill: mix(P.navy, P.teal, 0.3) })
    for (let i = 0; i < 9; i++) {
      const x = 520 + i * 28 + rand(-6, 6)
      line(x, 500 + rand(0, 40), x - rand(20, 50), 900, { stroke: alpha(P.navy, 0.7), width: rand(3, 6) })
    }
    // luz del sol en el borde superior izquierdo
    poly([[520, 560], [545, 520], [560, 470], [590, 452], [600, 464], [572, 482], [556, 530], [530, 570]], { fill: P.coral })
    poly([[590, 452], [660, 446], [720, 450], [768, 455], [768, 466], [660, 458], [598, 464]], { fill: mix(P.coral, P.orange, 0.4) })
  })
  // hierba en la cima
  for (let i = 0; i < 30; i++) {
    const x = rand(565, 768)
    brush([[x, 456], [x + rand(-4, 6), 456 - rand(8, 20)]], { width: 3, color: P.navy })
  }
})

layer("faro", () => {
  const cx = 640, base = 452, top = 250
  // haz de luz
  poly([[cx, 228], [0, 120], [0, 250]], { fill: linear(cx, 230, 0, 180, [alpha(P.cream, 0.55), alpha(P.cream, 0)]), blend: "screen" })
  // casita del farero
  rect(668, 410, 70, 46, { fill: P.cream })
  poly([[660, 412], [703, 385], [746, 412]], { fill: P.coral })
  rect(668, 410, 70, 46, { fill: P.navy, opacity: 0.35, blend: "multiply" })
  rect(690, 425, 12, 16, { fill: P.mustard })
  // torre
  const tower = poly([[cx - 30, base], [cx - 19, top], [cx + 19, top], [cx + 30, base]], { fill: P.cream })
  clip(tower, () => {
    for (let k = 0; k < 3; k++) rect(cx - 40, top + 30 + k * 62, 80, 30, { fill: P.coral })
    // sombra lado derecho (luz desde la izquierda)
    rect(cx + 2, top - 10, 40, 220, { fill: P.navy, opacity: 0.45, blend: "multiply" })
    // luz cálida en el lado del sol
    rect(cx - 40, top - 10, 18, 220, { fill: P.mustard, opacity: 0.35 })
  })
  rect(cx - 6, 400, 12, 22, { fill: P.navy, r: 6 })
  // galería
  rect(cx - 30, top - 8, 60, 10, { fill: P.navy })
  for (let i = 0; i < 6; i++) line(cx - 26 + i * 10.4, top - 8, cx - 26 + i * 10.4, top - 22, { stroke: P.navy, width: 2 })
  rect(cx - 30, top - 24, 60, 3, { fill: P.navy })
  // linterna
  rect(cx - 15, top - 42, 30, 34, { fill: P.mustard, glow: { blur: 18, color: P.cream } })
  rect(cx - 9, top - 38, 18, 26, { fill: P.cream })
  line(cx, top - 42, cx, top - 8, { stroke: P.navy, width: 2 })
  // cúpula
  path(`M ${cx - 20} ${top - 42} Q ${cx} ${top - 72} ${cx + 20} ${top - 42} Z`, { fill: P.navy })
  line(cx, top - 66, cx, top - 80, { stroke: P.navy, width: 3 })
})

layer("rocas y olas", () => {
  // espuma en la base del cabo intermedio
  for (let i = 0; i < 6; i++) rect(385 + i * 34 + rand(-6, 6), 714 + rand(-3, 3), rand(20, 40), 5, { r: 3, fill: alpha(P.cream, 0.6) })

  // rompiente: salpicaduras grandes contra la base del acantilado
  // espuma como racimo de bolas: crema arriba, sombra mostaza abajo
  const splash = (cx, cy, s) => {
    const puffs = []
    for (let i = 0; i < 16; i++) {
      const t = i / 15
      puffs.push([cx + rand(-0.8, 0.8) * s * (1 - t * 0.6), cy - t * s * 1.7, s * (0.62 - t * 0.35) * rand(0.85, 1.15)])
    }
    for (const [x, y, r] of puffs) circle(x, y, r, { fill: mix(P.cream, P.orange, 0.35) })
    for (const [x, y, r] of puffs) circle(x - r * 0.15, y - r * 0.18, r * 0.82, { fill: P.cream })
    for (let i = 0; i < 10; i++) circle(cx + rand(-s * 1.2, s * 1.2), cy - rand(s * 1.4, s * 2.3), rand(2, 5), { fill: P.cream, opacity: rand(0.6, 1) })
  }

  // rocas de fondo en la base
  blob([[400, 905], [430, 860, 0.5], [480, 845], [520, 870], [540, 915]], { fill: mix(P.navy, P.teal, 0.2) })
  splash(490, 880, 62)
  splash(415, 905, 34)

  // ola que rompe en primer plano
  const wave = path("M -20 1030 L -20 935 C 60 900 170 880 240 905 C 290 922 300 950 280 960 C 262 968 250 950 262 942 C 300 960 330 990 340 1030 Z", { fill: P.teal })
  clip(wave, () => {
    rect(-20, 880, 400, 40, { fill: mix(P.teal, P.cream, 0.25) })
    for (let i = 0; i < 6; i++) brush([[rand(-10, 150), 955 + i * 13], [rand(200, 320), 962 + i * 13]], { width: 3, color: alpha(P.cream, 0.3) })
  })
  // cresta de espuma que se riza
  brush([[-10, 932], [80, 902], [180, 890], [245, 906], [285, 935], [272, 955]], { width: [6, 12, 14, 8], color: P.cream, taper: [0.1, 0.3] })
  for (let i = 0; i < 6; i++) circle(rand(200, 300), rand(870, 900), rand(2, 4), { fill: P.cream })

  // rocas oscuras en primer plano
  const r1 = blob([[300, 1030], [320, 960], [360, 930, 0.5], [420, 925], [470, 950], [510, 1030]], { fill: P.navy })
  const r2 = blob([[480, 1030], [520, 945], [580, 905, 0.5], [640, 930], [660, 1030]], { fill: P.navy })
  clip([r1, r2], () => {
    // borde iluminado por el sol (lado izquierdo)
    brush([[318, 975], [335, 945], [362, 930], [410, 926]], { width: 7, color: P.coral, taper: [0.2, 0.6] })
    brush([[515, 960], [540, 930], [578, 906], [610, 912]], { width: 7, color: P.coral, taper: [0.2, 0.6] })
  })
  splash(460, 935, 40)
  splash(335, 960, 28)
})

layer("gaviotas", () => {
  const gull = (x, y, s, flap) => {
    const c = P.navy
    brush([[x - 22 * s, y - flap * s], [x - 10 * s, y - 6 * s], [x, y]], { width: [1, 4 * s, 2 * s], color: c, taper: [0.6, 0] })
    brush([[x, y], [x + 10 * s, y - 6 * s], [x + 22 * s, y - flap * s]], { width: [2 * s, 4 * s, 1], color: c, taper: [0, 0.6] })
    ellipse(x, y + s, 3 * s, 2 * s, { fill: c })
  }
  gull(360, 380, 1.8, 2)
  gull(430, 340, 1.3, 10)
  gull(300, 300, 1.0, -2)
  gull(470, 410, 0.9, 6)
  gull(150, 470, 0.7, 4)
  gull(110, 820, 2.6, 6)
})

layer("grano", () => {
  grain({ opacity: 0.14, size: 1.4, blend: "overlay" })
})
