// dibujo.js: fondo base del nivel 1 (enlazable en vertical).
// Lienzo de 720x1280. Todo se dibuja con tile(): copia en y, y-H e y+H para que el borde superior encaje con el inferior.
const W = 720, H = 1280
canvas({ width: W, height: H, background: "#0a0e1c" })

const P = {
  fondo: "#0a0e1c",    // azul marino casi negro
  hondo: "#0d1224",    // franja apenas más clara
  violeta: "#2b2140",  // nebulosa violeta
  violeta2: "#3a2c55", // núcleo violeta
  verde: "#16302c",    // nebulosa verde apagado
  verde2: "#1f3f38",   // núcleo verde
  estrella: "#6f7898", // estrellas tenues
  estrella2: "#a3aac4",// estrellas medias
  chispa: "#c9c3dc",   // destellos (sin llegar al blanco)
}

// Dibuja fn(dy) tres veces para que lo que cruce un borde continúe por el otro.
const tile = (fn) => { for (const dy of [-H, 0, H]) group({ move: [0, dy] }, () => fn()) }

// Eje serpenteante de la nebulosa: periódico en y con periodo H.
const eje = (y) => 360 + 120 * Math.sin((2 * Math.PI * y) / H + 0.6) + 45 * Math.sin((4 * Math.PI * y) / H + 2.1)

layer("cielo", { parallax: 0.1 }, () => {
  // banda plana apenas más clara que sigue el eje (polígono periódico)
  const izq = [], der = []
  for (let y = -40; y <= H + 40; y += 40) { izq.push([eje(y) - 230, y]); der.unshift([eje(y) + 230, y]) }
  tile(() => blob([...izq, ...der], { fill: P.hondo, blur: 30 }))
})

layer("nebulosa", { parallax: 0.2 }, () => {
  seed(7)
  // parámetros precalculados para que las tres copias sean idénticas
  const nubes = []
  for (let y = 0; y < H; y += 70) {
    nubes.push({ c: P.violeta, x: eje(y) + rand(-50, 50), y: y + rand(-20, 20), rx: rand(110, 170), ry: rand(70, 110), rot: rand(-30, 30), o: rand(0.35, 0.55), b: 45 })
  }
  for (let y = 35; y < H; y += 90) {
    // el verde va desplazado al otro lado del eje según la fase
    const lado = Math.sin((2 * Math.PI * y) / H + 2.2)
    nubes.push({ c: P.verde, x: eje(y) + 70 * lado + rand(-30, 30), y: y + rand(-20, 20), rx: rand(80, 130), ry: rand(50, 90), rot: rand(-40, 40), o: rand(0.4, 0.6), b: 40 })
  }
  for (let y = 20; y < H; y += 160) {
    nubes.push({ c: P.violeta2, x: eje(y) + rand(-40, 40), y: y + rand(-30, 30), rx: rand(60, 100), ry: rand(35, 60), rot: rand(-30, 30), o: rand(0.25, 0.4), b: 30 })
  }
  for (let y = 100; y < H; y += 210) {
    const lado = Math.sin((2 * Math.PI * y) / H + 2.2)
    nubes.push({ c: P.verde2, x: eje(y) + 60 * lado + rand(-30, 30), y: y + rand(-30, 30), rx: rand(50, 80), ry: rand(30, 50), rot: rand(-30, 30), o: rand(0.3, 0.45), b: 28 })
  }
  tile(() => {
    for (const n of nubes) ellipse(n.x, n.y, n.rx, n.ry, { fill: n.c, opacity: n.o, blur: n.b, rotate: n.rot })
  })
})

layer("estrellas lejanas", { parallax: 0.3 }, () => {
  // rejilla con jitter fuerte: reparto uniforme sin cúmulos ni filas visibles
  seed(21)
  const lejanas = []
  const paso = 48
  for (let gy = 0; gy < H; gy += paso) for (let gx = 0; gx < W; gx += paso) {
    if (rand() < 0.15) continue
    lejanas.push({ x: gx + rand(0, paso), y: gy + rand(0, paso), r: rand(0.5, 1.1), o: rand(0.35, 0.75) })
  }
  tile(() => { for (const s of lejanas) circle(s.x, s.y, s.r, { fill: P.estrella, opacity: s.o }) })
})

layer("estrellas cercanas", { parallax: 0.45 }, () => {
  seed(5)
  const medias = []
  for (let i = 0; i < 46; i++) medias.push({ x: rand(8, W - 8), y: rand(0, H), r: rand(1.2, 2.1), o: rand(0.5, 0.85), c: rand() < 0.7 ? P.estrella2 : "#b7a9cf" })
  // destellos: unas pocas estrellas con cruz corta y halo leve
  const chispas = [[118, 210], [604, 470], [262, 905], [540, 1150], [70, 1262], [430, 640]].map(([x, y]) => ({ x, y, l: rand(6, 10) }))
  tile(() => {
    for (const s of medias) circle(s.x, s.y, s.r, { fill: s.c, opacity: s.o })
    for (const s of chispas) {
      circle(s.x, s.y, 7, { fill: "#8a86b0", opacity: 0.18, blur: 4 })
      brush([[s.x - s.l, s.y], [s.x + s.l, s.y]], { width: 1.6, taper: [0.5, 0.5], color: P.chispa, opacity: 0.7 })
      brush([[s.x, s.y - s.l], [s.x, s.y + s.l]], { width: 1.6, taper: [0.5, 0.5], color: P.chispa, opacity: 0.7 })
      circle(s.x, s.y, 1.8, { fill: P.chispa })
    }
  })
})
