// Plataforma de tablones: la madera sobre la que anda el náufrago en "Los Anillos".
// De lado, a ras de agua: los tablones van atravesados (se les ven las testas) sobre dos troncos
// atados con cuerda. Luz baja y cálida del atardecer desde la izquierda. La línea de flotación
// (AGUA) queda a la altura de la superficie del mar; por debajo, todo está mojado y más oscuro.
canvas({ width: 600, height: 240, background: null })
setLight(-1, -0.5)

const AGUA = 150        // línea de flotación
const CUBIERTA = 104    // donde pisa el personaje
const P = {
  madera: "#8a5a36", maderaL: "#d4955a", maderaD: "#4a2e1c", testa: "#b07a4c", anillo: "#6b4428",
  tronco: "#5e3f29", troncoL: "#9a6a42", mojado: "#2a2420",
  cuerda: "#d2b27a", cuerdaD: "#7a6040", musgo: "#4f7a52", luz: "#ffc58a"
}
seed(7)

// tronco de fondo: asoma por detrás, más oscuro
layer("tronco lejano", () => {
  const t = rect(48, 124, 504, 34, { fill: shade(P.tronco, -0.35), r: 17 })
  clip(t, () => {
    rect(40, AGUA - 2, 520, 40, { fill: alpha(P.mojado, 0.6) })
    for (let i = 0; i < 9; i++) brush([[60 + i * 55, 132 + rand(-3, 3)], [100 + i * 55, 134 + rand(-3, 3)]], { width: 2, color: alpha("#1d140e", 0.5) })
  })
})

// tablones atravesados: de lado se ven sus testas, cada una con sus anillos y sus grietas
layer("tablones", () => {
  let x = 34
  while (x < 520) {
    const w = Math.min(rand(42, 54), 566 - x), top = CUBIERTA + rand(-2, 2), h = 28 + rand(-1, 2)
    const ang = rand(-1.2, 1.2)
    group({ rotate: ang, around: [x + w / 2, top + h / 2] }, () => {
      const tb = rect(x, top, w, h, { fill: linear(0, top, 0, top + h, [P.testa, P.madera]), r: 3 })
      clip(tb, () => {
        // anillos de la testa: el corazón del árbol cae fuera de la tabla
        const cx = x + w * rand(0.1, 0.9), cy = top + h * rand(1.2, 1.8)
        for (let r = 8; r < 60; r += rand(5, 8)) circle(cx, cy, r, { fill: "none", stroke: alpha(P.anillo, 0.5), width: 1.2 })
        // grieta
        if (rand() < 0.6) brush([[x + w * 0.3, top], [x + w * 0.4, top + h * 0.5], [x + w * 0.35, top + h * 0.8]], { width: [2, 1], color: alpha(P.maderaD, 0.8) })
        // luz rasante desde la izquierda y sombra a la derecha
        rect(x, top, 4, h, { fill: alpha(P.luz, 0.55) })
        rect(x, top, w, 3, { fill: alpha(P.luz, 0.6) })
        rect(x + w - 6, top, 6, h, { fill: alpha(P.maderaD, 0.45) })
        rect(x, top + h - 7, w, 7, { fill: alpha(P.mojado, 0.35) })
      })
    })
    x += w + rand(2, 4)
  }
})

// tronco delantero: largo, con sus testas redondas en los extremos, medio hundido
layer("tronco", () => {
  const t = rect(26, 126, 548, 40, { fill: linear(0, 126, 0, 166, [P.troncoL, P.tronco, P.maderaD]), r: 20 })
  clip(t, () => {
    // corteza: vetas a lo largo
    for (let i = 0; i < 26; i++) {
      const y = rand(130, 162), x0 = rand(30, 520)
      brush([[x0, y], [x0 + rand(40, 110), y + rand(-2, 2)]], { width: rand(1.2, 2.4), color: alpha(i % 3 ? "#3b2618" : P.troncoL, 0.55) })
    }
    // brillo de la luz baja sobre el lomo
    brush([[40, 131], [300, 129], [560, 132]], { width: [5, 3, 1], color: alpha(P.luz, 0.5) })
    // lo que queda bajo el agua, mojado y oscuro
    rect(20, AGUA, 560, 30, { fill: linear(0, AGUA, 0, AGUA + 16, [alpha(P.mojado, 0.55), alpha(P.mojado, 0.85)]) })
    brush([[26, AGUA + 1], [574, AGUA + 1]], { width: 1.5, color: alpha("#d9f3ee", 0.35) })
  })
  // testas del tronco
  for (const cx of [46, 554]) {
    const e = ellipse(cx, 146, 14, 20, { fill: P.testa })
    clip(e, () => {
      for (let r = 4; r < 22; r += 4) ellipse(cx + 1, 146, r * 0.7, r, { fill: "none", stroke: alpha(P.anillo, 0.6), width: 1 })
      rect(cx - 14, AGUA, 28, 20, { fill: alpha(P.mojado, 0.6) })
    })
  }
})

// cuerdas: atan los tablones al tronco en tres sitios
layer("cuerdas", () => {
  for (const cx of [92, 300, 508]) {
    // vueltas cruzadas, de la cubierta a la panza del tronco
    for (const s of [-1, 1]) {
      for (let k = 0; k < 3; k++) {
        const d = 10 - k * 6
        const pts = [[cx + s * d, CUBIERTA + 1], [cx, 131], [cx - s * d, 163]]
        brush(pts, { width: 4.5, color: P.cuerdaD, taper: [0, 0] })
        brush(pts, { width: 2.6, color: P.cuerda, taper: [0, 0] })
        brush(pts.slice(0, 2).map(p => [p[0] - 0.8, p[1] + 0.5]), { width: 1, color: alpha("#fff0c8", 0.55), taper: [0, 0] })
      }
    }
    // vuelta horizontal que sujeta el atado
    brush([[cx - 14, CUBIERTA + 14], [cx + 16, CUBIERTA + 13]], { width: 4, color: P.cuerdaD, taper: [0, 0] })
    brush([[cx - 14, CUBIERTA + 13], [cx + 16, CUBIERTA + 12]], { width: 2.2, color: P.cuerda, taper: [0, 0] })
  }
})

// musgo y algas a la altura del agua: la plataforma lleva tiempo flotando
layer("musgo", () => {
  for (let i = 0; i < 22; i++) {
    const x = rand(40, 560), y = AGUA + rand(-3, 2)
    brush([[x, y], [x + rand(-3, 3), y + rand(6, 16)]], { width: [2.5, 0.8], color: alpha(P.musgo, rand(0.5, 0.85)) })
  }
  for (let i = 0; i < 10; i++) ellipse(rand(40, 560), AGUA - rand(0, 3), rand(5, 12), rand(1.5, 3), { fill: alpha(P.musgo, 0.6) })
})
