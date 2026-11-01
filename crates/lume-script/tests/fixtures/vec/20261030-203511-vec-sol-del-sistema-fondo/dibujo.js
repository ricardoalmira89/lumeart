// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const CX = 512, CY = 512, R = 400
const P = {
  core: "#ffcf78",   // centro cálido (contenido, no blanco)
  mid: "#ff962c",    // naranja medio
  deep: "#e8581a",   // naranja rojizo
  limb: "#a8240f",   // rojo del limbo
  lane: "#c23d12",   // surcos entre células
  spot: "#3a0d0a",   // umbra
  pen: "#8a2c12",    // penumbra
  hive: "#4a6420",   // veta de la Colmena
  hiveDark: "#1e2a12",
  glow: "#ff7a2a",
}
seed(7)
const D = Math.PI / 180
// punto en el borde: ángulo en grados, 0 = arriba, sentido horario; h = altura sobre el limbo
const L = (a, h = 0) => [CX + (R + h) * Math.sin(a * D), CY - (R + h) * Math.cos(a * D)]
const INF = 228 // ángulo de la zona infectada (abajo a la izquierda)

// proyección de un punto (lat, lon en grados) sobre la esfera
function sp(lat, lon) {
  return { x: CX + R * Math.cos(lat * D) * Math.sin(lon * D), y: CY - R * Math.sin(lat * D), z: Math.cos(lat * D) * Math.cos(lon * D) }
}

layer("halo", () => {
  // brillo exterior que se desvanece a transparencia
  // anillos concéntricos muy tenues: la opacidad se acumula hacia el limbo
  const N = 48
  for (let i = 0; i < N; i++) {
    const t = i / (N - 1)
    const r = R + 100 * Math.pow(1 - t, 1.4)
    circle(CX, CY, r, { fill: mix(P.limb, P.glow, t), opacity: 0.022 })
  }
})

element("sol", { pivot: [CX, CY] }, () => {
  const disc = circle(CX, CY, R, { fill: radial(CX - 40, CY - 50, R * 1.05, [[0, P.core], [0.45, P.mid], [0.8, P.deep], [1, P.limb]]) })

  clip(disc, () => {
    // surcos oscuros base y células de convección encima, aplastadas hacia el limbo
    circle(CX, CY, R, { fill: P.lane, opacity: 0.18, blend: "multiply" })
    for (let lat = -86; lat <= 86; lat += 3.6) {
      for (let lon = -86; lon <= 86; lon += 3.6) {
        const p = sp(lat + rand(-1.4, 1.4), lon + rand(-1.4, 1.4))
        if (p.z < 0.04) continue
        const ang = Math.atan2(p.y - CY, p.x - CX) / D
        const s = rand(8, 11.5)
        const t = 1 - p.z
        const c = mix(mix(P.core, P.mid, rand(0, 0.6)), P.limb, t * t * 0.9)
        blob(jitter(ring(p.x, p.y, s * Math.max(p.z, 0.16), s, 6, rand(0, 60)), s * 0.15), { fill: c, rotate: ang, opacity: lerp(0.3, 0.1, t) * rand(0.6, 1), blur: 1.8, blend: "screen" })
      }
    }
    // supergranulación: manchas amplias de luz y sombra
    for (let i = 0; i < 16; i++) {
      const p = sp(rand(-65, 65), rand(-65, 65))
      circle(p.x, p.y, rand(50, 120), { fill: rand() < 0.5 ? P.core : P.deep, opacity: 0.2, blur: 40, blend: rand() < 0.5 ? "soft-light" : "multiply" })
    }
    // oscurecimiento del limbo
    circle(CX, CY, R, { fill: radial(CX, CY, R, [[0.55, alpha(P.limb, 0)], [0.9, alpha(P.limb, 0.3)], [1, alpha(P.limb, 0.5)]]), blend: "multiply" })

    // manchas solares (umbra, penumbra con filamentos cortos)
    layer("manchas", () => {
      const spots = [[24, 26, 26], [19, 34, 11], [-18, -30, 19]]
      for (const [la, lo, s] of spots) {
        const p = sp(la, lo)
        const ang = Math.atan2(p.y - CY, p.x - CX) / D
        const k = p.z
        group({ rotate: ang, around: [p.x, p.y] }, () => {
          ellipse(p.x, p.y, s * 2.2 * k, s * 2.2, { fill: P.deep, opacity: 0.45, blur: 12, blend: "multiply" })
          blob(jitter(ring(p.x, p.y, s * 1.25 * k, s * 1.25, 9), s * 0.12), { fill: P.pen, opacity: 0.75, blur: 3 })
          for (let i = 0; i < 34; i++) {
            const a = (i / 34) * Math.PI * 2 + rand(-0.08, 0.08)
            const r1 = s * 0.6, r2 = s * rand(1.05, 1.3)
            brush([[p.x + Math.cos(a) * r1 * k, p.y + Math.sin(a) * r1], [p.x + Math.cos(a) * r2 * k, p.y + Math.sin(a) * r2]],
              { width: s * 0.09, color: pick([P.deep, P.lane, "#b8481a"]), taper: [0.2, 0.7], opacity: 0.55 })
          }
          blob(jitter(ring(p.x, p.y, s * 0.6 * k, s * 0.6, 8), s * 0.07), { fill: P.spot, blur: 2 })
          ellipse(p.x + s * 0.1 * k, p.y - s * 0.1, s * 0.25 * k, s * 0.22, { fill: "#6a1a0c", opacity: 0.6, blur: 3 })
        })
      }
    })

    // infección de la Colmena: vetas verdosas ramificadas desde el borde inferior izquierdo
    layer("colmena", () => {
      const base = L(INF, -20)
      circle(base[0], base[1], 190, { fill: P.hiveDark, opacity: 0.3, blur: 70, blend: "multiply" })
      circle(base[0], base[1], 150, { fill: P.hive, opacity: 0.3, blur: 55, blend: "color" })
      function vein(x, y, dir, len, w, depth) {
        const pts = [[x, y]]
        let a = dir
        const n = 7
        const bend = rand(-9, 9)
        for (let i = 1; i <= n; i++) {
          a += bend + rand(-12, 12)
          const prev = pts[pts.length - 1]
          pts.push([prev[0] + Math.sin(a * D) * len / n, prev[1] - Math.cos(a * D) * len / n])
        }
        brush(pts, { width: w * 3.5, color: P.hive, taper: [0.1, 0.8], opacity: 0.22, blur: 5 })
        brush(pts, { width: w * 1.2, color: P.hiveDark, taper: [0, 0.9], opacity: 0.5, blur: 1.2 })
        brush(pts, { width: w * 0.35, color: "#2a3a14", taper: [0.2, 0.9], opacity: 0.5 })
        if (depth > 0) {
          const q = pts[3 + Math.floor(rand(0, 3))]
          vein(q[0], q[1], a + pick([-1, 1]) * rand(25, 45), len * 0.55, w * 0.6, depth - 1)
        }
      }
      // panal muy tenue: celdas hexagonales que se pierden hacia el centro
      const hs = 20
      for (let row = -12; row <= 12; row++) {
        for (let col = -12; col <= 12; col++) {
          const hx = base[0] + col * hs * 1.5, hy = base[1] + row * hs * 1.732 + (col % 2 ? hs * 0.866 : 0)
          const d = Math.hypot(hx - base[0], hy - base[1])
          if (d > 190 || rand() < 0.25) continue
          blob(jitter(ring(hx, hy, hs * 0.9, hs * 0.9, 6, 30), 2.5).map(q => [q[0], q[1], 0.5]), { fill: "none", stroke: P.hiveDark, width: rand(1.5, 2.5), opacity: 0.3 * (1 - d / 190), blur: 1, blend: "multiply" })
        }
      }
      for (let i = 0; i < 9; i++) {
        const ang = INF + lerp(-24, 24, i / 8) + rand(-3, 3)
        const st = L(ang, 10)
        vein(st[0], st[1], ang + 180 + rand(-15, 15), rand(90, 170) * (1 - Math.abs(i - 4) / 8), rand(3.5, 6), 2)
      }
    })
  })

  // borde suave: banda difuminada que funde el limbo con el halo
  circle(CX, CY, R - 2, { fill: "none", stroke: P.deep, width: 16, blur: 10, opacity: 0.6 })
  circle(CX, CY, R + 3, { fill: "none", stroke: P.deep, width: 8, blur: 5, opacity: 0.55 })
  clip(circle(CX, CY, R, { hidden: true }), () => grain({ opacity: 0.07, size: 1.2, blend: "overlay" }))
})

// protuberancias y arcos de plasma
layer("protuberancias", () => {
  // arco de plasma: varias hebras difusas que salen y vuelven al limbo
  function loop(a1, a2, h, w) {
    const mid = (a1 + a2) / 2
    const pts = [L(a1, -8), L(a1 + (mid - a1) * 0.1, h * 0.6), L(mid, h), L(a2 - (a2 - mid) * 0.1, h * 0.6), L(a2, -8)]
    // velo de plasma bajo el arco
    const veil = [L(a1, -10), L(a1 + (mid - a1) * 0.1, h * 0.55), L(mid, h * 0.9), L(a2 - (a2 - mid) * 0.1, h * 0.55), L(a2, -10)]
    blob(veil, { fill: P.deep, opacity: 0.28, blur: 6 })
    const fat = (k) =>(u) => w * k * (0.75 + 0.55 * Math.sin(Math.PI * u))
    brush(pts, { width: fat(2.6), color: P.deep, taper: [0, 0], opacity: 0.3, blur: 7 })
    brush(pts, { width: fat(1.5), color: P.deep, taper: [0, 0], opacity: 0.75, blur: 3 })
    brush(pts, { width: fat(0.9), color: P.glow, taper: [0.05, 0.05], opacity: 0.7, blur: 2.5 })
    for (let i = 0; i < 3; i++) {
      const q = pts.map((p, j) => j === 0 || j === 4 ? p : [p[0] + rand(-w * 0.3, w * 0.3), p[1] + rand(-w * 0.3, w * 0.3)])
      brush(q, { width: fat(rand(0.2, 0.35)), color: pick([P.mid, P.core]), taper: [0.2, 0.2], opacity: 0.35, blur: 1.5 })
    }
  }
  // grupo de arcos arriba a la derecha (lado visible principal)
  loop(30, 47, 60, 15)
  loop(52, 60, 30, 10)
  // arco a la derecha
  loop(98, 111, 40, 12)
  // arco pequeño arriba a la izquierda
  loop(316, 326, 28, 9)
  // llamarada: penacho inclinado que se deshilacha arriba
  const pl = [L(4, -10), L(3, 30), L(8, 66), L(14, 84), L(13, 60), L(12, 30), L(13, -10)]
  blob(pl, { fill: P.deep, opacity: 0.35, blur: 9 })
  blob(grow(pl, 0.8), { fill: P.deep, opacity: 0.7, blur: 4 })
  for (let i = 0; i < 4; i++) {
    const f = [L(8 + rand(-3, 2), -6), L(7 + rand(-2, 2), 30), L(10 + rand(-1, 2), 58), L(13, 76 + rand(-6, 6))]
    brush(f, { width: [10, 7, 4, 1], color: pick([P.glow, P.mid]), opacity: 0.5, blur: 2, taper: [0, 0.7] })
  }
  // espículas cortas a lo largo del limbo (menos en la zona infectada)
  for (let i = 0; i < 90; i++) {
    const a = rand(0, 360)
    if (Math.abs(a - INF) < 32) continue
    const h = rand(6, 18)
    brush([L(a, -3), L(a + rand(-1.5, 1.5), h)], { width: rand(2, 4), color: P.glow, opacity: 0.35, blur: 2, taper: [0, 1] })
  }
})
