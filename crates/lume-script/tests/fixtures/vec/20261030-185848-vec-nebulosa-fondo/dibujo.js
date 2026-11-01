// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  violetaOsc: "#2a1f4e",
  violeta: "#4a3577",
  magenta: "#7a3a6e",
  magentaClaro: "#9a5a8a",
  turquesa: "#2f7a86",
  turquesaClaro: "#5aa5a8",
  rosa: "#d8a8c0",
  rosaPalido: "#f0d0dc",
  polvo: "#0c0e22",
  verde: "#6f8a3a",
  estrella: "#fff2f6",
}
seed(7)

// Eje de la nube: una S diagonal suave
const eje = (t) => [lerp(210, 820, t) + Math.sin(t * Math.PI * 2) * 60, lerp(780, 250, t) + Math.sin(t * Math.PI * 1.3) * 40 - 20]

// Nube de manchas a lo largo del eje
function nubes(n, rMin, rMax, ancho, colores, op, bl) {
  for (let i = 0; i < n; i++) {
    const t = rand(0.02, 0.98)
    const [x, y] = eje(t)
    const k = Math.sin(t * Math.PI) // más ancha en el centro
    const cx = x + rand(-ancho, ancho) * k
    const cy = y + rand(-ancho, ancho) * k
    const r = rand(rMin, rMax) * (0.35 + k * 0.6)
    const pts = jitter(ring(cx, cy, r * rand(0.9, 1.4), r * rand(0.6, 1), 8, rand(0, 180)), r * 0.25)
    blob(pts, { fill: pick(colores), opacity: op * rand(0.6, 1.1), blur: bl * rand(0.8, 1.3) })
  }
}

layer("halo exterior", () => {
  // Velo violeta enorme que se desvanece en transparencia
  blob(jitter(ring(512, 505, 380, 250, 10, -40), 40), { fill: P.violetaOsc, opacity: 0.35, blur: 90 })
  nubes(22, 100, 170, 140, [P.violetaOsc, P.violeta], 0.24, 60)
  // Lóbulos exteriores amplios para ocupar casi todo el lienzo
  const lobulos = [[250, 330, 170, 120], [330, 190, 130, 90], [780, 300, 170, 140], [830, 520, 120, 150], [200, 620, 150, 180], [430, 850, 190, 110], [690, 840, 140, 90], [150, 450, 90, 120]]
  lobulos.forEach(([x, y, rx, ry]) => blob(jitter(ring(x, y, rx, ry, 9, rand(0, 90)), rx * 0.2), { fill: pick([P.violeta, P.violetaOsc, P.magenta]), opacity: 0.2, blur: 70 }))
})

layer("jirones", () => {
  // Filamentos que se deshilachan hacia los bordes
  // Salen del cuerpo hacia fuera, perpendiculares al eje, y se afinan
  for (let i = 0; i < 60; i++) {
    const t = rand(0.05, 0.95)
    const [x, y] = eje(t)
    const lado = rand() < 0.5 ? 1 : -1
    const ang = 52 + lado * 90 + rand(-35, 35) // perpendicular a la diagonal
    const k = Math.sin(t * Math.PI)
    const ini = 90 + k * 110
    const len = rand(110, 230) * (0.6 + k * 0.5)
    const curva = rand(-14, 14)
    const pts = []
    for (let j = 0; j < 5; j++) pts.push(polar(x, y, ini + (len * j) / 4, ang + curva * j))
    brush(pts, { width: [rand(10, 22), rand(5, 10), 1], taper: [0.15, 0.85], color: alpha(pick([P.violeta, P.magenta, P.turquesa, P.violeta]), rand(0.12, 0.22)), blur: rand(5, 10) })
  }
  // Hebras finas que se deshilachan en el borde exterior
  for (let i = 0; i < 70; i++) {
    const t = rand(0.08, 0.92)
    const [x, y] = eje(t)
    const k = Math.sin(t * Math.PI)
    const ang = 52 + (rand() < 0.5 ? 90 : -90) + rand(-40, 40)
    const ini = 160 + k * 120, len = rand(80, 170), c = rand(-10, 10)
    const pts = range(4).map((j) => polar(x, y, ini + (len * j) / 3, ang + c * j))
    brush(pts, { width: [rand(3, 6), 2, 0.5], taper: [0.2, 0.8], color: alpha(pick([P.magentaClaro, P.turquesaClaro, P.violeta]), rand(0.06, 0.12)), blur: rand(3, 5) })
  }
})

layer("capas de gas", () => {
  nubes(22, 90, 160, 120, [P.magenta, P.violeta], 0.3, 40)
  // Contraluz frío turquesa arriba a la izquierda
  group({ move: [-60, -50] }, () => nubes(14, 70, 130, 90, [P.turquesa, P.turquesaClaro], 0.22, 38))
  nubes(14, 60, 110, 80, [P.magentaClaro, P.magenta], 0.28, 30)
  // Volutas ondulantes que siguen el flujo del gas
  for (let i = 0; i < 14; i++) {
    const t0 = rand(0, 0.6), off = rand(-180, 180), fase = rand(0, 6)
    const pts = range(7).map((j) => {
      const t = t0 + j * 0.06
      const [x, y] = eje(t)
      const o = off * Math.sin(t * Math.PI) + Math.sin(j * 1.1 + fase) * 30
      return [x + o * 0.62, y + o * 0.78]
    })
    brush(pts, { width: [4, rand(20, 40), rand(14, 30), 3], color: alpha(pick([P.magentaClaro, P.turquesaClaro, P.magenta]), rand(0.1, 0.18)), blur: rand(8, 16) })
  }
})

layer("tinte verde", () => {
  // Borde inferior derecho: verde enfermizo, la colmena se acerca
  blob(jitter(ring(770, 700, 140, 70, 9, 40), 18), { fill: P.verde, opacity: 0.26, blur: 55 })
  blob(jitter(ring(840, 600, 80, 40, 8, 60), 10), { fill: P.verde, opacity: 0.15, blur: 36 })
  brush([[700, 760], [780, 730], [860, 660], [900, 580]], { width: [4, 18, 12, 2], color: alpha(P.verde, 0.16), blur: 10 })
})

layer("nucleo", () => {
  // Zona interior de formación estelar, rosa pálido y contenida
  blob(jitter(ring(540, 470, 170, 120, 9, -35), 25), { fill: P.magentaClaro, opacity: 0.32, blur: 50 })
  blob(jitter(ring(540, 470, 110, 75, 8, -35), 18), { fill: P.rosa, opacity: 0.3, blur: 40 })
  blob(jitter(ring(548, 462, 55, 38, 7, -30), 10), { fill: P.rosaPalido, opacity: 0.28, blur: 26 })
})

layer("polvo oscuro", () => {
  // Franjas de polvo que atraviesan la nube
  // Irregulares y fragmentadas, siguiendo el flujo diagonal del gas
  const franjas = [
    [[300, 600], [390, 550], [470, 530], [560, 505], [650, 470], [710, 450]],
    [[420, 690], [480, 640], [535, 630], [590, 590], [640, 585]],
    [[480, 360], [545, 395], [600, 380], [670, 330], [740, 320]],
    [[620, 640], [690, 600], [760, 560]],
  ]
  franjas.forEach((f, i) => {
    const g = jitter(f, 4)
    brush(g, { width: [10, 50, 64 - i * 8, 40, 12], taper: [0.3, 0.4], color: alpha(P.polvo, 0.38), blur: 22 })
    brush(shift(g, 0, 4), { width: [4, 18, 26, 14, 3], color: alpha(P.polvo, 0.34), blur: 9 })
    // Pequeñas hebras que se desgajan de la franja
    for (let j = 0; j < 4; j++) {
      const [x, y] = pick(g)
      brush([[x, y], [x + rand(-40, 40), y + rand(10, 40)], [x + rand(-70, 70), y + rand(30, 70)]], { width: [5, 2, 0.5], color: alpha(P.polvo, 0.25), blur: 4 })
    }
  })
})

layer("estrellas jovenes", () => {
  const st = [[520, 455, 3.2], [575, 490, 2.4], [500, 505, 2], [610, 440, 1.8], [470, 430, 1.6], [560, 420, 2.2], [640, 480, 1.4]]
  st.forEach(([x, y, r]) => {
    circle(x, y, r * 6, { fill: P.rosa, opacity: 0.14, blur: r * 4 })
    circle(x, y, r, { fill: P.estrella, opacity: 0.75, blur: 0.6 })
  })
})

layer("textura", () => {
  // Motas de gas tenue en vez de grano (el grano ensuciaría la transparencia)
  for (let i = 0; i < 90; i++) {
    const [x, y] = eje(rand(0.05, 0.95))
    circle(x + rand(-160, 160), y + rand(-160, 160), rand(4, 14), { fill: pick([P.magentaClaro, P.turquesaClaro, P.violeta]), opacity: rand(0.05, 0.12), blur: rand(3, 8) })
  }
})
