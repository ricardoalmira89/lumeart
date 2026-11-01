// Hoja 1 · Shooter espacial: nave del jugador, primeros enemigos y proyectiles.
// Estilo pegatina: tinta oscura de grosor variable, colores saturados, brillos blancos y borde blanco.
// Escala: 10 unidades = 1 píxel del juego (pantalla lógica de 256x240). Cada sprite vive en su caja.
// Con EXPORTAR = true solo se dibujan los sprites, sobre fondo transparente, para recortarlos en PNG.
const EXPORTAR = false
canvas({ width: 1600, height: 900, background: EXPORTAR ? null : "#15122b" })
setLight(-1, -1)

const P = {
  tinta: "#1a1433", borde: "#ffffff",
  // nave del jugador
  casco: "#eef1f8", acento: "#22b8d4", cabina: "#62ecff", motor: "#ff8a2a", llama: "#ffd447", nucleo: "#fff3a8",
  // enemigos
  zumbon: "#9be22f", zumbonOsc: "#3f7d1f", ala: "#bff4ff", visor: "#ff3b5c",
  vigia: "#7a4fd8", ojo: "#fff3cf", iris: "#ffc21f",
  medusa: "#ff6db3", medusaLuz: "#ffd3ea", tentaculo: "#ff9bd0",
  aguja: "#ff5534", agujaPunta: "#ffd166",
  cristal: "#59e3ff", cristal2: "#b27dff", roca: "#4a3b6b",
  // proyectiles y recogibles
  balaEnemiga: "#ff3df2", balaJugador: "#2ad4ff", energia: "#ffd23f",
  // presentación de la hoja
  carta: "#221d42", cartaBorde: "#342c63", texto: "#bfb6e8",
}
const BORDE = 7   // grosor del borde blanco de pegatina
const TINTA = 5   // grosor base de la tinta

// Cajas de los sprites: [x, y, ancho, alto] en unidades (÷10 = píxeles del juego).
const CAJA = {
  nave_1: [60, 70, 160, 200], nave_2: [280, 70, 160, 200],
  zumbon_1: [520, 70, 160, 160], zumbon_2: [740, 70, 160, 160],
  aguja: [960, 70, 160, 160], medusa: [1180, 70, 160, 160], geoda: [1420, 90, 120, 120],
  vigia_1: [60, 350, 160, 160], vigia_2: [280, 350, 160, 160],
  bala_jugador: [530, 370, 40, 120], bala_enemiga: [650, 400, 60, 60],
  esquirla: [780, 400, 60, 60], energia: [920, 390, 80, 80],
}

// ---------- utilidades ----------

// Un sprite: su tarjeta y su nombre en la hoja, y el dibujo en coordenadas locales de su caja.
function sprite(nombre, dibujar) {
  const [x, y, w, h] = CAJA[nombre]
  if (!EXPORTAR) {
    rect(x - 20, y - 20, w + 40, h + 40, { fill: P.carta, stroke: P.cartaBorde, width: 2, r: 18 })
    text(x + w / 2, y + h + 46, nombre.replace("_", " · "), { size: 18, fill: P.texto, anchor: "middle", font: "monospace" })
  }
  layer(nombre, () => group({ move: [x, y] }, () => dibujar(w, h)))
}

// Refleja puntos respecto a la vertical x = eje (conserva el tercer valor: esquina o suavidad).
function espejo(pts, eje) {
  return pts.map((q) => (q.length > 2 ? [2 * eje - q[0], q[1], q[2]] : [2 * eje - q[0], q[1]]))
}

// Borde de pegatina: las siluetas en blanco, engordadas BORDE unidades, detrás del sprite.
function pegatina(siluetas) {
  siluetas.forEach((pts) => blob(pts, { fill: P.borde, stroke: P.borde, width: BORDE * 2 }))
}

// Una pieza: color plano, sombra de un tono (cel), detalles dentro y tinta de grosor variable.
function pieza(pts, color, o) {
  o = o || {}
  const f = blob(pts, { fill: color })
  cel(f, { depth: o.sombra === undefined ? 0.16 : o.sombra, color: shade(color, o.osc === undefined ? -0.32 : o.osc) })
  if (o.dentro) clip(f, o.dentro)
  ink(f, { width: o.tinta || TINTA, weight: 1.7, color: P.tinta })
  return f
}

// Brillo blanco de pegatina: una pincelada afilada.
function brillo(pts, ancho, op) {
  brush(pts, { width: [ancho * 0.35, ancho, ancho * 0.3], color: "#ffffff", opacity: op === undefined ? 0.95 : op })
}

// Llama de motor hacia abajo, desde la tobera (x, y), de radio r y largo dado.
function llama(x, y, r, largo) {
  const forma = (k) => [[x - r * k, y], [x, y - r * 0.5 * k], [x + r * k, y], [x + r * 0.55 * k, y + largo * 0.45 * k],
    [x, y + largo * k, 0], [x - r * 0.55 * k, y + largo * 0.45 * k]]
  blob(forma(1), { fill: P.motor, glow: { blur: 7, color: "#ff6a1aaa" } })
  blob(forma(0.68), { fill: P.llama })
  blob(forma(0.36), { fill: "#fffbe8" })
}

// ---------- nave del jugador (mira hacia arriba; núcleo vulnerable en el centro) ----------

function nave(w, h, fotograma) {
  const k = fotograma === 1 ? 1 : 1.35 // las llamas laten entre dos largos
  llama(30, 134, 8, 26 * k)
  llama(130, 134, 8, 26 * k)
  llama(80, 150, 11, 32 * k)

  const alaI = [[66, 70, 0], [38, 84], [9, 108, 0], [11, 124, 0], [36, 120], [66, 118, 0]]
  const vainaI = [[30, 60], [40, 68], [43, 98], [40, 126], [30, 136], [20, 126], [17, 98], [20, 68]]
  const fuselaje = [[80, 9, 0], [91, 32], [97, 62], [99, 100], [95, 132], [88, 152], [72, 152], [65, 132], [61, 100], [63, 62], [69, 32]]
  pegatina([alaI, espejo(alaI, 80), vainaI, espejo(vainaI, 80), fuselaje])

  // alas: azul verdoso, con una franja clara en el borde de ataque
  ;[alaI, espejo(alaI, 80)].forEach((ala, i) => {
    const s = i === 0 ? 1 : -1
    pieza(ala, P.acento, { sombra: 0.22, dentro: () => {
      brush([[80 - s * 18, 76], [80 - s * 44, 88], [80 - s * 72, 110]], { width: [3, 6, 3], color: "#bff6ff", opacity: 0.85 })
    } })
  })
  // vainas de los motores: blancas, con la toma naranja delante y la tobera oscura detrás
  ;[vainaI, espejo(vainaI, 80)].forEach((vaina, i) => {
    const cx = i === 0 ? 30 : 130
    pieza(vaina, P.casco, { sombra: 0.16, osc: -0.2, dentro: () => {
      rect(cx - 16, 54, 32, 20, { fill: P.motor })
      rect(cx - 16, 72, 32, 4, { fill: shade(P.motor, -0.4) })
    } })
    brillo([[cx - 6, 80], [cx - 8, 98], [cx - 6, 116]], 4)
  })
  // fuselaje: blanco, galón azul verdoso en el morro y franja central
  pieza(fuselaje, P.casco, { sombra: 0.14, osc: -0.2, dentro: () => {
    poly([[80, 20], [96, 40], [96, 50], [80, 34], [64, 50], [64, 40]], { fill: P.acento })
    rect(76, 88, 8, 50, { fill: P.acento })
  } })
  brillo([[70, 38], [65, 70], [66, 104]], 6)

  // cabina de cristal
  const cabina = blob(ring(80, 62, 11, 20, 12), { fill: P.cabina })
  clip(cabina, () => {
    ellipse(84, 72, 12, 14, { fill: shade(P.cabina, -0.45) })
    brush([[76, 50], [74, 60], [76, 68]], { width: [2, 5, 2], color: "#ffffff" })
  })
  ink(cabina, { width: 4, weight: 2, color: P.tinta })

  // núcleo: el único punto vulnerable de la nave
  circle(80, 104, 9, { fill: P.tinta })
  circle(80, 104, 5.5, { fill: P.nucleo, glow: { blur: 5, color: "#ffd447" } })
}

sprite("nave_1", (w, h) => nave(w, h, 1))
sprite("nave_2", (w, h) => nave(w, h, 2))

// ---------- enemigos (todos miran hacia abajo, hacia el jugador) ----------

// zumbon: dron insecto que vuela en formación. Dos fotogramas de aleteo.
function zumbon(w, h, fotograma) {
  const giro = fotograma === 1 ? -24 : 14
  ;[1, -1].forEach((s) => {
    const ala = blob(turn(ring(80 + s * 40, 80, 30, 12, 10), s * giro, 80 + s * 14, 86), { fill: alpha(P.ala, 0.78) })
    clip(ala, () => brush([[80 + s * 16, 84], [80 + s * 44, 80], [80 + s * 68, 78]], { width: [2, 3, 1], color: "#ffffff", opacity: 0.8 }))
    ink(ala, { width: 3, weight: 1.5, color: P.tinta })
  })
  const abdomen = ring(80, 50, 21, 28, 12)
  const torax = ring(80, 92, 27, 21, 12)
  const cabeza = ring(80, 122, 21, 16, 12)
  const mandibula = [[70, 130], [63, 150, 0], [76, 138]]
  pegatina([abdomen, torax, cabeza, mandibula, espejo(mandibula, 80)])
  pieza(mandibula, P.zumbonOsc, { tinta: 3.5 })
  pieza(espejo(mandibula, 80), P.zumbonOsc, { tinta: 3.5 })
  pieza(abdomen, P.zumbon, { dentro: () => [34, 50, 66].forEach((y) => rect(50, y, 60, 6, { fill: P.zumbonOsc })) })
  brillo([[68, 30], [64, 44], [66, 56]], 5)
  pieza(torax, P.zumbon, { dentro: () => ellipse(80, 92, 12, 9, { fill: shade(P.zumbon, 0.3) }) })
  pieza(cabeza, P.zumbonOsc, { dentro: () => {
    ellipse(80, 125, 15, 7, { fill: P.visor, glow: { blur: 4, color: "#ff3b5c" } })
    ellipse(75, 123, 5, 2.4, { fill: "#ffffff" })
  } })
}

// aguja: dardo que fija tu columna, avisa y se lanza en picado.
function aguja(w, h) {
  group({ rotate: 180, around: [80, 30] }, () => llama(80, 30, 8, 22))
  const aleta = [[72, 40, 0], [44, 26, 0], [42, 60], [66, 84, 0]]
  const cuerpo = [[80, 150, 0], [91, 112], [95, 70], [91, 40], [80, 26], [69, 40], [65, 70], [69, 112]]
  pegatina([aleta, espejo(aleta, 80), cuerpo])
  pieza(aleta, shade(P.aguja, -0.25), { tinta: 4 })
  pieza(espejo(aleta, 80), shade(P.aguja, -0.25), { tinta: 4 })
  pieza(cuerpo, P.aguja, { dentro: () => {
    poly([[80, 156], [100, 110], [60, 110]], { fill: P.agujaPunta })
    rect(60, 104, 40, 7, { fill: P.tinta })
    rect(78, 36, 4, 52, { fill: "#ffc2b0" })
  } })
  circle(80, 78, 8, { fill: P.tinta })
  circle(80, 78, 4.8, { fill: "#7cf8ff", glow: { blur: 4, color: "#2ad4ff" } })
  brillo([[73, 46], [70, 70], [72, 96]], 4)
}

// medusa: deriva lenta hacia abajo y late; los tentáculos cuelgan hacia el jugador.
function medusa(w, h) {
  for (let i = 0; i < 6; i++) {
    const x0 = 42 + i * 15.2
    const pts = range(5).map((j) => [x0 + Math.sin(j * 1.3 + i * 1.9) * 6, 84 + j * 17])
    brush(pts, { width: [9, 6.5, 3], color: P.tinta, taper: [0, 0.8] })
    brush(pts, { width: [4.5, 3, 1], color: P.tentaculo, taper: [0, 0.8] })
  }
  ;[-1, 1].forEach((s) => {
    const pts = [[80 + s * 8, 84], [80 + s * 15, 104], [80 + s * 6, 124], [80 + s * 13, 148]]
    brush(pts, { width: [17, 12, 5], color: P.tinta, taper: [0, 0.7] })
    brush(pts, { width: [10, 7, 2], color: P.medusaLuz, taper: [0, 0.7] })
  })
  const campana = [[22, 84], [28, 52], [48, 26], [80, 16], [112, 26], [132, 52], [138, 84], [124, 96], [109, 88], [94, 97],
    [80, 89], [66, 97], [51, 88], [36, 96]]
  pegatina([campana])
  pieza(campana, P.medusa, { sombra: 0.2, dentro: () => {
    ;[-44, -22, 0, 22, 44].forEach((dx) => curve([[80 + dx * 0.25, 20], [80 + dx * 0.8, 50], [80 + dx, 92]], { stroke: shade(P.medusa, 0.35), width: 3, opacity: 0.85 }))
    circle(80, 58, 16, { fill: P.medusaLuz, glow: { blur: 8, color: "#ffffff" } })
    circle(80, 58, 7, { fill: "#ffffff" })
  } })
  brillo([[38, 64], [44, 44], [60, 30]], 6)
}

// geoda: mina de cristal; al romperse suelta esquirlas en todas direcciones.
function geoda(w, h) {
  const angulos = [-90, -30, 30, 90, 150, 210]
  const cristales = angulos.map((a, i) => {
    const largo = i % 2 ? 46 : 50, abre = i % 2 ? 13 : 16
    const b1 = polar(60, 60, 16, a - 42), b2 = polar(60, 60, 16, a + 42)
    const h1 = polar(60, 60, largo - 14, a - abre), h2 = polar(60, 60, largo - 14, a + abre)
    const t = polar(60, 60, largo, a)
    return [[b1[0], b1[1], 0], [h1[0], h1[1], 0], [t[0], t[1], 0], [h2[0], h2[1], 0], [b2[0], b2[1], 0]]
  })
  const roca = jitter(ring(60, 60, 23, 23, 9), 2.5)
  pegatina(cristales.concat([roca]))
  cristales.forEach((c, i) => {
    const a = angulos[i], m = polar(60, 60, 18, a)
    pieza(c, i % 2 ? P.cristal2 : P.cristal, { sombra: 0.25, tinta: 3.5, dentro: () => {
      poly([m, c[1], c[2]], { fill: "#ffffff", opacity: 0.5 })
    } })
  })
  pieza(roca, P.roca, { tinta: 4 })
  circle(60, 60, 9, { fill: P.balaEnemiga, glow: { blur: 6, color: "#ff3df2" } })
  circle(57, 57, 3, { fill: "#ffffff" })
}

// vigia: ojo blindado. Cerrado mientras se coloca; se abre (aviso) y dispara en abanico.
function vigia(w, h, abierto) {
  const puas = [45, 135, 225, 315].map((a) => {
    const t = polar(80, 80, 74, a), b1 = polar(80, 80, 44, a - 17), b2 = polar(80, 80, 44, a + 17)
    return [b1, [t[0], t[1], 0], b2]
  })
  const casco = ring(80, 80, 55, 55, 16)
  pegatina(puas.concat([casco]))
  puas.forEach((p) => pieza(p, shade(P.vigia, -0.3), { tinta: 4 }))
  pieza(casco, P.vigia, { sombra: 0.18, dentro: () => {
    curve([[38, 48], [80, 34], [122, 48]], { stroke: shade(P.vigia, -0.35), width: 4 })
    curve([[38, 112], [80, 126], [122, 112]], { stroke: shade(P.vigia, -0.35), width: 4 })
    if (!abierto) {
      rect(20, 75, 120, 10, { fill: P.tinta })
      rect(44, 78.5, 72, 3, { fill: P.iris, glow: { blur: 4, color: P.iris } })
    } else {
      ellipse(80, 80, 47, 31, { fill: P.tinta })
    }
  } })
  if (abierto) {
    const blanco = blob(ring(80, 80, 39, 24, 14), { fill: P.ojo })
    clip(blanco, () => {
      circle(80, 82, 17, { fill: P.iris, glow: { blur: 6, color: "#ffb000" } })
      ellipse(80, 82, 5, 13, { fill: P.tinta })
      ellipse(84, 100, 34, 10, { fill: shade(P.ojo, -0.35), opacity: 0.7 })
      circle(72, 74, 4.5, { fill: "#ffffff" })
    })
    ink(blanco, { width: 4, weight: 1.6, color: P.tinta })
  }
  brillo([[42, 52], [54, 38], [72, 31]], 6)
}

sprite("zumbon_1", (w, h) => zumbon(w, h, 1))
sprite("zumbon_2", (w, h) => zumbon(w, h, 2))
sprite("aguja", aguja)
sprite("medusa", medusa)
sprite("geoda", geoda)
sprite("vigia_1", (w, h) => vigia(w, h, false))
sprite("vigia_2", (w, h) => vigia(w, h, true))

// ---------- proyectiles y recogibles ----------
// bala_jugador, bala_enemiga, esquirla, energia

// ---------- vista en juego y paleta ----------
