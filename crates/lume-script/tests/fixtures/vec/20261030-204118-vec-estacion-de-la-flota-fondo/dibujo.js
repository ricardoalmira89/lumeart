// dibujo.js: estación orbital de la flota, asediada por la Colmena (vista cenital, fondo lejano)
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

// Plan: anillo r 318–392 con 12 módulos, 4 radios (0/90/180/270°) a un cubo central r 112,
// paneles solares en las diagonales 45° y 225°. Costra de la Colmena en el lado derecho.
// Luz arriba-izquierda, cálida y contenida; contraluz fría abajo-derecha. Poco contraste.
const P = {
  deep: "#4a5768",    // sombras del casco
  dark: "#617084",    // casco en sombra
  hull: "#7b8a9d",    // casco medio
  light: "#96a4b5",   // casco iluminado
  stripe: "#adb6c1",  // franjas "blancas" apagadas
  panel: "#24365a",   // paneles solares
  panelL: "#344d78",
  win: "#e6dba6",     // ventanas
  burn: "#2e3138",    // placas ennegrecidas
  hive: "#1e2c1f",    // costra de la Colmena
  hiveM: "#2b4027",
  hiveL: "#4b6a36",
  cold: "#7f9cc4",    // contraluz fría
}
const CX = 512, CY = 512
const R0 = 318, R1 = 392
// punto en coordenadas polares (a en grados, 0 = derecha, sentido horario)
const pt = (r, a) => [CX + r * Math.cos(a * Math.PI / 180), CY + r * Math.sin(a * Math.PI / 180)]
// punto con desplazamiento tangencial t (en unidades) respecto al radio a
const pt2 = (r, a, t) => {
  const c = Math.cos(a * Math.PI / 180), s = Math.sin(a * Math.PI / 180)
  return [CX + r * c - t * s, CY + r * s + t * c]
}
// cuadrilátero alineado al radio a: de r0 a r1, de t0 a t1 a lo ancho
const quad = (a, r0, r1, t0, t1) => [pt2(r0, a, t0), pt2(r1, a, t0), pt2(r1, a, t1), pt2(r0, a, t1)]
// corona circular (exterior horario, interior antihorario: queda hueco)
const annulus = (ri, ro) =>
  `M ${CX + ro} ${CY} A ${ro} ${ro} 0 1 1 ${CX - ro} ${CY} A ${ro} ${ro} 0 1 1 ${CX + ro} ${CY} Z ` +
  `M ${CX + ri} ${CY} A ${ri} ${ri} 0 1 0 ${CX - ri} ${CY} A ${ri} ${ri} 0 1 0 ${CX + ri} ${CY} Z`
const arcPts =(r, a0, a1, n) => range(n + 1).map(i => pt(r, lerp(a0, a1, i / n)))

const SPOKES = [0, 90, 180, 270]
const PANELS = [45, 225]
const MODS = range(12).map(i => i * 30 + 15)
const DAMAGED = [315, 45]

element("estacion", { pivot: [CX, CY] }, () => {

  layer("paneles", () => {
    // mástiles y alas de paneles solares en las diagonales
    for (const a of PANELS) {
      // mástil
      poly(quad(a, R1 - 10, 598, -9, 9), { fill: P.dark })
      poly(quad(a, R1 - 10, 598, -9, -2), { fill: P.light, opacity: 0.35 })
      // alas: 4 pares de paneles
      for (let k = 0; k < 4; k++) {
        const r0 = 412 + k * 46, r1 = r0 + 40
        for (const side of [-1, 1]) {
          const t0 = side * 14, t1 = side * 98
          const q = quad(a, r0, r1, Math.min(t0, t1), Math.max(t0, t1))
          const cq = center(q)
          const s = poly(q, { fill: linear(cq[0] - 60, cq[1] - 60, cq[0] + 60, cq[1] + 60, [P.panelL, P.panel]) })
          clip(s, () => {
            // rejilla de celdas
            for (let c = 1; c < 6; c++) {
              const tt = lerp(t0, t1, c / 6)
              line(...pt2(r0, a, tt), ...pt2(r1, a, tt), { stroke: "#1a2744", width: 1.6, opacity: 0.8 })
            }
            line(...pt2((r0 + r1) / 2, a, t0), ...pt2((r0 + r1) / 2, a, t1), { stroke: "#1a2744", width: 1.6, opacity: 0.8 })
            // reflejo suave de la luz
            circle(...pt2(r0, a, side * 30), 70, { fill: "#6f8fc0", opacity: 0.12, blur: 20 })
          })
          // bastidor
          poly(q, { stroke: shade(P.dark, -0.2), width: 2.5, fill: "none", opacity: 0.8 })
        }
        // travesaño
        poly(quad(a, r0 + 14, r0 + 26, -16, 16), { fill: P.hull })
      }
      // remate del mástil
      circle(...pt(600, a), 11, { fill: P.hull })
      circle(...pt(600, a), 11, { fill: P.deep, opacity: 0.5, blend: "multiply" })
    }
  })

  layer("anillo", () => {
    // banda del anillo con volumen (toro visto desde arriba: cresta clara al centro)
    const k0 = R0 / R1
    const ring = path(annulus(R0, R1), {
      fill: radial(CX, CY, R1, [[k0 - 0.01, P.deep], [k0 + 0.03, P.dark], [0.9, P.hull], [0.95, P.light], [0.985, P.hull], [1, P.dark]]),
    })
    clip(ring, () => {
      // luz arriba-izquierda y sombra abajo-derecha
      rect(0, 0, 1024, 1024, { fill: linear(200, 200, 830, 830, ["#e6dccb", "#808080", "#2a3350"]), blend: "soft-light", opacity: 0.45 })
      // contraluz fría en el borde exterior de abajo a la derecha
      circle(CX + 14, CY + 14, R1, { fill: "none", stroke: P.cold, width: 10, opacity: 0.35, blur: 6 })
      // juntas de segmentos
      for (let i = 0; i < 48; i++) {
        const a = i * 7.5
        line(...pt(R0, a), ...pt(R1, a), { stroke: P.deep, width: 1.5, opacity: 0.35 })
      }
      // carriles concéntricos
      circle(CX, CY, 336, { fill: "none", stroke: P.deep, width: 2, opacity: 0.35 })
      circle(CX, CY, 378, { fill: "none", stroke: P.deep, width: 2, opacity: 0.3 })
      // placas con tono variado
      seed(4)
      for (let i = 0; i < 40; i++) {
        const a = rand(0, 360), r = rand(340, 376)
        poly(quad(a, r - 8, r + 8, -12, 12), { fill: pick([P.light, P.dark, P.hull]), opacity: 0.25 })
      }
      // franjas blancas apagadas
      for (let i = 0; i < 12; i++) {
        const a0 = i * 30 + 2
        curve(arcPts(358, a0 - 6, a0 + 6, 6), { stroke: P.stripe, width: 6, opacity: 0.55 })
      }
      grain({ opacity: 0.06 })
    })
  })

  layer("radios", () => {
    // cuatro radios del anillo al cubo
    for (const a of SPOKES) {
      const s = poly(quad(a, 100, R0 + 8, -20, 20), { fill: P.hull })
      clip(s, () => {
        poly(quad(a, 100, R0 + 8, -20, -8), { fill: P.light, opacity: 0.5, blur: 4 })
        poly(quad(a, 100, R0 + 8, 8, 20), { fill: P.deep, opacity: 0.45, blur: 4, blend: "multiply" })
        for (let k = 0; k < 7; k++) {
          const r = 130 + k * 28
          line(...pt2(r, a, -20), ...pt2(r, a, 20), { stroke: P.deep, width: 2, opacity: 0.4 })
        }
        poly(quad(a, 110, R0, -3, 3), { fill: P.stripe, opacity: 0.35 })
      })
      // sombra proyectada sobre el anillo interior
      poly(quad(a, R0 - 4, R0 + 14, -26, 26), { fill: P.deep, opacity: 0.3, blur: 6, blend: "multiply" })
      // collar de unión con el anillo
      poly(quad(a, R0 - 14, R0 + 6, -28, 28), { fill: P.dark })
      poly(quad(a, R0 - 14, R0 - 6, -28, 28), { fill: P.light, opacity: 0.4 })
    }
  })

  layer("modulos", () => {
    // módulos sobre el anillo
    seed(11)
    for (const a of MODS) {
      const long = rand() < 0.5
      const tw = long ? 46 : 36
      // sombra proyectada abajo-derecha
      const m = poly(quad(a, R0 - 8, R1 + 10, -tw, tw), { fill: P.hull })
      const cm = center(quad(a, R0 - 8, R1 + 10, -tw, tw))
      clip(m, () => {
        rect(cm[0] - 70, cm[1] - 70, 140, 140, { fill: linear(cm[0] - 50, cm[1] - 50, cm[0] + 50, cm[1] + 50, [P.light, P.hull, P.dark]) })
        // techo elevado
        poly(quad(a, R0 + 6, R1 - 4, -tw + 10, tw - 10), { fill: P.light, opacity: 0.35 })
        poly(quad(a, R0 + 6, R1 - 4, -tw + 10, tw - 10), { fill: "none", stroke: P.deep, width: 2, opacity: 0.35 })
        // franja blanca
        poly(quad(a, R1 - 2, R1 + 4, -tw, tw), { fill: P.stripe, opacity: 0.5 })
        // rejillas
        for (let k = 0; k < 4; k++) {
          const t = -tw + 16 + k * ((2 * tw - 32) / 3)
          line(...pt2(R0 + 14, a, t), ...pt2(R0 + 30, a, t), { stroke: P.deep, width: 2, opacity: 0.4 })
        }
        // contraluz fría
        poly(shift(quad(a, R0 - 8, R1 + 10, -tw, tw), -5, -5), { fill: "none", stroke: P.cold, width: 6, opacity: 0.25, blur: 3 })
      })
      // radiador lateral pequeño
      if (!long) {
        poly(quad(a, R1 + 10, R1 + 26, -14, 14), { fill: P.dark })
        poly(quad(a, R1 + 10, R1 + 26, -14, -4), { fill: P.light, opacity: 0.3 })
      }
    }
  })

  layer("cubo", () => {
    // muelles en diagonal con naves atracadas
    for (const a of [45, 135, 225, 315]) {
      poly(quad(a, 90, 196, -12, 12), { fill: P.dark })
      poly(quad(a, 90, 196, -12, -4), { fill: P.light, opacity: 0.35 })
      // plataforma del muelle
      const pad = poly(quad(a, 170, 212, -26, 26), { fill: P.hull })
      clip(pad, () => {
        poly(quad(a, 170, 212, -26, -10), { fill: P.light, opacity: 0.35 })
        poly(quad(a, 170, 212, 12, 26), { fill: P.deep, opacity: 0.35, blend: "multiply" })
      })
      poly(quad(a, 176, 206, -18, 18), { fill: "none", stroke: P.stripe, width: 2, opacity: 0.35, dash: "5 4" })
    }
    // nave pequeña atracada en el muelle 135
    {
      const b = pt(192, 135)
      blob([[b[0] - 6, b[1] - 20, 0], [b[0] + 10, b[1] + 2], [b[0] + 4, b[1] + 14], [b[0] - 12, b[1] + 8]], { fill: P.light, opacity: 0.85 })
    }
    // cuerpo del cubo
    const hub = circle(CX, CY, 112, { fill: P.hull })
    freeform(hub, [
      [CX - 55, CY - 60, P.stripe, 0.2],
      [CX - 10, CY - 15, P.light, 0.3],
      [CX + 40, CY + 40, P.hull, 0.3],
      [CX + 85, CY + 70, P.deep, 0.3],
      [CX - 90, CY + 50, P.dark, 0.2],
    ])
    const hubC = circle(CX, CY, 112, { hidden: true })
    clip(hubC, () => {
      // anillos de placas
      circle(CX, CY, 86, { fill: "none", stroke: P.deep, width: 3, opacity: 0.35 })
      circle(CX, CY, 100, { fill: "none", stroke: P.stripe, width: 4, opacity: 0.3, dash: "26 10" })
      for (let i = 0; i < 16; i++) {
        const a = i * 22.5 + 11
        line(...pt(86, a), ...pt(112, a), { stroke: P.deep, width: 2, opacity: 0.3 })
      }
      circle(CX + 108, CY + 108, 120, { fill: "none", stroke: P.cold, width: 14, opacity: 0.0 })
      grain({ opacity: 0.07 })
    })
    // borde de contraluz fría abajo-derecha
    curve(arcPts(109, 10, 80, 10), { stroke: P.cold, width: 4, opacity: 0.3, blur: 2 })
    // cúpula central
    const dome = circle(CX, CY, 56, { fill: P.light })
    freeform(dome, [
      [CX - 26, CY - 28, "#d4d8da", 0.2],
      [CX, CY, P.light, 0.3],
      [CX + 32, CY + 30, P.dark, 0.3],
    ])
    circle(CX, CY, 56, { fill: "none", stroke: P.dark, width: 3, opacity: 0.5 })
    circle(CX, CY, 30, { fill: "none", stroke: P.deep, width: 2, opacity: 0.35 })
    // antenas
    const ant = [[-35, 108], [-140, 98], [80, 100], [160, 92]]
    for (const [a, len] of ant) {
      const base = pt(56, a), tip = pt(len, a)
      brush([[base[0] + 3, base[1] + 4], [tip[0] + 3, tip[1] + 4]], { width: 5, color: alpha(P.deep, 0.5), taper: [0, 0.3] })
      brush([base, tip], { width: 5, color: P.dark, taper: [0, 0.5] })
      brush([base, tip], { width: 1.8, color: alpha(P.light, 0.7), taper: [0, 0.6] })
      circle(...pt(len * 0.78, a), 4.5, { fill: P.hull })
    }
    // plato de antena
    const dc = pt(64, -100)
    ellipse(dc[0] + 4, dc[1] + 5, 20, 20, { fill: P.deep, opacity: 0.4, blur: 3 })
    circle(dc[0], dc[1], 19, { fill: radial(dc[0] - 6, dc[1] - 6, 22, ["#c9cfd4", P.hull, P.dark]) })
    circle(dc[0], dc[1], 4, { fill: P.dark })
  })

  layer("danos", () => {
    // placas ennegrecidas (quemaduras suaves en multiply)
    seed(21)
    const burns = [[355, 200, 24], [355, 208, 18], [352, 290, 20], [358, 118, 22], [70, 150, 24], [356, 78, 16]]
    {
      for (const [r, a, s] of burns) {
        const c = pt(r, a)
        blob(jitter(ring(c[0], c[1], s, s * 0.9, 9, a), s * 0.2), { fill: P.burn, opacity: 0.4, blur: 6, blend: "multiply" })
        blob(jitter(ring(c[0], c[1], s * 0.55, s * 0.45, 7, a), s * 0.15), { fill: P.burn, opacity: 0.4, blur: 2, blend: "multiply" })
      }
      for (const a of DAMAGED) {
        const c = pt(358, a)
        blob(jitter(ring(c[0], c[1], 30, 26, 10, a), 4), { fill: P.burn, opacity: 0.55, blur: 5, blend: "multiply" })
      }
    }
    // módulos dañados: casco reventado con hueco oscuro
    for (const a of DAMAGED) {
      const c = pt(358, a)
      const hole = poly(roughen(ring(c[0] + 4, c[1] - 2, 22, 17, 9, a + 20), 7, { sharp: true }), { fill: "#1b1e26" })
      clip(hole, () => {
        circle(c[0] + 8, c[1] + 4, 14, { fill: "#7a5038", opacity: 0.35, blur: 7 })
        // vigas expuestas
        line(...pt2(340, a, -20), ...pt2(376, a, 16), { stroke: P.dark, width: 3, opacity: 0.7 })
        line(...pt2(372, a, -18), ...pt2(346, a, 12), { stroke: P.dark, width: 2, opacity: 0.6 })
      })
      // placas levantadas
      for (let k = 0; k < 3; k++) {
        const t = rand(-30, 30), r = rand(334, 384)
        poly(quad(a, r, r + 10, t, t + 12), { fill: P.dark, opacity: 0.8, rotate: rand(-30, 30) })
      }
    }
  })

  layer("colmena", () => {
    // costra orgánica verde trepando por el lado derecho del anillo
    seed(33)
    const A0 = -32, A1 = 30
    // masa de la costra: racimo de lóbulos irregulares, más grueso en el centro del frente
    const lobes = []
    for (let i = 0; i < 70; i++) {
      const u = i / 69, a = lerp(A0, A1, u) + rand(-1.5, 1.5)
      const th = Math.sin(u * Math.PI) ** 0.6
      const r = 355 + rand(-1, 1) * lerp(8, 44, th)
      lobes.push([...pt(r, a), lerp(6, 20, th) * rand(0.7, 1.1)])
    }
    const lobePts = ([x, y, s]) => jitter(ring(x, y, s, s * rand(0.7, 0.95), 7, rand(0, 180)), s * 0.12)
    // base continua bajo los lóbulos (sin rendijas)
    const baseO = [], baseI = []
    for (let i = 0; i <= 8; i++) {
      const u = i / 8, a = lerp(A0 + 3, A1 - 3, u)
      const th = Math.sin(u * Math.PI) ** 0.6
      baseO.push(pt(355 + lerp(4, 40, th) + rand(-3, 3), a))
      baseI.push(pt(355 - lerp(4, 40, th) + rand(-3, 3), a))
    }
    const base = blob([...baseO, ...baseI.reverse()], { fill: P.hive })
    for (const l of lobes) l[2] *= 1.25
    const crustC = [base, ...lobes.map(l => blob(lobePts(l), { fill: P.hiveM }))]
    clip(crustC, () => {
      // volumen general: luz arriba-izquierda, sombra abajo-derecha
      rect(0, 0, 1024, 1024, { fill: linear(770, 330, 930, 700, [P.hiveL, P.hiveM, P.hive, "#151f15"]) })
      // sombreado de cada lóbulo
      for (const [x, y, s] of lobes) {
        circle(x + s * 0.3, y + s * 0.35, s * 0.9, { fill: "#121a12", opacity: 0.35, blur: s * 0.3 })
        circle(x - s * 0.25, y - s * 0.3, s * 0.5, { fill: P.hiveL, opacity: 0.35, blur: s * 0.3 })
      }
      // nódulos de quitina
      for (let i = 0; i < 18; i++) {
        const a = rand(A0 + 6, A1 - 6), r = rand(R0, R1 + 8)
        const c = pt(r, a), s = rand(5, 11)
        circle(c[0] + 2, c[1] + 3, s, { fill: P.hive, opacity: 0.6, blur: 2 })
        circle(c[0], c[1], s, { fill: radial(c[0] - s * 0.4, c[1] - s * 0.4, s * 1.2, [P.hiveL, P.hiveM, P.hive]) })
      }
      // poros y celdillas de la costra
      for (let i = 0; i < 40; i++) {
        const c = pt(rand(R0, R1), rand(A0 + 4, A1 - 4))
        ellipse(c[0], c[1], rand(1.5, 3.5), rand(1.2, 2.5), { fill: "#101810", opacity: 0.5 })
      }
      grain({ opacity: 0.1 })
    })
    // pústulas brillantes (brillo contenido)
    for (const [r, a] of [[362, -16], [344, 6], [374, 20]]) {
      const c = pt(r, a)
      circle(c[0], c[1], 5, { fill: "#7aa244", opacity: 0.5, glow: { blur: 5, color: "#6f9f3a44" } })
      circle(c[0] - 1.5, c[1] - 1.5, 1.8, { fill: "#bcd58e", opacity: 0.5 })
    }
    // zarcillos que trepan más allá del frente: por el anillo y por el radio de 0°
    const tendrils = [
      [pt(350, A0 + 4), pt(356, A0 - 6), pt(346, A0 - 14), pt(352, A0 - 22)],
      [pt(372, A0 + 6), pt(380, A0 - 4), pt(374, A0 - 10)],
      [pt(340, A1 - 4), pt(334, A1 + 6), pt(342, A1 + 14), pt(338, A1 + 20)],
      [pt(366, A1 - 2), pt(372, A1 + 8), pt(366, A1 + 13)],
      [pt2(R0, 0, -8), pt2(290, 0, -13), pt2(262, 0, -8), pt2(240, 0, -12)],
      [pt2(R0, 0, 12), pt2(292, 0, 14), pt2(272, 0, 9)],
    ]
    for (const t of tendrils) {
      brush(shift(t, 2, 3), { width: [6, 3.5, 1], color: alpha("#141c18", 0.4) })
      brush(t, { width: [7, 4.5, 1.2], color: P.hiveM, taper: [0, 0.6] })
      brush(t, { width: [2.5, 1.6, 0.4], color: alpha(P.hiveL, 0.45), taper: [0, 0.6] })
    }
    // esporas sueltas delante del frente
    for (let i = 0; i < 14; i++) {
      const a = pick([rand(A0 - 22, A0), rand(A1, A1 + 22)])
      const c = pt(rand(R0 + 8, R1 - 6), a), s = rand(2.5, 6)
      circle(c[0], c[1], s, { fill: radial(c[0] - 1, c[1] - 1, s, [P.hiveL, P.hive]), opacity: 0.85 })
    }
  })

  layer("luces", () => {
    // ventanas, balizas y retoques de luz
    seed(51)
    const inHive = (a) => { const b = ((a + 540) % 360) - 180; return b > -62 && b < 50 }
    const win = (x, y, w = 2.6) => circle(x, y, w, { fill: P.win, opacity: rand(0.55, 0.9), glow: { blur: 3, color: "#e6dba655" } })
    // hileras de ventanas a lo largo del anillo (bordes interior y exterior)
    for (let i = 0; i < 150; i++) {
      const a = i * 2.4 + rand(-0.4, 0.4)
      if (inHive(a) || rand() < 0.25) continue
      const r = pick([R0 + 8, R1 - 8])
      win(...pt(r, a), 2.2)
    }
    // ventanas de los módulos (dos filas cortas)
    for (const a of MODS) {
      if (inHive(a)) continue
      for (let k = 0; k < 4; k++) {
        if (rand() < 0.2) continue
        win(...pt2(R1 + 2, a, -20 + k * 13), 2.4)
        if (rand() < 0.6) win(...pt2(R0 - 2, a, -20 + k * 13), 2.4)
      }
    }
    // ventanas del cubo
    for (let i = 0; i < 28; i++) {
      const a = i * (360 / 28)
      if (rand() < 0.2) continue
      win(...pt(93, a), 2.6)
    }
    // ventanas en los radios
    for (const a of SPOKES) {
      for (let k = 0; k < 6; k++) {
        if (a === 0 && k > 1) continue
        win(...pt2(140 + k * 28, a, 13), 1.8)
      }
    }
    // balizas de posición (muy tenues)
    for (const [r, a, c] of [[600, 45, "#d9a070"], [600, 225, "#8fb0d8"], [212, 315, "#d9a070"]]) {
      circle(...pt(r, a), 3, { fill: c, opacity: 0.65 })
      circle(...pt(r, a), 8, { fill: c, opacity: 0.15, blur: 4 })
    }
  })
})
