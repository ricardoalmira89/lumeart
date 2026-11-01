// dibujo.js: hoja de armas nuevas (3x2 celdas de 341x384, la última vacía)
canvas({ width: 1024, height: 768, background: null })

const P = {
  ink: "#1e2033",      // contorno
  white: "#eef0f7",    // metal blanco
  whiteSh: "#b6bbd0",  // sombra del metal blanco
  clamp: "#3d4260",    // abrazadera
  clampHi: "#5a6188",
  clampDk: "#2a2d45",
  steel: "#d3d8e6",    // acero de la broca
  steelDk: "#7c84a3",
  turq: "#22d6c6",
  turqDk: "#0e93a0",
  pink: "#ff5fa8",
  pinkDk: "#b02a6c",
  lila: "#b98cff",
  elec: "#2fa8ff",
}
const W = 7 // grosor de contorno
const HALO = { fill: "#ffffff", stroke: "#ffffff", width: W + 22, join: "round", cap: "round" }
const LINE = { fill: "none", stroke: P.ink, width: W, join: "round", cap: "round" }

// Pieza con pasada de halo blanco, relleno, detalles dentro de clip y contorno encima
function piece(halo, mk, fill, det, line) {
  if (halo) return mk(HALO)
  const s = mk({ fill })
  if (det) clip(s, det)
  mk({ ...LINE, ...(line || {}) })
}

// Base común: abrazadera en U + cuerpo blanco (origen: centro abajo de la abrazadera)
const CLAMP = [[-76, -90], [-69, -97], [69, -97], [76, -90], [76, -9], [67, 0], [40, 0], [31, -9],
  [31, -40], [-31, -40], [-31, -9], [-40, 0], [-67, 0], [-76, -9]]
function base(h) {
  piece(h, (st) => poly(CLAMP, st), P.clamp, () => {
    rect(-80, -100, 160, 14, { fill: P.clampDk })
    rect(-80, -86, 160, 8, { fill: P.clampHi, opacity: 0.6 })
    rect(52, -100, 30, 110, { fill: P.clampDk, opacity: 0.7 })
    brush([[-64, -70], [-64, -14]], { width: 6, color: P.clampHi, taper: [0.2, 0.3] })
  })
  if (!h) {
    circle(-52, -62, 6, { fill: "#6c7396", stroke: P.ink, width: 3 })
    circle(52, -62, 6, { fill: "#6c7396", stroke: P.ink, width: 3 })
  }
  piece(h, (st) => rect(-67, -177, 134, 104, { r: 26, ...st }), P.white, () => {
    blob([[-70, -98], [0, -92], [70, -100], [70, -60], [-70, -60]], { fill: P.whiteSh })
    blob([[50, -180], [72, -180], [72, -70], [52, -90], [48, -140]], { fill: P.whiteSh })
    brush([[-50, -110], [-52, -145], [-36, -162], [-10, -165]], { width: 7, color: "#ffffff", taper: [0.4, 0.4] })
  })
}

function module(x, by, h, emitterBehind, emitterFront) {
  group({ move: [x, by] }, () => group({ scale: 0.9, around: [0, 0] }, () => {
    for (const pass of [true, false]) {
      if (emitterBehind) emitterBehind(pass)
      base(pass)
      if (emitterFront) emitterFront(pass)
    }
  }))
}

layer("modulos", () => {
  // (1) TALADRO
  element("modulo_taladro", { pivot: [171, 192] }, () => {
    module(171, 350, 350, (h) => {
      // collar
      piece(h, (st) => rect(-32, -210, 64, 40, { r: 7, ...st }), "#565c7e", () => {
        rect(-40, -214, 80, 9, { fill: P.clampHi })
        rect(18, -214, 20, 50, { fill: P.clampDk, opacity: 0.7 })
      })
      // broca cónica en espiral
      piece(h, (st) => blob([[0, -345, 0], [58, -206, 0], [0, -190], [-58, -206, 0]], st), P.steel, () => {
        for (let i = 0; i < 6; i++) {
          const y = -212 - i * 27
          const tip = y < -250
          path(`M -70 ${y + 2} Q 0 ${y - 2} 70 ${y - 34} L 70 ${y - 23} Q 0 ${y + 9} -70 ${y + 13} Z`, { fill: tip ? P.turq : "#ffffff" })
          path(`M -70 ${y + 13} Q 0 ${y + 9} 70 ${y - 23} L 70 ${y - 14} Q 0 ${y + 18} -70 ${y + 22} Z`, { fill: tip ? P.turqDk : P.steelDk })
        }
        poly([[-30, -360], [30, -360], [30, -312], [-30, -312]], { fill: P.turq })
        poly([[0, -360], [70, -200], [70, -180], [6, -180]], { fill: "#3a3f6a", opacity: 0.28, blend: "multiply" })
        brush([[-6, -336], [-24, -262], [-36, -214]], { width: 5, color: "#ffffff", taper: [0.3, 0.4] })
      })
    })
  })
  // (2) BURBUJA
  element("modulo_burbuja", { pivot: [512, 192] }, () => {
    module(512, 330, 300, null, (h) => {
      // disco emisor
      piece(h, (st) => circle(0, -196, 54, st), P.white, () => {
        circle(10, -186, 54, { fill: P.whiteSh })
        circle(-3, -199, 50, { fill: P.white })
        brush([[-42, -200], [-36, -228], [-14, -244]], { width: 6, color: "#ffffff" })
      })
      // anillo rosa con su boca oscura
      piece(h, (st) => circle(0, -196, 36, st), P.pink, () => {
        circle(5, -191, 36, { fill: P.pinkDk })
        circle(-2, -198, 33, { fill: P.pink })
      }, { width: 5 })
      if (!h) circle(0, -196, 21, { fill: "#3b1f3e", stroke: P.ink, width: 4 })
      // burbuja a medio salir
      piece(h, (st) => circle(0, -246, 44, st), radial(-14, -262, 60, [[0, "#ffe4f3"], [0.55, "#ff9ccd"], [1, P.lila]]), () => {
        circle(14, -234, 44, { fill: "none", stroke: "#8fe9ff", width: 6, opacity: 0.5 })
        circle(0, -246, 40, { fill: "none", stroke: "#ffffff", width: 3, opacity: 0.5 })
        brush(arc(0, -246, 30, 190, 255, 8), { width: 8, color: "#ffffff", taper: [0.4, 0.4] })
        circle(16, -228, 5, { fill: "#ffffff", opacity: 0.8 })
      }, { stroke: P.pinkDk, width: 5 })
    })
  })
  // (3) CENTELLA
  element("modulo_centella", { pivot: [853, 192] }, () => {
    const L = [[-18, -186], [-50, -258], [-76, -312]]
    const R = L.map(([x, y]) => [-x, y])
    const ZAP = [[-66, -318], [-44, -348], [-22, -304], [2, -352], [20, -302], [44, -346], [66, -318]]
    module(853, 345, 345, (h) => {
      // antenas en horquilla
      for (const pts of [L, R]) {
        if (h) curve(pts, { stroke: "#ffffff", width: 18 + 22, cap: "round" })
        else {
          curve(pts, { stroke: P.ink, width: 18, cap: "round" })
          curve(pts, { stroke: P.white, width: 8, cap: "round" })
          curve(shift(pts, 2, 0), { stroke: P.whiteSh, width: 3, cap: "round" })
        }
      }
      // chispa (detrás de las bolas)
      if (h) polyline(ZAP, { stroke: "#ffffff", width: 30, join: "round", cap: "round" })
      else {
        polyline(ZAP, { stroke: "#1a5fd0", width: 20, join: "round", cap: "round", glow: { blur: 12, color: P.elec } })
        polyline(ZAP, { stroke: P.elec, width: 13, join: "round", cap: "round" })
        polyline(ZAP, { stroke: "#ffffff", width: 5, join: "round", cap: "round" })
      }
      // electrodos
      for (const s of [-1, 1]) {
        piece(h, (st) => circle(s * 78, -318, 15, st), P.elec, () => {
          circle(s * 78 + 3, -315, 15, { fill: "#1665c4" })
          circle(s * 78 - 1, -319, 13, { fill: P.elec })
          circle(s * 78 - 5, -323, 4.5, { fill: "#ffffff" })
        }, { width: 5 })
      }
      // casquillo
      piece(h, (st) => rect(-34, -200, 68, 30, { r: 8, ...st }), "#565c7e", () => {
        rect(-40, -204, 80, 9, { fill: P.clampHi })
      })
    })
  })
})

layer("proyectiles", () => {
  // (4) BROCA
  element("proyectil_broca", { pivot: [171, 576] }, () => {
    group({ move: [171, 576] }, () => {
      const cone = (st) => blob([[0, -140, 0], [70, 92, 0], [0, 122], [-70, 92, 0]], st)
      cone({ ...HALO, width: 8 + 24 })
      const s = cone({ fill: P.turq })
      clip(s, () => {
        for (let i = 0; i < 7; i++) {
          const y = 130 - i * 44
          path(`M -90 ${y} Q -10 ${y + 6} 90 ${y - 80} L 90 ${y - 64} Q -10 ${y + 22} -90 ${y + 16} Z`, { fill: "#ffffff" })
          path(`M -90 ${y + 16} Q -10 ${y + 22} 90 ${y - 64} L 90 ${y - 54} Q -10 ${y + 32} -90 ${y + 26} Z`, { fill: P.turqDk })
        }
        poly([[0, -150], [90, 92], [90, 140], [6, 140]], { fill: "#0b4f7a", opacity: 0.3, blend: "multiply" })
        brush([[-8, -110], [-30, -20], [-48, 70]], { width: 7, color: "#ffffff", taper: [0.3, 0.4] })
      })
      cone({ ...LINE, width: 8 })
    })
  })
  // (5) BURBUJA
  element("proyectil_burbuja", { pivot: [512, 576] }, () => {
    group({ move: [512, 576] }, () => {
      const R = 118
      circle(0, 0, R, { ...HALO, width: 5 + 24 })
      const s = circle(0, 0, R, { fill: radial(0, 0, R, [[0, "#ffd6ee66"], [0.6, "#ffb0da88"], [0.88, "#d79cffcc"], [1, "#b07cffee"]]) })
      clip(s, () => {
        // borde iridiscente más marcado
        circle(0, 0, R, { fill: "none", stroke: P.pink, width: 22, opacity: 0.6 })
        circle(10, 10, R, { fill: "none", stroke: P.lila, width: 30, opacity: 0.75 })
        brush(arc(0, 0, R - 14, 20, 80, 10), { width: 10, color: "#8fe9ff", taper: [0.4, 0.4] })
        brush(arc(0, 0, R - 14, 105, 150, 8), { width: 8, color: "#fff28a", taper: [0.4, 0.4] })
        // brillo blanco curvo arriba a la izquierda
        brush(arc(0, 0, R - 24, 190, 262, 12), { width: 20, color: "#ffffff", taper: [0.35, 0.35] })
        circle(-30, -78, 9, { fill: "#ffffff" })
        circle(56, 58, 7, { fill: "#ffffff", opacity: 0.7 })
      })
      circle(0, 0, R, { fill: "none", stroke: P.pinkDk, width: 5 })
    })
  })
})
