// dibujo.js: brújula antigua de latón con rosa de los vientos, con esqueleto para animar la aguja
canvas({ width: 1024, height: 1024, background: null })

const P = {
  brassDark: "#5a2c10",
  brass: "#b8692a",
  brassMid: "#d58a3c",
  brassLight: "#f3c27a",
  cream: "#f3ecc4",
  creamShade: "#d8cf9a",
  ink: "#2e1a10",
  needle: "#3a1420",
  gold: "#e0a630",
  bottle: "#0f3b24", // verde botella de la aguja
  bottleLight: "#2f7a4c",
  cold: "#9fb6d6",
}
const C = [508, 492] // centro de la esfera
setLight(-1, -1)

// Esqueleto: la caja, la esfera, la rosa, el pivote y el cristal van fijos en «brujula»; la aguja
// gira sobre el pivote. Su sombra va en su propio hueso, desplazado lo mismo que la sombra (luz de
// arriba a la izquierda): gira igual que la aguja pero no se mueve con ella alrededor del pivote.
const SOMBRA = [10, 14]
bone("brujula", { at: C })
bone("sombra", { parent: "brujula", at: [C[0] + SOMBRA[0], C[1] + SOMBRA[1]] })
bone("aguja", { parent: "brujula", at: C })
rig({ origin: C })

// Caja de latón: aro exterior con volumen
part("brujula", () => layer("caja", () => {
  // sombra de apoyo
  ellipse(C[0] + 14, C[1] + 22, 452, 458, { fill: "#2a1408", opacity: 0.35, blur: 18 })
  const outer = circle(C[0], C[1], 450, { fill: P.brass })
  freeform(outer, [
    [170, 140, P.brassLight, 0.3],
    [120, 520, P.brassMid, 0.2],
    [880, 300, P.brass, 0.2],
    [880, 820, P.brassDark, 0.3],
    [500, 950, "#8a4518", 0.2],
    [300, 860, P.brassMid, 0.1],
  ])
  // reflejo frío de contraluz en el borde inferior derecho
  clip(circle(C[0], C[1], 450, { hidden: true }), () => {
    curve(arc(C[0], C[1], 436, 100, 170, 20), { stroke: P.cold, width: 14, opacity: 0.3, blur: 8 })
    curve(arc(C[0], C[1], 438, 290, 350, 20), { stroke: "#fff3d0", width: 12, opacity: 0.55, blur: 5 })
  })
  // escalones del bisel
  circle(C[0], C[1], 418, { fill: "none", stroke: P.brassDark, width: 4, opacity: 0.5 })
  const bez = circle(C[0], C[1], 405, { fill: P.brassMid })
  freeform(bez, [
    [880, 820, P.brassLight, 0.2],
    [200, 200, "#7a3a12", 0.2],
    [140, 560, P.brass, 0.1],
    [820, 200, P.brass, 0.1],
    [500, 900, P.brassLight, 0.1],
  ])
  circle(C[0], C[1], 382, { fill: P.brassDark })
}))

// Esfera crema
part("brujula", () => layer("esfera", () => {
  const face = circle(C[0], C[1], 376, { fill: P.cream })
  freeform(face, [
    [500, 500, "#fbf6dc", 0.4],
    [250, 220, P.creamShade, 0.1],
    [800, 800, "#efe6b4", 0.2],
    [180, 700, "#e6dca8", 0.1],
  ])
  // sombra del bisel sobre la esfera (luz desde arriba izquierda)
  clip(circle(C[0], C[1], 376, { hidden: true }), () => {
    circle(C[0] + 22, C[1] + 26, 376, { fill: "none", stroke: "#6b4a20", width: 40, opacity: 0.35, blur: 14, blend: "multiply" })
  })
  // anillos de la escala
  for (const [r, w] of [[360, 2], [352, 1.2], [318, 2.2], [312, 1], [262, 2], [256, 1]])
    circle(C[0], C[1], r, { fill: "none", stroke: P.ink, width: w, opacity: 0.8 })
  // marcas de grados
  for (let d = 0; d < 360; d += 2) {
    const long = d % 10 === 0
    const a = polar(C[0], C[1], 352, d), b = polar(C[0], C[1], long ? 338 : 345, d)
    line(a[0], a[1], b[0], b[1], { stroke: P.ink, width: long ? 1.8 : 1, opacity: 0.8 })
  }
  // números
  for (let d = 0; d < 360; d += 20) {
    const p = polar(C[0], C[1], 327, d - 90)
    group({ rotate: d, around: p }, () => {
      text(p[0], p[1] + 8, String(d), { size: 24, anchor: "middle", font: "Georgia, serif", fill: P.ink })
    })
  }
  // letras de rumbos
  const names = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
  names.forEach((n, i) => {
    const d = i * 45
    const p = polar(C[0], C[1], 287, d - 90)
    group({ rotate: d, around: p }, () => {
      text(p[0], p[1] + 12, n, { size: n.length > 1 ? 30 : 40, weight: "bold", anchor: "middle", font: "Georgia, serif", fill: P.ink, spacing: 1 })
    })
  })
}))

// Rosa de los vientos
part("brujula", () => layer("rosa", () => {
  // radios finos
  for (let d = 0; d < 360; d += 11.25) {
    const a = polar(C[0], C[1], 60, d), b = polar(C[0], C[1], 256, d)
    line(a[0], a[1], b[0], b[1], { stroke: P.ink, width: 1, opacity: 0.55 })
  }
  circle(C[0], C[1], 130, { fill: "none", stroke: P.ink, width: 1.6, opacity: 0.7 })
  circle(C[0], C[1], 124, { fill: "none", stroke: P.ink, width: 0.8, opacity: 0.6 })
  // puntas secundarias (16), claras con nervio oscuro
  const point = (d, len, w, light, dark) => {
    const tip = polar(C[0], C[1], len, d)
    const l = polar(C[0], C[1], w, d - 45), r = polar(C[0], C[1], w, d + 45)
    poly([tip, l, C], { fill: light, stroke: P.ink, width: 1 })
    poly([tip, C, r], { fill: dark, stroke: P.ink, width: 1 })
  }
  for (let i = 0; i < 16; i++) point(11.25 + i * 22.5, 225, 24, "#efe4b8", "#b7a46a")
  // puntas intercardinales y cardinales, ornamentadas
  const orn = (d, len, w) => {
    const tip = polar(C[0], C[1], len, d)
    const l = polar(C[0], C[1], w, d - 45), r = polar(C[0], C[1], w, d + 45)
    const s = poly([tip, l, C, r], { fill: P.needle, stroke: P.ink, width: 1.5 })
    clip(s, () => {
      // hojitas crema a lo largo de la punta
      for (let k = 0.12; k < 0.92; k += 0.075) {
        const q = polar(C[0], C[1], len * k, d)
        const side = (1 - k) * w * 0.55
        for (const sg of [-1, 1]) {
          const o = polar(q[0], q[1], side * 0.6, d + 90 * sg)
          ellipse(o[0], o[1], 5 + (1 - k) * 6, 3 + (1 - k) * 3, { fill: "#ead9a4", rotate: -d + 90 + sg * 35, opacity: 0.9 })
        }
      }
      // mitad en sombra
      poly([tip, C, r], { fill: "#1a0810", opacity: 0.3, blend: "multiply" })
    })
  }
  for (let i = 0; i < 4; i++) orn(45 + i * 90, 245, 36)
  for (let i = 0; i < 4; i++) orn(i * 90, 262, 40)
}))

// Aguja
// ángulo de la aguja en el dibujo (de abajo-derecha a arriba-izquierda): las acciones la giran desde aquí
const dn = -134, len = 395
const shape = (d, o = [0, 0]) => {
  const c = [C[0] + o[0], C[1] + o[1]]
  const t = polar(c[0], c[1], len, d)
  const s1 = polar(c[0], c[1], 230, d), s2 = polar(c[0], c[1], 140, d)
  return [
    polar(t[0], t[1], 7, d + 90),
    polar(s1[0], s1[1], 18, d + 90), polar(s2[0], s2[1], 38, d + 90),
    polar(c[0], c[1], 24, d + 90),
    polar(c[0], c[1], 24, d - 90),
    polar(s2[0], s2[1], 38, d - 90), polar(s1[0], s1[1], 18, d - 90),
    polar(t[0], t[1], 7, d - 90),
  ]
}
part("sombra", () => layer("sombra_aguja", () => {
  poly(shape(dn, SOMBRA), { fill: "#3a2a10", opacity: 0.53, blur: 10 })
  poly(shape(dn + 180, SOMBRA), { fill: "#3a2a10", opacity: 0.53, blur: 10 })
}))

part("aguja", () => layer("aguja", () => {
  poly(shape(dn), { fill: P.bottle })
  poly(shape(dn + 180), { fill: P.bottle })
  // luz sobre el filo superior
  for (const d of [dn, dn + 180]) {
    const t = polar(C[0], C[1], len, d), m = polar(C[0], C[1], 140, d)
    const ml = polar(m[0], m[1], 30, d - 90)
    poly([t, ml, polar(C[0], C[1], 20, d - 90), C], { fill: P.bottleLight, opacity: 0.55, blend: "screen" })
  }
  // letras N y S
  const pN = polar(C[0], C[1], 180, dn), pS = polar(C[0], C[1], 180, dn + 180)
  group({ rotate: dn + 180, around: pN }, () =>
    text(pN[0], pN[1] + 14, "N", { size: 40, weight: "bold", anchor: "middle", font: "Georgia, serif", fill: P.gold }))
  group({ rotate: dn + 180, around: pS }, () =>
    text(pS[0], pS[1] + 14, "S", { size: 40, weight: "bold", anchor: "middle", font: "Georgia, serif", fill: P.gold }))
}))

// Pivote central dorado
part("brujula", () => layer("pivote", () => {
  circle(C[0] + 4, C[1] + 6, 46, { fill: "#1a0a04", opacity: 0.5, blur: 6 })
  const cap = circle(C[0], C[1], 44, { fill: P.gold })
  freeform(cap, [
    [C[0] - 22, C[1] - 24, "#fff0b0", 0.2],
    [C[0], C[1], "#e6a92c", 0.2],
    [C[0] + 30, C[1] + 30, "#8a5210", 0.2],
  ])
  circle(C[0], C[1], 16, { fill: radial(C[0] - 4, C[1] - 4, 18, ["#b88a2a", "#6a4210"]) })
  circle(C[0], C[1], 44, { fill: "none", stroke: "#6a3a08", width: 2 })
  ellipse(C[0] - 16, C[1] - 20, 12, 6, { fill: "#ffffff", opacity: 0.8, rotate: -35, blur: 1 })
}))

// Cristal: reflejos
part("brujula", () => layer("cristal", () => {
  clip(circle(C[0], C[1], 376, { hidden: true }), () => {
    blob([[200, 260], [330, 150], [480, 120], [360, 230], [250, 360]], { fill: "#ffffff", opacity: 0.18, blur: 12 })
    brush(arc(C[0], C[1], 360, 300, 340, 10), { width: 10, color: "#ffffff", taper: [0.4, 0.4] })
    brush(arc(C[0], C[1], 355, 120, 160, 10), { width: 6, color: P.cold, taper: [0.4, 0.4] })
  })
  grain({ opacity: 0.06 })
}))

// ---------- acciones ----------
// La aguja gira sobre el pivote (grados, positivo = sentido horario) desde donde la deja el dibujo;
// su sombra gira lo mismo. Las claves se unen con curvas suaves, como en Godot.
const ag = d => ({ aguja: d, sombra: d })
// reposo: una aguja imantada nunca está quieta del todo; tiembla un poco, sin ritmo fijo
action("reposo", { duration: 4, loop: true }, [
  [0, ag(0)], [0.17, ag(1.4)], [0.36, ag(-0.9)], [0.55, ag(0.7)], [0.78, ag(-1.2)], [1, ag(0)],
])
// buscar el norte: al mover la brújula la aguja se queda 80° fuera y vuelve oscilando cada vez menos
// (una oscilación amortiguada: quieta un instante en cada extremo y más rápida al pasar por su sitio,
// unos 0,7 s por ida y vuelta) hasta quedarse quieta
const AMORTIGUADA = [80, -52, 33, -21, 13, -8, 4.5, -2.2, 0.8]
action("buscar_norte", { duration: 3.4, loop: false }, [
  ...AMORTIGUADA.flatMap((a, i) => [[i * 0.105, ag(a)], [i * 0.105 + 0.0525, ag(0)]]),
  [1, ag(0)],
])
// imán: con un imán cerca la aguja se vuelve loca: va y viene a tirones y casi da la vuelta
action("iman", { duration: 2.6, loop: true }, [
  [0, ag(0)], [0.12, ag(70)], [0.2, ag(40)], [0.34, ag(160)], [0.44, ag(120)],
  [0.58, ag(-60)], [0.7, ag(-20)], [0.82, ag(-110)], [0.92, ag(-40)], [1, ag(0)],
])
