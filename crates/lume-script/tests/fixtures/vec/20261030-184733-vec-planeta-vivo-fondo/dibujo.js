// dibujo.js: planeta vivo (fondo) — el último mundo habitado del sistema
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)
seed(7)

const P = {
  oceanLight: "#4a90bb", ocean: "#2a6590", oceanDeep: "#173d63", night: "#0d1f3a",
  green: "#5a9257", greenDark: "#3b6c48", ochre: "#b39a62", ochreDark: "#8a7350",
  cloud: "#e4edf1", atmo: "#8fd0f0", city: "#f2c062",
  moon: "#9b9da4", moonDark: "#3f4658",
}

const CX = 505, CY = 530, R = 395, TILT = -18

// proyección ortográfica: lon/lat en grados -> punto del lienzo (con inclinación del eje)
function proj(lon, lat) {
  const a = lon * Math.PI / 180, b = lat * Math.PI / 180
  const x = R * Math.cos(b) * Math.sin(a), y = -R * Math.sin(b)
  const t = TILT * Math.PI / 180
  return [CX + x * Math.cos(t) - y * Math.sin(t), CY + x * Math.sin(t) + y * Math.cos(t)]
}
function projPts(pts) { return pts.map(([lo, la, s]) => s === undefined ? proj(lo, la) : [...proj(lo, la), s]) }
// contorno de continente en lon/lat
function land(lon, lat, rl, rb, n, rot) {
  return ring(lon, lat, rl, rb, n, rot).map(([x, y]) => [x + rand(-rl, rl) * 0.28, y + rand(-rb, rb) * 0.28])
}

let planet

layer("atmosfera exterior", { parallax: 0.15 }, () => {
  // halo azul muy tenue, desplazado hacia la luz (el planeta tapa el centro)
  // capas finas que se adelgazan hacia el terminador (sin extremos duros)
  for (const [dr, op] of [[1, 0.2], [4, 0.13], [8, 0.08], [13, 0.045], [19, 0.025]]) {
    const pts = range(32).map(k => {
      const t = (25 + 220 * k / 31) * Math.PI / 180
      return [CX + (R + dr) * Math.cos(t), CY - (R + dr) * Math.sin(t)]
    })
    brush(pts, { width: u => 7 * Math.sin(Math.PI * u), taper: [0, 0], color: alpha(P.atmo, op), blur: 2 })
  }
})

element("planeta", { pivot: [CX, CY] }, () => {
  layer("oceano", () => {
    planet = circle(CX, CY, R, { fill: radial(CX, CY, R, [[0, P.ocean], [0.85, P.ocean], [1, P.oceanDeep]], { fx: 340, fy: 360 }) })
    clip(planet, () => {
      circle(360, 370, 260, { fill: P.oceanLight, opacity: 0.45, blur: 90 })
    })
    // corrientes marinas sutiles
    clip(planet, () => {
      for (let i = 0; i < 9; i++) {
        const lat = rand(-60, 60), lon0 = rand(-90, -10)
        const pts = range(8).map(k => proj(lon0 + k * 14, lat + Math.sin(k * 0.8) * 4))
        brush(pts, { width: rand(10, 22), color: alpha(P.oceanLight, 0.12), blur: 6 })
      }
    })
  })
  layer("continentes", () => {
    clip(planet, () => {
      const conts = [
        // [lon, lat, radio lon, radio lat, puntos, color]
        [-40, 26, 20, 15, 10, P.green],
        [-22, 44, 12, 8, 8, P.greenDark],
        [14, -4, 16, 22, 11, mix(P.green, P.ochre, 0.55)],
        [30, 22, 14, 10, 9, P.green],
        [-48, -28, 13, 11, 9, P.green],
        [52, -36, 15, 9, 9, P.greenDark],
        [-68, 58, 12, 5, 7, P.green],
        [64, 18, 9, 12, 8, P.ochre],
        [-10, -42, 10, 6, 7, P.green],
      ]
      for (const [lo, la, rl, rb, n, col] of conts) {
        const pts = roughen(subdivide(projPts(land(lo, la, rl, rb, n, rand(0, 90))), 16), 5, { detail: 2 })
        const s = blob(pts, { fill: col, blur: 1.5 })
        clip(s, () => {
          // manchas de ocre/verde interiores (desiertos, bosques)
          for (let k = 0; k < 5; k++) {
            const c = proj(lo + rand(-rl, rl) * 0.6, la + rand(-rb, rb) * 0.6)
            ellipse(c[0], c[1], rand(18, 50), rand(12, 30), {
              fill: pick([P.ochre, P.greenDark, P.ochreDark, mix(P.green, P.ochre, 0.5)]),
              opacity: 0.55, blur: 14, rotate: rand(0, 180) })
          }
          // cordilleras
          const m = range(5).map(k => proj(lo - rl * 0.5 + k * rl * 0.25, la + rand(-rb, rb) * 0.3))
          brush(m, { width: 6, color: alpha(P.ochreDark, 0.5), blur: 2 })
        })
        // aguas someras alrededor de la costa
        blob(pts, { fill: "none", stroke: alpha("#6fb3c9", 0.22), width: 10, blur: 6, behind: true })
      }
    })
  })
  layer("nubes", () => {
    clip(planet, () => {
      // remolinos (borrascas) en espiral
      const swirl = (lo, la, r0, turns, dir) => {
        const c = proj(lo, la)
        // masa nubosa difusa alrededor del ojo
        circle(c[0], c[1], r0 * 4.2, { fill: P.cloud, opacity: 0.12, blur: 22 })
        for (let arm = 0; arm < 2; arm++) {
          const off = arm * Math.PI + rand(-0.4, 0.4), rr = r0 * rand(0.8, 1.2)
          const pts = range(16).map(k => {
            const u = k / 15, a = off + dir * u * turns * 0.6 * Math.PI * 2, r = rr * (1.2 - u * 0.95)
            return proj(lo + Math.cos(a) * r * 1.3 + rand(-1, 1), la + Math.sin(a) * r + rand(-1, 1))
          })
          brush(pts, { width: u => 6 + 16 * (1 - u), taper: [0.4, 0.3], color: alpha(P.cloud, 0.24), blur: 6 })
          brush(pts, { width: u => 14 + 30 * (1 - u), taper: [0.4, 0.3], color: alpha(P.cloud, 0.1), blur: 14 })
        }
      }
      swirl(-20, 12, 14, 1.4, 1)
      swirl(40, -22, 11, 1.3, -1)
      swirl(-55, -50, 9, 1.2, 1)
      swirl(10, 52, 10, 1.1, -1)
      // bandas largas de nubes arrastradas por el viento
      for (let i = 0; i < 12; i++) {
        const lat = rand(-70, 70), lo0 = rand(-95, 40), len = rand(20, 45)
        const pts = range(6).map(k => proj(lo0 + k * len / 5, lat + Math.sin(k * 0.9 + i) * rand(0.5, 2)))
        brush(pts, { width: rand(10, 20), color: alpha(P.cloud, rand(0.14, 0.24)), blur: rand(5, 8) })
      }
      // velo nuboso disperso
      for (let i = 0; i < 22; i++) {
        const c = proj(rand(-80, 80), rand(-70, 70))
        ellipse(c[0], c[1], rand(20, 45), rand(6, 14), { fill: P.cloud, opacity: 0.1, blur: 10, rotate: TILT + rand(-10, 10) })
      }
      // casquete polar suave
      const pole = proj(0, 84)
      ellipse(pole[0], pole[1], 110, 28, { fill: P.cloud, opacity: 0.35, blur: 14, rotate: TILT })
    })
  })
  layer("noche", () => {
    clip(planet, () => {
      // sombra propia: el lado contrario a la luz se hunde en azul marino
      circle(CX + 330, CY + 330, 440, { fill: P.night, opacity: 0.55, blur: 120, blend: "multiply" })
      circle(CX + 330, CY + 330, 410, { fill: P.night, opacity: 0.85, blur: 60 })
      // contraluz fría muy suave en el borde de sombra
      clip(circle(CX + 260, CY + 260, 330, { hidden: true }), () => {
        circle(CX, CY, R, { fill: "none", stroke: alpha("#3d6fa8", 0.18), width: 24, blur: 14 })
      })
      // atenuación general (poco contraste)
      circle(CX, CY, R, { fill: "#1a3352", opacity: 0.12 })
    })
  })
  layer("ciudades", () => {
    clip(planet, () => {
      const hubs = [[52, -36, 12], [60, -28, 7], [-8, -42, 8], [22, -18, 8], [70, 0, 6], [40, -50, 5]]
      for (const [lo, la, n] of hubs) {
        for (let i = 0; i < n; i++) {
          const p = proj(lo + rand(-7, 7), la + rand(-5, 5))
          if (Math.hypot(p[0] - (CX + 330), p[1] - (CY + 330)) > 360) continue
          const r = rand(1.2, 2.6)
          circle(p[0], p[1], r * 3, { fill: P.city, opacity: 0.18, blur: 4 })
          circle(p[0], p[1], r, { fill: P.city, opacity: rand(0.55, 0.85) })
        }
        // carreteras entre ciudades: hilos tenues
        const a = proj(lo - 5, la - 2), b = proj(lo + 5, la + 3)
        const inNight = p => Math.hypot(p[0] - (CX + 330), p[1] - (CY + 330)) < 340
        if (inNight(a) && inNight(b)) brush([a, [(a[0] + b[0]) / 2 + 6, (a[1] + b[1]) / 2], b], { width: 1.5, color: alpha(P.city, 0.25) })
      }
    })
  })
  layer("atmosfera", () => {
    const rim = (r, a0, a1) => range(24).map(k => {
      const t = (a0 + (a1 - a0) * k / 23) * Math.PI / 180
      return [CX + r * Math.cos(t), CY - r * Math.sin(t)]
    })
    clip(planet, () => {
      // brillo azul claro dentro del borde iluminado
      brush(rim(R - 6, 30, 240), { width: u => 34 * Math.sin(Math.PI * u), taper: [0, 0],
        color: alpha(P.atmo, 0.25), blur: 14, blend: "screen" })
      brush(rim(R - 2, 50, 220), { width: u => 8 * Math.sin(Math.PI * u), taper: [0, 0],
        color: alpha(P.atmo, 0.2), blur: 4, blend: "screen" })
      grain({ opacity: 0.06, size: 1.2 })
      // calidez de la luz principal
      circle(330, 350, 220, { fill: "#f3d9a4", opacity: 0.1, blur: 80, blend: "soft-light" })
    })
  })
})

element("luna", { pivot: [888, 150] }, () => {
  const M = [888, 150, 50]
  const moon = circle(M[0], M[1], M[2], { fill: radial(M[0], M[1], M[2], [P.moon, mix(P.moon, P.moonDark, 0.5)], { fx: 865, fy: 128 }) })
  clip(moon, () => {
    // cráteres
    for (let i = 0; i < 9; i++) {
      const c = polar(M[0], M[1], rand(0, 40), rand(0, 360)), r = rand(3, 10)
      circle(c[0], c[1], r, { fill: shade(P.moon, -0.25), opacity: 0.45, blur: 1 })
      circle(c[0] - r * 0.25, c[1] - r * 0.25, r * 0.7, { fill: shade(P.moon, 0.15), opacity: 0.25, blur: 1 })
    }
    // sombra propia
    circle(M[0] + 50, M[1] + 50, 62, { fill: P.night, opacity: 0.75, blur: 16 })
    circle(M[0], M[1], M[2], { fill: "#1a3352", opacity: 0.1 })
  })
})
