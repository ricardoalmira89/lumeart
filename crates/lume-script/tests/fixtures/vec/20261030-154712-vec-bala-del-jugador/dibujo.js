// dibujo.js: bala del jugador — rayo de energía cian que sube.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  halo: "#00d8ff",
  cian: "#12e6ff",
  cianOsc: "#00a8e0",
  cianClaro: "#8ff6ff",
  nucleo: "#ffffff",
}
const CX = 512
const TOP = 60      // punta
const BOT = 760     // fin de la cápsula, empieza la cola
const TAIL = 975    // fin de la cola

// Silueta de cápsula con la punta de arriba más redondeada y la base más estrecha
function capsula(w, top, bot, wBot) {
  const r = w / 2, rb = wBot / 2
  return [
    [CX, top],
    [CX + r * 0.72, top + r * 0.32],
    [CX + r, top + r * 1.1],
    [CX + r * 0.96, lerp(top, bot, 0.45)],
    [CX + rb, bot - rb * 0.6],
    [CX, bot + rb * 0.5],
    [CX - rb, bot - rb * 0.6],
    [CX - r * 0.96, lerp(top, bot, 0.45)],
    [CX - r, top + r * 1.1],
    [CX - r * 0.72, top + r * 0.32],
  ]
}

layer("halo", () => {
  // halo exterior muy suave
  blob(capsula(330, TOP - 30, TAIL - 20, 160), { fill: P.halo, opacity: 0.22, blur: 45 })
  blob(capsula(250, TOP - 12, BOT + 120, 140), { fill: P.halo, opacity: 0.35, blur: 26 })
  // resplandor de la punta
  circle(CX, TOP + 95, 150, { fill: radial(CX, TOP + 95, 150, [[0, alpha(P.cianClaro, 0.7)], [1, alpha(P.halo, 0)]]) })
})

layer("cola", () => {
  // cola que se desvanece hacia abajo
  const cola = (w, y0, y1) => [
    [CX - w / 2, y0], [CX + w / 2, y0],
    [CX + w * 0.3, lerp(y0, y1, 0.5)], [CX, y1, 0], [CX - w * 0.3, lerp(y0, y1, 0.5)],
  ]
  blob(cola(120, BOT - 100, TAIL), {
    fill: linear(0, BOT - 140, 0, TAIL, [[0, P.cian], [0.4, alpha(P.cian, 0.55)], [1, alpha(P.cian, 0)]]),
    blur: 10,
  })
  blob(cola(70, BOT - 140, TAIL - 70), {
    fill: linear(0, BOT - 140, 0, TAIL - 70, [[0, P.nucleo], [0.45, alpha(P.cianClaro, 0.6)], [1, alpha(P.cianClaro, 0)]]),
    blur: 6,
  })
})

layer("cuerpo", () => {
  // cápsula cian saturada
  const cuerpo = blob(capsula(160, TOP + 20, BOT, 110), {
    fill: linear(0, TOP, 0, BOT + 60, [[0, P.cianClaro], [0.35, P.cian], [0.8, P.cian], [1, alpha(P.cian, 0)]]),
    glow: { blur: 18, color: P.halo },
    blur: 3,
  })
  // banda interior más clara
  blob(capsula(112, TOP + 34, BOT - 30, 72), {
    fill: linear(0, TOP, 0, BOT, [[0, "#ffffff"], [0.4, P.cianClaro], [1, alpha(P.cianClaro, 0.5)]]),
    blur: 6,
  })
  // núcleo blanco
  blob(capsula(64, TOP + 48, BOT - 60, 34), {
    fill: linear(0, TOP, 0, BOT - 40, [[0, "#ffffff"], [0.75, "#ffffff"], [1, alpha("#ffffff", 0.4)]]),
    blur: 2,
  })
})

layer("punta", () => {
  // punta más brillante
  ellipse(CX, TOP + 85, 62, 70, { fill: radial(CX, TOP + 85, 70, [[0, "#ffffff"], [0.6, alpha("#ffffff", 0.85)], [1, alpha(P.cianClaro, 0)]]), blur: 4 })
  circle(CX, TOP + 82, 40, { fill: "#ffffff", blur: 6, glow: { blur: 20, color: P.cianClaro } })
  // pequeños destellos en cruz
  brush([[CX - 120, TOP + 78], [CX + 120, TOP + 78]], { width: 10, taper: [0.5, 0.5], color: alpha("#ffffff", 0.75), blur: 1 })
  brush([[CX, TOP - 30], [CX, TOP + 190]], { width: 9, taper: [0.5, 0.5], color: alpha("#ffffff", 0.6) })
})
