// Fondo nivel 1: espacio profundo junto a la Colmena.
canvas({ width: 768, height: 1024, background: linear(0, 0, 0, 1024, ["#0b1022", "#080b18"]) })

const P = {
  cielo: "#0a0e1c",
  nebVio: "#231c3d",
  nebVio2: "#2c2349",
  nebVerde: "#16292a",
  roca: "#1b2030",
  rocaSombra: "#121522",
  rocaLuz: "#262c3e",
  celda: "#0e1a1a",
  verde: "#3e7a52",
  estrella: "#8590ad",
}
setLight(-1, -1)

// Banda diagonal: puntos a lo largo de la línea de arriba-izq a abajo-der
function banda(off, ancho, amp, sem) {
  seed(sem)
  const top = [], bot = []
  for (let i = 0; i <= 8; i++) {
    const t = i / 8
    const x = lerp(-150, 920, t), y = lerp(80, 980, t) + off
    const a = ancho * (0.6 + 0.5 * Math.sin(t * Math.PI)) + rand(-amp, amp)
    top.push([x + a * 0.55, y - a * 0.8])
    bot.unshift([x - a * 0.55, y + a * 0.8])
  }
  return top.concat(bot)
}

layer("nebulosa", { parallax: 0.1 }, () => {
  blob(banda(-40, 260, 40, 3), { fill: P.nebVio, opacity: 0.55, blur: 30 })
  blob(banda(-10, 150, 30, 5), { fill: P.nebVio2, opacity: 0.5, blur: 22 })
  blob(banda(90, 120, 40, 8), { fill: P.nebVerde, opacity: 0.5, blur: 26 })
  blob(banda(-120, 70, 25, 11), { fill: P.nebVerde, opacity: 0.45, blur: 18 })
  // nudos más densos
  ellipse(180, 300, 120, 60, { fill: P.nebVio2, opacity: 0.35, blur: 30, rotate: 40 })
  ellipse(560, 760, 140, 60, { fill: P.nebVerde, opacity: 0.4, blur: 30, rotate: 40 })
})

layer("estrellas", { parallax: 0.05 }, () => {
  seed(42)
  for (let i = 0; i < 260; i++) {
    const x = rand(0, 768), y = rand(0, 1024)
    const r = rand() < 0.85 ? rand(0.5, 1.1) : rand(1.2, 1.9)
    circle(x, y, r, { fill: mix(P.estrella, pick(["#6a7a9a", "#7d8f80", "#8a7fa8"]), 0.5), opacity: rand(0.35, 0.8) })
  }
  // algunas con destello leve
  seed(7)
  for (let i = 0; i < 9; i++) {
    const x = rand(20, 748), y = rand(20, 1000), s = rand(5, 9)
    circle(x, y, s * 0.9, { fill: "#6e7aa0", opacity: 0.15, blur: 3 })
    poly([[x, y - s], [x + 0.9, y], [x, y + s], [x - 0.9, y]], { fill: P.estrella, opacity: 0.6 })
    poly([[x - s, y], [x, y + 0.9], [x + s, y], [x, y - 0.9]], { fill: P.estrella, opacity: 0.6 })
    circle(x, y, 1.4, { fill: "#a3abc4", opacity: 0.8 })
  }
})

// Asteroide plano con sombra y luz recortadas
function asteroide(cx, cy, r, sem, rot = 0) {
  seed(sem)
  const pts = roughen(ring(cx, cy, r, r * 0.8, 9, rot), r * 0.12, { detail: 1 })
  const a = blob(pts, { fill: P.roca })
  clip(a, () => {
    circle(cx + r * 0.45, cy + r * 0.45, r * 1.05, { fill: P.rocaSombra })
    circle(cx - r * 0.55, cy - r * 0.6, r * 0.6, { fill: P.rocaLuz })
    for (let i = 0; i < 3; i++) {
      const p = [cx + rand(-r, r) * 0.5, cy + rand(-r, r) * 0.4]
      ellipse(p[0], p[1], r * rand(0.12, 0.2), r * rand(0.1, 0.15), { fill: P.rocaSombra, opacity: 0.8 })
    }
  })
  return a
}

layer("asteroides lejanos", { parallax: 0.2 }, () => {
  element("asteroide 1", () => { asteroide(118, 150, 26, 1, 20) })
  element("asteroide 2", () => { asteroide(250, 92, 11, 2) })
  element("asteroide 3", () => { asteroide(38, 470, 18, 3, 60) })
  element("asteroide 4", () => { asteroide(724, 640, 14, 4, 10) })
  element("asteroide 5", () => { asteroide(470, 60, 8, 5) })
})

layer("colmena", { parallax: 0.3 }, () => {
  element("colmena", { pivot: [610, 250] }, () => {
    const cx = 610, cy = 250, R = 170
    seed(21)
    // halo verde muy tenue
    circle(cx, cy, R * 1.25, { fill: P.verde, opacity: 0.07, blur: 40 })
    const contorno = [[470, 150], [540, 92, 0.5], [640, 78], [730, 110], [790, 190], [800, 330], [760, 410, 0.5], [660, 440], [560, 420], [470, 370, 0.5], [440, 270]]
    const forma = blob(contorno, { fill: P.rocaSombra })
    const hex = (x, y, s) => range(6).map(k => polar(x, y, s, k * 60 + 30))
    clip(forma, () => {
      // cara iluminada (arriba a la izquierda) y media luz, como formas planas
      circle(cx - 40, cy - 45, R * 1.0, { fill: P.roca })
      circle(cx - 85, cy - 95, R * 0.62, { fill: P.rocaLuz })
      // celdas hexagonales: más pequeñas hacia el borde para sugerir curvatura
      const s = 15, w = s * Math.sqrt(3)
      for (let row = -13; row <= 13; row++) {
        for (let col = -11; col <= 11; col++) {
          const x = cx + col * w + (row % 2 ? w / 2 : 0), y = cy + row * s * 1.5
          const d = Math.hypot(x - cx, (y - cy) / 0.86) / R
          if (d > 0.95 || rand() < 0.12) continue
          const k = s * 0.9 * (1 - d * d * 0.45)
          const enLuz = Math.hypot(x - (cx - 85), y - (cy - 95)) < R * 0.62
          const enMedia = Math.hypot(x - (cx - 40), y - (cy - 45)) < R * 1.0
          // labio de luz arriba a la izquierda (según la zona) y hueco oscuro
          if (enLuz || enMedia) poly(hex(x - 1, y - 1.2, k), { fill: enLuz ? "#2f3649" : "#232838" })
          poly(hex(x + 0.8, y + 1, k * 0.86), { fill: P.celda })
          const g = rand()
          if (g < 0.5) poly(hex(x + 1.6, y + 2, k * 0.55), { fill: P.verde, opacity: rand(0.15, 0.38) })
          if (g < 0.12) poly(hex(x + 1.6, y + 2, k * 0.3), { fill: "#5f9a68", opacity: 0.45 })
        }
      }
      // terminador: oscurece suavemente la parte de abajo a la derecha
      circle(cx + 120, cy + 135, R * 0.95, { fill: "#05070d", opacity: 0.35 })
    })
  })
})

layer("textura", { parallax: 0.3 }, () => {
  grain({ opacity: 0.06, size: 1, blend: "overlay" })
})
