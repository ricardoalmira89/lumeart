// dibujo.js: galaxia espiral lejana para fondo con parallax.
canvas({ width: 1024, height: 1024, background: null })
seed(7)

const P = {
  halo: "#2a2550",
  violet: "#6a58b0",
  blue: "#4a6fc0",
  pink: "#b06a9e",
  pale: "#c9c4f0",
  coreOut: "#e8b86a",
  core: "#ffd98a",
  coreHot: "#fff4d0",
  dust: "#0a0c1e",
}

const CX = 512, CY = 512, TILT = 0.8, ROT = -25 * Math.PI / 180
const RMAX = 495
// Proyección del disco: coordenadas polares del plano galáctico -> lienzo
function proj(r, th) {
  const x = r * Math.cos(th), y = r * Math.sin(th) * TILT
  return [CX + x * Math.cos(ROT) - y * Math.sin(ROT), CY + x * Math.sin(ROT) + y * Math.cos(ROT)]
}
// Espiral logarítmica: r = A·e^(B·t)
const A = 34, B = 0.3
const TMAX = Math.log(RMAX / A) / B
function armPts(phase, t0, t1, n, dr = 0) {
  return range(n).map(i => {
    const t = lerp(t0, t1, i / (n - 1))
    return proj(A * Math.exp(B * t) + dr, t + phase)
  })
}
const fade = r => 1 - smoothstep(RMAX * 0.55, RMAX, r)
const ARMS = [
  { phase: 0, k: 1 },
  { phase: Math.PI, k: 1 },
  { phase: Math.PI * 0.5, k: 0.45 },
]

layer("halo", () => {
  // Resplandor difuso del disco, se desvanece a transparente
  group({ rotate: -25, around: [CX, CY], blur: 30 }, () => {
    // Pila de elipses tenues: el centro acumula más luz y el borde se pierde
    const N = 36
    for (let i = 0; i < N; i++) {
      const u = i / (N - 1)
      const rr = lerp(450, 50, Math.pow(u, 0.8))
      ellipse(CX, CY, rr, rr * TILT, { fill: mix(P.halo, P.violet, u), opacity: 0.018 + 0.02 * u })
    }
  })
})

layer("brazos", () => {
  // Nubes difusas a lo largo de cada brazo
  for (const arm of ARMS) group({ blur: 14 }, () => {
    const n = 90
    for (let i = 0; i < n; i++) {
      const t = lerp(0.6, TMAX, i / (n - 1)) + rand(-0.05, 0.05)
      const r = A * Math.exp(B * t)
      const [x, y] = proj(r + rand(-12, 12), t + arm.phase)
      const u = r / RMAX
      const c = u < 0.3 ? mix(P.pale, P.violet, u / 0.3) : mix(P.violet, i % 3 ? P.blue : P.pink, Math.min(1, (u - 0.3) * 1.6))
      ellipse(x, y, 22 + 40 * u, (16 + 28 * u), { fill: c, opacity: 0.085 * arm.k * fade(r), rotate: rand(0, 180) })
    }
    // Cresta más luminosa del brazo
    brush(armPts(arm.phase, 0.8, TMAX * 0.95, 40), { width: u => 8 + 26 * u, taper: [0.1, 0.7], color: alpha(P.pale, 0.1 * arm.k) })
    brush(armPts(arm.phase, 2, TMAX * 0.9, 40, 6), { width: u => 3 + 10 * u, taper: [0.2, 0.7], color: alpha(P.blue, 0.14 * arm.k) })
    // Velo rosado en la mitad exterior del brazo
    brush(armPts(arm.phase + 0.08, TMAX * 0.5, TMAX * 0.95, 30, 10), { width: u => 14 + 30 * u, taper: [0.4, 0.6], color: alpha(P.pink, 0.07 * arm.k) })
  })
})

layer("polvo", () => {
  // Franjas oscuras por el borde interno de cada brazo
  for (const arm of ARMS.slice(0, 2)) {
    group({ blur: 7, opacity: 0.8 }, () => {
      brush(armPts(arm.phase - 0.32, 1.2, TMAX * 0.82, 40), { width: u => 4 + 18 * u, taper: [0.3, 0.5], color: alpha(P.dust, 0.45) })
    })
    group({ blur: 3 }, () => {
      brush(armPts(arm.phase - 0.22, 2.2, TMAX * 0.7, 30, -4), { width: u => 2 + 6 * u, taper: [0.3, 0.5], color: alpha(P.dust, 0.3) })
    })
  }
})

layer("estrellas", () => {
  // Estrellas diminutas: la mayoría sobre los brazos, algunas en el disco
  for (let i = 0; i < 700; i++) {
    let x, y, r
    if (rand() < 0.7) {
      const arm = pick(ARMS)
      const t = rand(0.8, TMAX)
      r = A * Math.exp(B * t) + rand(-25, 25) * (0.4 + t / TMAX)
      ;[x, y] = proj(r, t + arm.phase + rand(-0.12, 0.12))
    } else {
      r = RMAX * Math.sqrt(rand()) * 0.95
      ;[x, y] = proj(r, rand(0, Math.PI * 2))
    }
    const c = pick([P.pale, "#ffffff", "#aac4ff", "#e6d0ff"])
    circle(x, y, rand(0.6, 1.6), { fill: c, opacity: rand(0.25, 0.6) * fade(Math.abs(r)) })
  }
  // Nudos brillantes (regiones de formación estelar)
  for (let i = 0; i < 45; i++) {
    const arm = pick(ARMS.slice(0, 2))
    const t = rand(2.2, TMAX * 0.92)
    const r = A * Math.exp(B * t) + rand(-10, 10)
    const [x, y] = proj(r, t + arm.phase + rand(-0.05, 0.05))
    const c = rand() < 0.55 ? P.pink : "#9fb8ff"
    const s = rand(2, 5)
    circle(x, y, s * 2.4, { fill: c, opacity: 0.18 * fade(r), blur: s * 1.5 })
    circle(x, y, s * 0.5, { fill: mix(c, "#ffffff", 0.5), opacity: 0.5 * fade(r), blur: 0.6 })
  }
})

layer("nucleo", () => {
  // Núcleo pequeño y cálido: lo único brillante de la pieza
  group({ rotate: -25, around: [CX, CY] }, () => {
    group({ blur: 22 }, () => {
      for (let i = 0; i < 12; i++) {
        const rr = lerp(140, 30, i / 11)
        ellipse(CX, CY, rr, rr * TILT, { fill: mix(P.coreOut, P.core, i / 11), opacity: 0.07 })
      }
    })
    group({ blur: 8 }, () => {
      ellipse(CX, CY, 40, 40 * TILT, { fill: P.core, opacity: 0.6 })
      ellipse(CX, CY, 20, 20 * TILT, { fill: P.coreHot, opacity: 0.9 })
    })
    circle(CX, CY, 7, { fill: "#fffbe8", blur: 2 })
  })
})
