// dibujo.js: galaxias en colisión (fondo lejano para shooter vertical)
canvas({ width: 1024, height: 1024, background: null })

const P = {
  navy: "#0c1236",
  deep: "#18205a",
  blue: "#3a5aa0",
  violet: "#5e4396",
  teal: "#3f8f9e",
  turq: "#5fb8bb",
  coreY: "#ffe3a6",
  coreP: "#ffa3c8",
  knot: "#f07aa8",
  dust: "#0a0a1e",
  star: "#dce6ff",
}
seed(7)

// Proyección de un disco galáctico: centro, giro e inclinación
function galaxy(c, rot, incl) {
  const cr = Math.cos(rot * Math.PI / 180), sr = Math.sin(rot * Math.PI / 180)
  return ([u, v]) => {
    const y = v * incl
    return [c[0] + u * cr - y * sr, c[1] + u * sr + y * cr]
  }
}
// Brazo espiral logarítmico, con deformación de marea opcional
function arm(g, a0, r0, r1, turns, n, tide) {
  return range(n).map(i => {
    const t = i / (n - 1)
    const th = a0 + t * turns * Math.PI * 2
    const r = r0 * Math.pow(r1 / r0, t)
    let u = r * Math.cos(th), v = r * Math.sin(th)
    if (tide) { u *= 1 + tide * t; }
    return g([u, v])
  })
}
// Banda difusa en varias pasadas (ancha y tenue → estrecha y algo más clara)
function haze(pts, w, cols, k = 1) {
  const env = u => smoothstep(0, 0.25, u) * (1 - smoothstep(0.55, 1, u))
  brush(pts, { width: u => w * 2.2 * (0.25 + env(u) * 0.9), color: cols[0], opacity: 0.15 * k, blur: w * 0.9, taper: [0.2, 0.6] })
  brush(pts, { width: u => w * 1.2 * (0.2 + env(u) * 0.9), color: cols[1], opacity: 0.18 * k, blur: w * 0.45, taper: [0.25, 0.6] })
  brush(pts, { width: u => w * 0.45 * (0.15 + env(u) * 0.9), color: cols[2], opacity: 0.16 * k, blur: w * 0.2, taper: [0.3, 0.6] })
}
// Estrellas y nudos esparcidos a lo largo de un trazado
function sprinkle(pts, spread, n, knots) {
  for (let i = 0; i < n; i++) {
    const a = pts[Math.floor(rand(0, pts.length - 1))]
    const b = pts[Math.min(pts.length - 1, pts.indexOf(a) + 1)]
    const t = rand()
    const x = lerp(a[0], b[0], t) + rand(-spread, spread)
    const y = lerp(a[1], b[1], t) + rand(-spread, spread)
    circle(x, y, rand(0.5, 1.5), { fill: pick([P.star, P.star, P.turq, "#fff2d8"]), opacity: rand(0.2, 0.6) })
  }
  for (let i = 0; i < knots; i++) {
    const a = pts[Math.floor(rand(1, pts.length - 2))]
    const x = a[0] + rand(-spread * 0.5, spread * 0.5), y = a[1] + rand(-spread * 0.5, spread * 0.5)
    circle(x, y, rand(4, 8), { fill: P.knot, opacity: 0.22, blur: 4 })
    circle(x, y, rand(1.2, 2), { fill: "#ffd0e2", opacity: 0.55, blur: 0.6 })
  }
}

const A = galaxy([450, 560], -22, 0.62)   // espiral mediana
const B = galaxy([610, 440], -42, 0.3)    // pequeña y alargada

const tailA = [[400, 640], [300, 680], [215, 760], [185, 850], [240, 925], [360, 962], [500, 970], [620, 950], [720, 915]]
const tailB = [[670, 400], [760, 325], [830, 235], [840, 145], [785, 82], [675, 52], [545, 55], [430, 82], [330, 120]]

layer("halo", () => {
  // Halo azul marino muy tenue que une el conjunto
  ellipse(500, 520, 300, 230, { fill: P.navy, opacity: 0.22, blur: 80, rotate: -30 })
  ellipse(460, 555, 250, 170, { fill: P.deep, opacity: 0.25, blur: 55, rotate: -22 })
  ellipse(610, 440, 190, 60, { fill: P.deep, opacity: 0.25, blur: 35, rotate: -42 })
})

layer("colas de marea", () => {
  // velo ancho y muy tenue para que las colas tengan cuerpo difuso
  for (const t of [tailA, tailB]) {
    brush(t, { width: u => 90 * Math.sin(Math.PI * clamp(u * 1.1, 0, 1)) + 10, color: P.violet, opacity: 0.12, blur: 40, taper: [0.2, 0.6] })
  }
  haze(tailA, 24, [P.violet, P.blue, P.turq], 0.9)
  haze(shift(tailA, 16, -12).slice(1, 7), 12, [P.blue, P.teal, P.turq], 0.6)
  haze(tailB, 21, [P.violet, P.blue, P.turq], 0.9)
  haze(shift(tailB, -14, 12).slice(1, 7), 11, [P.blue, P.teal, P.turq], 0.6)
  // polvo fino en las colas
  brush(shift(tailA, -6, 8).slice(1, 5), { width: 5, color: P.dust, opacity: 0.3, blur: 3 })
  brush(shift(tailB, 6, -6).slice(1, 5), { width: 4, color: P.dust, opacity: 0.3, blur: 3 })
  sprinkle(tailA, 26, 90, 6)
  sprinkle(tailB, 24, 80, 5)
})

layer("galaxia A", () => {
  // disco difuso
  ellipse(450, 560, 240, 150, { fill: P.blue, opacity: 0.18, blur: 50, rotate: -22 })
  ellipse(450, 560, 150, 95, { fill: P.teal, opacity: 0.18, blur: 35, rotate: -22 })
  ellipse(450, 560, 80, 50, { fill: P.violet, opacity: 0.25, blur: 25, rotate: -22 })
  // dos brazos principales, el segundo estirado por la marea
  const arms = [arm(A, 0.3, 35, 250, 0.85, 14, 0.15), arm(A, 0.3 + Math.PI, 35, 240, 0.8, 14, 0.25)]
  for (const a of arms) {
    haze(a, 30, [P.violet, P.blue, P.turq], 1)
  }
  // brazos secundarios tenues
  for (const off of [1.5, 4.6]) haze(arm(A, off, 60, 200, 0.55, 10), 14, [P.blue, P.teal, P.turq], 0.6)
  // franjas de polvo por el borde interior de los brazos
  for (const off of [0.15, 0.15 + Math.PI]) {
    brush(arm(A, off, 40, 215, 0.7, 12), { width: u => 3 + Math.sin(u * Math.PI) * 6, color: P.dust, opacity: 0.4, blur: 3 })
  }
  for (const a of arms) sprinkle(a, 22, 110, 9)
})

layer("galaxia B", () => {
  ellipse(610, 440, 175, 55, { fill: P.violet, opacity: 0.2, blur: 35, rotate: -42 })
  ellipse(610, 440, 120, 34, { fill: P.blue, opacity: 0.22, blur: 22, rotate: -42 })
  ellipse(610, 440, 70, 20, { fill: P.turq, opacity: 0.18, blur: 14, rotate: -42 })
  // brazos aplanados por la inclinación, estirados hacia la cola
  const armsB = [arm(B, 0, 20, 170, 0.6, 12, 0.3), arm(B, Math.PI, 20, 160, 0.6, 12, 0.2)]
  for (const a of armsB) haze(a, 14, [P.violet, P.blue, P.turq], 0.9)
  // franja de polvo que cruza el disco de canto
  brush([B([-150, -10]), B([-60, -22]), B([30, -18]), B([140, -6])], { width: u => 2 + Math.sin(u * Math.PI) * 7, color: P.dust, opacity: 0.35, blur: 3.5 })
  for (const a of armsB) sprinkle(a, 14, 60, 5)
})

layer("nucleos", () => {
  // Núcleo amarillento (A)
  ellipse(450, 560, 70, 48, { fill: "#d9a36a", opacity: 0.3, blur: 30, rotate: -22 })
  ellipse(450, 560, 34, 24, { fill: P.coreY, opacity: 0.55, blur: 14, rotate: -22 })
  circle(450, 560, 9, { fill: "#fff6dc", opacity: 0.9, blur: 5 })
  // Núcleo rosado (B)
  ellipse(610, 440, 50, 20, { fill: "#c06a9a", opacity: 0.3, blur: 20, rotate: -42 })
  ellipse(610, 440, 22, 10, { fill: P.coreP, opacity: 0.6, blur: 8, rotate: -42 })
  circle(610, 440, 6, { fill: "#ffe6f0", opacity: 0.9, blur: 3.5 })
  // zona de choque: nudos de formación estelar entre los dos núcleos
  for (let i = 0; i < 9; i++) {
    const t = rand(0.15, 0.85)
    const x = lerp(450, 610, t) + rand(-40, 40), y = lerp(560, 440, t) + rand(-30, 30)
    circle(x, y, rand(5, 10), { fill: P.knot, opacity: 0.18, blur: 6 })
    circle(x, y, rand(1, 1.8), { fill: "#ffd0e2", opacity: 0.5, blur: 0.6 })
  }
  ellipse(530, 500, 70, 25, { fill: P.violet, opacity: 0.18, blur: 22, rotate: -38 })
  // franja de polvo de A que pasa junto a su núcleo
  brush([[400, 590], [440, 585], [490, 568], [520, 550]], { width: [2, 5, 3], color: P.dust, opacity: 0.35, blur: 2.5 })
})

layer("estrellas", () => {
  // Polvo de estrellas diminutas dentro del conjunto, más denso cerca de los discos
  for (let i = 0; i < 260; i++) {
    const g = rand() < 0.65 ? A : B
    const ang = rand(0, Math.PI * 2), r = Math.pow(rand(), 0.7) * (g === A ? 250 : 170)
    const [x, y] = g([r * Math.cos(ang), r * Math.sin(ang)])
    circle(x, y, rand(0.4, 1.2), { fill: pick([P.star, "#fff0d6", "#cfe0ff"]), opacity: rand(0.15, 0.5) })
  }
})
