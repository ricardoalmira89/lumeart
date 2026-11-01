// Crucero de la flota humana: fondo lejano para shooter vertical, vista cenital, proa arriba.
canvas({ width: 1024, height: 1024, background: null })

// Luz cálida desde arriba a la izquierda, contraluz frío desde abajo a la derecha (toberas)
const P = {
  lit: "#7d8ba1",     // faceta iluminada del casco
  mid: "#647189",     // medio tono
  dark: "#475268",    // faceta en sombra
  deep: "#2f374a",    // juntas, huecos, placas oscuras
  plate: "#535d74",   // placas más oscuras
  stripe: "#a6b1c3",  // franjas blancas apagadas
  haze: "#14203a",    // atmósfera: azul marino del fondo
  warm: "#d9cfae",    // luz cálida
  win: "#e6dc9c",     // ventanas amarillo pálido
  blue: "#6fb4f0",    // toberas y rayos
  burn: "#2c2a33",    // quemaduras
}
const CX = 512

// Medio contorno del casco (lado derecho), de proa a popa
const HALF = [
  [512, 30], [526, 90], [546, 170], [564, 248], [584, 282], [590, 330],
  [620, 380], [636, 420], [636, 540], [618, 566], [618, 650],
  [650, 700], [670, 744], [670, 828], [640, 852], [608, 858], [600, 906],
]
const full = (half) => [...half, ...half.slice().reverse().map(([x, y]) => [2 * CX - x, y])]

element("crucero", { pivot: [512, 512] }, () => {
  layer("brillo toberas", () => {
    ellipse(512, 950, 150, 46, { fill: alpha(P.blue, 0.22), blur: 26 })
    for (const x of [452, 512, 572]) {
      ellipse(x, 955, 24, 42, { fill: alpha(P.blue, 0.45), blur: 12 })
      ellipse(x, 944, 12, 20, { fill: alpha("#cfe8ff", 0.55), blur: 6 })
    }
  })
  layer("casco", () => {
    const hull = poly(full(HALF), { fill: P.dark })
    // cubierta central: el contorno estrechado hacia el eje
    const DECK = HALF.map(([x, y], i) => [CX + (x - CX) * 0.58, i === 0 ? y + 40 : y + 4])
    clip(hull, () => {
      // bisel izquierdo iluminado, derecho en sombra
      rect(0, 0, CX, 1024, { fill: linear(300, 0, 512, 0, [P.mid, P.lit]) })
      rect(CX, 0, 512, 1024, { fill: linear(512, 0, 720, 0, [P.dark, shade(P.dark, -0.15)]) })
      // degradado general: más luz arriba
      rect(0, 0, 1024, 1024, { fill: linear(0, 40, 0, 900, [alpha(P.warm, 0.18), alpha(P.deep, 0.25)]), blend: "soft-light" })
      // cubierta plana
      const deck = poly(full(DECK), { fill: linear(400, 0, 640, 0, [P.lit, P.mid]) })
      clip(deck, () => {
        rect(0, 0, 1024, 1024, { fill: linear(0, 60, 0, 880, [alpha(P.warm, 0.25), alpha(P.deep, 0.3)]), blend: "soft-light" })
      })
      // sombra de contacto en el borde derecho de la cubierta
      polyline(full(DECK).slice(0, HALF.length), { stroke: alpha(P.deep, 0.5), width: 3, blur: 1.5 })
      polyline(full(DECK).slice(HALF.length), { stroke: alpha(P.stripe, 0.18), width: 2, blur: 1 })
      // cresta central del bisel en los costados
      // contraluz frío por el borde derecho y popa
      outline(hull, -6, alpha(P.blue, 0.14), { blur: 4 })
      ellipse(512, 900, 220, 80, { fill: alpha(P.blue, 0.12), blur: 30, blend: "screen" })
    })
  })
  // toberas: tres bloques de popa con boca azul
  layer("toberas", () => {
    for (const x of [452, 512, 572]) {
      const w = x === 512 ? 50 : 44
      rect(x - w / 2, 872, w, 52, { r: 4, fill: linear(x - w / 2, 0, x + w / 2, 0, [P.mid, P.dark, P.deep]) })
      rect(x - w / 2 + 4, 914, w - 8, 14, { r: 3, fill: shade(P.deep, -0.2) })
      ellipse(x, 922, w / 2 - 6, 6, { fill: alpha(P.blue, 0.8), blur: 2 })
      rect(x - w / 2, 872, 6, 52, { fill: alpha(P.stripe, 0.25) })
    }
  })
  layer("placas y franjas", () => {
    const hullShape = poly(full(HALF), { hidden: true })
    clip(hullShape, () => {
      seed(7)
      // juntas transversales del casco
      for (let y = 110; y < 900; y += rand(34, 52)) {
        line(380, y, 644, y, { stroke: alpha(P.deep, 0.22), width: 1.5 })
      }
      // juntas longitudinales en los biseles
      for (const s of [-1, 1]) {
        polyline(HALF.slice(3).map(([x, y]) => [CX + s * (x - CX) * 0.8, y]), { stroke: alpha(P.deep, 0.25), width: 1.5 })
        // ojos de buey en los costados
        for (let y = 300; y < 840; y += 9) {
          if (rand() < 0.35) {
            const w = HALF.reduce((a, [hx, hy], i) => (hy <= y ? hx : a), 600) - CX
            rect(CX + s * (w * 0.87) - 1.5, y, 3, 4, { fill: alpha(P.win, s < 0 ? 0.6 : 0.4), glow: { blur: 2, color: alpha(P.win, 0.4) } })
          }
        }
      }
      // placas más oscuras repartidas
      for (let i = 0; i < 26; i++) {
        const y = rand(140, 860)
        const x = rand(400, 600)
        rect(x, y, rand(16, 46), rand(18, 50), { fill: alpha(P.plate, rand(0.25, 0.5)), blend: "multiply" })
      }
      // placas de blindaje laterales en los sponsones
      for (const s of [-1, 1]) {
        rect(s < 0 ? 388 : 600, 430, 36, 100, { fill: alpha(P.plate, 0.45), blend: "multiply" })
        rect(s < 0 ? 364 : 624, 710, 40, 110, { fill: alpha(P.plate, 0.45), blend: "multiply" })
      }
      // franjas blancas: chevrones en proa y bandas en popa
      const stripeCol = alpha(P.stripe, 0.55)
      for (const [y, t] of [[150, 10], [176, 5]]) {
        poly([[400, y + 50], [512, y], [624, y + 50], [624, y + 50 + t], [512, y + t], [400, y + 50 + t]], { fill: stripeCol })
      }
      rect(300, 744, 1024 - 600, 9, { fill: alpha(P.stripe, 0.4) })
      rect(300, 760, 1024 - 600, 4, { fill: alpha(P.stripe, 0.3) })
      // proa: arista central, rejillas y sensores
      line(512, 60, 512, 250, { stroke: alpha(P.stripe, 0.18), width: 1.5 })
      for (let y = 100; y < 140; y += 6) rect(504, y, 16, 2.5, { fill: alpha(P.deep, 0.45) })
      for (const s of [-1, 1]) {
        poly([[512 + s * 14, 214], [512 + s * 34, 232], [512 + s * 34, 262], [512 + s * 14, 250]], { fill: alpha(P.deep, s < 0 ? 0.4 : 0.55) })
        circle(512 + s * 22, 232, 2, { fill: alpha(P.win, 0.6), glow: { blur: 3, color: alpha(P.win, 0.4) } })
      }
      // franja longitudinal a babor sobre la cubierta
      rect(478, 250, 7, 260, { fill: alpha(P.stripe, 0.35) })
      rect(539, 250, 7, 260, { fill: alpha(P.stripe, 0.22) })
    })
  })
  layer("superestructura", () => {
    // bloque visto desde arriba: sombra proyectada abajo-derecha, cara superior y bisel lateral iluminado
    const block = (pts, k) => {
      poly(shift(pts, 9, 12), { fill: alpha(P.deep, 0.45), blur: 5, blend: "multiply" })
      const top = poly(pts, { fill: linear(pts[0][0] - 40, 0, pts[0][0] + 60, 0, [shade(P.lit, k), shade(P.mid, k)]) })
      clip(top, () => {
        poly(shift(pts, 5, 6), { fill: alpha(P.dark, 0.5) })
        poly(grow(pts, 0.86), { fill: linear(0, pts[0][1], 0, pts[0][1] + 140, [shade(P.lit, k + 0.05), shade(P.mid, k)]) })
      })
      return top
    }
    const s1 = [[480, 492], [544, 492], [566, 530], [566, 690], [548, 712], [476, 712], [458, 690], [458, 530]]
    const s2 = [[490, 520], [534, 520], [550, 548], [550, 640], [536, 660], [488, 660], [474, 640], [474, 548]]
    const s3 = [[497, 548], [527, 548], [536, 566], [536, 604], [526, 616], [498, 616], [488, 604], [488, 566]]
    block(s1, -0.05)
    // ventanas del primer nivel
    seed(3)
    for (let y = 545; y < 700; y += 12) {
      for (const x of [463, 559]) {
        if (rand() < 0.75) rect(x - 1.5, y, 3, 6, { fill: alpha(P.win, x < 512 ? 0.85 : 0.6), glow: { blur: 3, color: alpha(P.win, 0.5) } })
      }
    }
    block(s2, 0.02)
    for (let x = 482; x < 545; x += 8) {
      if (rand() < 0.8) rect(x, 650, 4, 3, { fill: alpha(P.win, 0.8), glow: { blur: 3, color: alpha(P.win, 0.5) } })
    }
    block(s3, 0.08)
    // puente: franja de ventanas en la proa del último nivel
    rect(495, 552, 34, 4, { r: 1.5, fill: alpha(P.win, 0.85), glow: { blur: 4, color: alpha(P.win, 0.5) } })
    // antenas y sensores
    circle(512, 588, 9, { fill: shade(P.mid, -0.1) })
    circle(510, 586, 5, { fill: P.lit })
    line(512, 588, 512, 640, { stroke: alpha(P.deep, 0.5), width: 2 })
    rect(492, 676, 40, 20, { r: 3, fill: P.plate })
    rect(494, 678, 18, 6, { fill: alpha(P.stripe, 0.4) })
  })
  layer("torretas", () => {
    const turret = (x, y, ang, s = 1) => {
      ellipse(x + 6, y + 8, 26 * s, 24 * s, { fill: alpha(P.deep, 0.45), blur: 4, blend: "multiply" })
      // barbeta: anillo bajo la torreta
      circle(x, y, 24 * s, { fill: linear(x - 24, y - 24, x + 24, y + 24, [P.mid, P.deep]) })
      circle(x, y, 20 * s, { fill: shade(P.dark, -0.1) })
      // cañones dobles
      group({ rotate: ang, around: [x, y] }, () => {
        for (const dx of [-6, 6]) {
          rect(x + dx * s - 3, y - 54 * s, 6, 50 * s, { r: 2, fill: linear(x + dx * s - 3, 0, x + dx * s + 3, 0, [P.lit, P.dark]) })
          rect(x + dx * s - 3.5, y - 58 * s, 7, 7, { r: 1.5, fill: P.deep })
        }
        const housing = blob([[x - 18 * s, y + 18 * s, 0], [x - 22 * s, y - 6 * s, 0], [x - 10 * s, y - 22 * s, 0], [x + 10 * s, y - 22 * s, 0], [x + 22 * s, y - 6 * s, 0], [x + 18 * s, y + 18 * s, 0]], { fill: P.mid })
        clip(housing, () => {
          rect(x, y - 40, 40, 80, { fill: alpha(P.dark, 0.6) })
          rect(x - 40, y - 40, 40, 80, { fill: linear(x - 22, 0, x, 0, [P.lit, P.mid]) })
          rect(x - 40, y + 10 * s, 80, 30, { fill: alpha(P.deep, 0.35) })
          rect(x - 8 * s, y - 14 * s, 16 * s, 3, { fill: alpha(P.stripe, 0.4) })
        })
      })
    }
    turret(512, 304, -32, 1.1)
    turret(512, 428, -14, 1.25)
    turret(512, 756, -68, 1.25)
    turret(512, 832, 74, 1.1)
  })
  layer("daños", () => {
    const hullShape = poly(full(HALF), { hidden: true })
    clip(hullShape, () => {
      seed(21)
      for (const [x, y, r] of [[438, 360, 22], [590, 470, 26], [416, 800, 20], [556, 236, 14], [606, 728, 18], [470, 610, 12]]) {
        // quemadura difusa con estela hacia popa
        blob(roughen(ring(x, y, r, r * 0.8, 9), r * 0.4), { fill: alpha(P.burn, 0.5), blur: 5, blend: "multiply" })
        ellipse(x + rand(-3, 3), y + r * 1.6, r * 0.5, r * 1.6, { fill: alpha(P.burn, 0.35), blur: 7, blend: "multiply" })
        // impacto: hueco con brasa apagada
        blob(roughen(ring(x - 2, y - 2, r * 0.32, r * 0.26, 8), r * 0.12), { fill: alpha(shade(P.burn, -0.2), 0.55), blur: 1 })
        circle(x - 1, y - 1, r * 0.1, { fill: alpha("#c98a5a", 0.3), blur: 1.5 })
        // borde de metal abombado que atrapa la luz arriba a la izquierda
        curve(arc(x - 2, y - 2, r * 0.42, 250, 340, 5), { stroke: alpha(P.stripe, 0.25), width: 1.2 })
      }
      // placa desprendida a estribor: mancha con borde de luz
      poly([[620, 600], [640, 604], [638, 636], [618, 632]], { fill: alpha(P.deep, 0.55), blend: "multiply" })
      line(620, 600, 640, 604, { stroke: alpha(P.stripe, 0.3), width: 1.2 })
      grain({ opacity: 0.08, size: 1.2 })
    })
  })
  layer("bruma", () => {
    // velo atmosférico para empujar la nave al fondo y bajar el contraste
    const hullShape = poly(full(HALF), { hidden: true })
    const nozzles = rect(426, 872, 172, 56, { hidden: true })
    clip([hullShape, nozzles], () => {
      rect(0, 0, 1024, 1024, { fill: alpha(P.haze, 0.22) })
    })
  })
})

