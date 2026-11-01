// dibujo.js: asteroide infestado (fondo)
canvas({ width: 1024, height: 1024, background: null })

const P = {
  rockL: "#6a5d50", rockM: "#4a4139", rockD: "#2b2622", rockS: "#1b1c24",
  shadow: "#0e1426",
  chD: "#262c15", chM: "#3d4522", chL: "#5a6331", memb: "#1d2112",
  lime: "#a6dc4a", amber: "#e2a33c",
  egg: "#86a35a", eggD: "#34461f",
  warm: "#d8b98a", cold: "#3d5878",
}

seed(11)
const C = [512, 512], R = 450
const Lv = (() => { const l = [-0.55, -0.62, 0.56]; const m = Math.hypot(...l); return l.map(x => x / m) })()

// proyección azimutal: (a, b) en radianes sobre la superficie -> punto del lienzo
function sp(a, b) {
  const d = Math.hypot(a, b) || 1e-6
  const k = R * Math.sin(Math.min(d, Math.PI / 2)) / d
  return [C[0] + a * k, C[1] + b * k]
}
function lit(a, b) {
  const d = Math.hypot(a, b) || 1e-6
  const n = [Math.sin(d) * a / d, Math.sin(d) * b / d, Math.cos(d)]
  return n[0] * Lv[0] + n[1] * Lv[1] + n[2] * Lv[2]
}
// máscara de la colmena (lado derecho e inferior)
function hiveF(a, b) {
  return a * 0.9 + b * 0.55 + 0.22 * Math.sin(b * 4 + 1) + 0.18 * Math.sin(a * 5 + 2)
}

layer("asteroide", { parallax: 0.3 }, () => {
  element("asteroide_infestado", { pivot: [512, 512] }, () => {
    // silueta de la roca
    const radii = [0.95, 0.86, 0.93, 0.97, 0.84, 0.9, 0.96, 0.88, 0.82, 0.93, 0.97, 0.87]
    const base = radii.map((k, i) => polar(C[0], C[1], R * k, i / radii.length * 360 + rand(-8, 8)))
    const rockF = blob(roughen(base, 10, { detail: 2 }), {
      fill: radial(340, 300, 760, [[0, P.rockL], [0.35, P.rockM], [0.7, P.rockD], [1, P.rockS]]),
    })
    clip(rockF, () => {
      // facetas suaves de la roca
      for (let i = 0; i < 9; i++) {
        const a = rand(-1.3, 1.3), b = rand(-1.3, 1.3)
        const p = sp(a, b), l = lit(a, b)
        blob(jitter(ring(p[0], p[1], rand(60, 130), rand(40, 90), 7, rand(0, 180)), 18), {
          fill: l > 0.3 ? P.rockL : P.rockD, opacity: 0.35, blur: 30,
        })
      }
    })

    clip(rockF, () => {
      // CRÁTERES (en la zona de roca desnuda)
      for (let i = 0; i < 60; i++) {
        const a = rand(-1.4, 0.6), b = rand(-1.4, 0.9)
        if (hiveF(a, b) > -0.35 || Math.hypot(a, b) > 1.45) continue
        const d = Math.hypot(a, b), p = sp(a, b), r = rand(14, 46)
        const squash = Math.cos(Math.min(d, 1.45))
        const rot = Math.atan2(b, a) * 180 / Math.PI
        const tone = mix(P.rockM, P.rockL, clamp(lit(a, b), 0, 1) * 0.6)
        // borde levantado con luz, sombra interior arriba-izquierda y fondo
        ellipse(p[0] + r * 0.06, p[1] + r * 0.06, r * squash * 1.12, r * 1.12, { fill: P.rockL, opacity: 0.18, blur: 4, rotate: rot })
        const cr = ellipse(p[0], p[1], r * squash, r, { fill: P.rockD, opacity: 0.6, blur: 1.5, rotate: rot })
        clip(cr, () => {
          ellipse(p[0] + r * 0.32, p[1] + r * 0.32, r * squash, r, { fill: tone, opacity: 0.7, blur: r * 0.15, rotate: rot })
        })
      }

      // COLMENA
      const s = 0.078, rh = s / Math.sqrt(3)
      const cells = []
      for (let j = -22; j <= 22; j++) {
        for (let i = -22; i <= 22; i++) {
          // la rejilla se ondula un poco para que no parezca un suelo de baldosas
          const a0 = i * s + (j & 1 ? s / 2 : 0), b0 = j * s * 0.866
          const a = a0 + 0.02 * Math.sin(b0 * 6) + rand(-0.005, 0.005)
          const b = b0 + 0.02 * Math.sin(a0 * 5 + 1) + rand(-0.005, 0.005)
          if (Math.hypot(a, b) > 1.5) continue
          const f = hiveF(a, b)
          if (f < -0.3) continue
          if (f < -0.08 && rand() < 0.55) continue // borde que se deshace
          cells.push({ a, b, f, l: lit(a, b), k: f < -0.08 ? rand(0.5, 0.8) : rand(0.82, 0.95) })
        }
      }
      for (const c of cells) c.w = range(6).map(() => rand(0.9, 1.08))
      const hexPts = (c, k) => range(6).map(n => {
        const t = (30 + n * 60) * Math.PI / 180, kk = k * c.w[n]
        const p = sp(c.a + Math.cos(t) * rh * kk, c.b + Math.sin(t) * rh * kk)
        return [p[0], p[1], 0.35]
      })
      // membrana orgánica que une las placas (borde blando)
      for (const c of cells) {
        const p = sp(c.a, c.b)
        if (c.f < -0.08) circle(p[0], p[1], rh * R * 1.05, { fill: P.memb, blur: 3 })
        else circle(p[0], p[1], rh * R * 1.35, { fill: P.memb })
      }
      for (const c of cells) {
        if (c.f > -0.12) continue
        const p = sp(c.a, c.b)
        circle(p[0], p[1], rh * R * 1.6, { fill: P.memb, opacity: 0.45, blur: 8 })
      }
      // placas de quitina
      for (const c of cells) {
        const p = sp(c.a, c.b)
        const t = clamp(c.l * 0.55 + 0.4 + rand(-0.08, 0.08), 0, 1)
        const top = mix(P.chM, P.chL, t), bot = mix(P.chD, P.chM, t * 0.8)
        const q = 22 * c.k
        c.top = top
        blob(hexPts(c, c.k), { fill: linear(p[0] - q, p[1] - q, p[0] + q, p[1] + q, [top, bot]) })
        c.open = c.f > 0.0 && rand() < 0.1
        if (c.open) {
          blob(hexPts(c, c.k * 0.66), { fill: "#121506" })
        } else if (rand() < 0.35) {
          // placa abombada: pequeño brillo hacia la luz
          circle(p[0] - 4, p[1] - 4, rh * R * 0.35, { fill: top, opacity: 0.4, blur: 4 })
        }
      }
      // FILAMENTOS: raíces que se agarran a la roca desde el borde de la colmena
      const edge = cells.filter(c => c.f < -0.02)
      // raíces gruesas y pegadas a la superficie, que se abren en dedos
      const root = (a, b, da, db, len, w, depth) => {
        const n = 7, pts = [], ab = []
        const curl = rand(-0.28, 0.28)
        for (let k = 0; k < n; k++) {
          pts.push(sp(a, b)); ab.push([a, b, da, db])
          a += da * len / (n - 1); b += db * len / (n - 1)
          const tw = curl + rand(-0.12, 0.12); const ca = Math.cos(tw), sa = Math.sin(tw)
          ;[da, db] = [da * ca - db * sa, da * sa + db * ca]
        }
        // tallo de grosor casi constante: un tramo grueso y uno más fino hacia la punta
        const h1 = pts.slice(0, 5), h2 = pts.slice(3)
        curve(shift(pts, 2, 3), { stroke: "#14130f", width: w * 1.1, cap: "round", opacity: 0.3, blur: 3 })
        curve(h1, { stroke: P.memb, width: w, cap: "round" })
        curve(h2, { stroke: P.memb, width: w * 0.65, cap: "round" })
        curve(shift(h1, -w * 0.15, -w * 0.15), { stroke: P.chD, width: w * 0.35, cap: "round" })
        if (depth > 0) {
          const nb = Math.floor(rand(1, 3))
          for (let q = 0; q < nb; q++) {
            const [ba, bb, bda, bdb] = ab[Math.floor(rand(2, 5))]
            const tw = (rand() < 0.5 ? -1 : 1) * rand(0.4, 0.8)
            root(ba, bb, bda * Math.cos(tw) - bdb * Math.sin(tw), bda * Math.sin(tw) + bdb * Math.cos(tw),
              len * rand(0.35, 0.55), w * 0.5, depth - 1)
          }
        }
      }
      const outer = edge.filter(c => c.f < -0.05 && Math.hypot(c.a, c.b) < 1.35)
      for (let i = 0; i < outer.length; i += 2) {
        const c = outer[i]
        let da = -0.9 + rand(-0.8, 0.8), db = -0.55 + rand(-0.8, 0.8)
        const m = Math.hypot(da, db)
        root(c.a, c.b, da / m, db / m, rand(0.14, 0.34), rand(8, 14), 1)
      }

      // HUEVOS / capullos translúcidos agrupados abajo a la izquierda
      const eggs = []
      for (let i = 0; i < 18; i++) {
        const a = rand(-0.72, -0.22), b = rand(0.5, 0.95)
        eggs.push({ p: sp(a, b), rx: rand(13, 22), ry: rand(18, 28), rot: rand(-35, 35) })
      }
      eggs.sort((u, v) => u.p[1] - v.p[1])
      blob(jitter(ring(330, 790, 120, 95, 10, 20), 15), { fill: P.memb, opacity: 0.85, blur: 6 })
      for (const e of eggs) {
        const [x, y] = e.p
        ellipse(x + 3, y + 4, e.rx, e.ry, { fill: "#0f1209", opacity: 0.6, blur: 3, rotate: e.rot })
        ellipse(x, y, e.rx, e.ry, {
          fill: radial(x - e.rx * 0.3, y - e.ry * 0.35, e.ry * 1.3, [[0, "#a9c27a"], [0.45, P.egg], [1, P.eggD]]),
          opacity: 0.62, rotate: e.rot,
        })
        // larva dentro, intuida, con brillo verdoso que atraviesa la cáscara
        ellipse(x + 2, y + 3, e.rx * 0.45, e.ry * 0.55, { fill: "#1f2a14", opacity: 0.55, blur: 3, rotate: e.rot })
        ellipse(x + 1, y + 5, e.rx * 0.55, e.ry * 0.4, { fill: P.lime, opacity: 0.18, blur: 6 })
        ellipse(x - e.rx * 0.35, y - e.ry * 0.45, e.rx * 0.2, e.ry * 0.12, { fill: "#cfe0a6", opacity: 0.45, blur: 1, rotate: e.rot })
      }

      // TEXTURA DE ROCA: motas y vetas
      for (let i = 0; i < 260; i++) {
        const a = rand(-1.45, 1.45), b = rand(-1.45, 1.45)
        if (Math.hypot(a, b) > 1.5 || hiveF(a, b) > -0.3) continue
        const p = sp(a, b)
        circle(p[0], p[1], rand(1.5, 5), { fill: rand() < 0.5 ? P.rockD : P.rockL, opacity: rand(0.2, 0.5), blur: 1 })
      }

      // SOMBRA Y LUZ GLOBAL
      rect(0, 0, 1024, 1024, {
        fill: radial(330, 300, 760, [[0.3, alpha(P.shadow, 0)], [0.62, alpha(P.shadow, 0.45)], [0.85, alpha(P.shadow, 0.82)], [1, alpha(P.shadow, 0.92)]]),
      })
      ellipse(330, 300, 300, 260, { fill: P.warm, opacity: 0.16, blur: 90, blend: "soft-light" })
      // contraluz fría muy tenue abajo a la derecha
      outline([rockF], -14, P.cold, { opacity: 0.22, blur: 10 })
      rect(0, 0, 1024, 1024, { fill: radial(330, 300, 520, [[0.6, alpha("#000000", 0)], [1, alpha("#000000", 0)]]) })

      // BRILLO DE CELDAS ABIERTAS (emiten luz propia, por eso van encima de la sombra)
      for (const c of cells.filter(c => c.open)) {
        const p = sp(c.a, c.b), r = rh * R * c.k * 0.6
        const col = rand() < 0.3 ? P.amber : P.lime
        const dim = 0.65 + rand(0, 0.3)
        circle(p[0], p[1], r * 2.2, { fill: col, opacity: 0.12 * dim, blur: 14 })
        blob(hexPts(c, c.k * 0.62), { fill: radial(p[0] + 2, p[1] + 3, r * 1.2, [[0, col], [0.5, mix(col, "#2a3010", 0.45)], [1, "#121506"]]), opacity: dim })
        circle(p[0] + 1, p[1] + 2, r * 0.3, { fill: mix(col, "#ffffff", 0.25), opacity: 0.6 * dim, blur: 2 })
      }

      // GRANO
      grain({ opacity: 0.08, size: 1.2 })
    })
  })
})
