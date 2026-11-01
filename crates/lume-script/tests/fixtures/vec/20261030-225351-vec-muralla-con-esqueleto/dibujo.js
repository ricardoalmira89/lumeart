// Núcleo de la muralla: placa orgánica con ojo, párpados, dientes y nervios, con esqueleto.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  white: "#ffffff",
  gInk: "#0e2113", gDark: "#1d4825", gMid: "#2f6b35", gBase: "#3b7d3f", gLight: "#5aa556", gHi: "#86cf78",
  vInk: "#1b0d29", vDark: "#3f1f6b", vBase: "#6a3a9e", vLight: "#9663cf", vHi: "#c9a2ef",
  rDark: "#5a0a12", rRed: "#d42a26", rOrange: "#ff7a2e", rYellow: "#ffc451", pupil: "#22050c",
  tInk: "#2a1406", tDark: "#c2650f", tBase: "#f3a12c", tLight: "#ffd47c",
}
const CX = 512, CY = 345           // centro del ojo
const EYE = 138                    // radio del hueco del ojo

// punto en ángulo matemático (0 = derecha, 90 = arriba)
const pa = (r, a, cx = CX, cy = CY) => [cx + r * Math.cos(a * Math.PI / 180), cy - r * Math.sin(a * Math.PI / 180)]
const arcPts = (r, a0, a1, n, f = () => 1) => range(n + 1).map(i => { const a = lerp(a0, a1, i / n); return pa(r * f(a), a) })
const mx = pts => pts.map(p => [1024 - p[0], p[1], ...p.slice(2)]).reverse()

// silueta de la placa (banda + óvalo), mitad izquierda de arriba abajo
const halfL = [[512, 131], [420, 136], [348, 151], [292, 178], [258, 212], [236, 240, 0.5], [150, 252], [60, 261], [-40, 266, 0],
  [-40, 468, 0], [60, 467], [150, 477], [232, 495], [305, 512], [385, 532], [455, 545], [512, 549]]
const halfR = mx(halfL)
// recorrido completo en sentido horario: arriba-centro → izquierda → abajo → derecha → arriba
const SIL = [...halfL, ...halfR.slice(1, 16)]
const OVAL = [[512, 131], [400, 138], [318, 162], [258, 212], [226, 270], [216, 340], [228, 420], [270, 482], [345, 525], [430, 546], [512, 550],
  [594, 546], [679, 525], [754, 482], [796, 420], [808, 340], [798, 270], [766, 212], [706, 162], [624, 138]]

// ---------------- Esqueleto ----------------
rig({ origin: [512, 615] })
bone("placa", { at: [512, 384] })
bone("mandibula", { parent: "placa", at: [512, 530] })
bone("iris", { parent: "placa", at: [CX, CY] })
bone("parpado_sup", { parent: "placa", at: [372, 485] })   // bisagra del párpado plegado (izquierda)
bone("parpado_inf", { parent: "placa", at: [652, 205] })   // bisagra del párpado plegado (derecha)
bone("nervio_izq", { parent: "placa", at: [262, 352] })
bone("nervio_der", { parent: "placa", at: [762, 352] })

// ---------------- Piezas, de atrás hacia delante ----------------
// 1. borde blanco de pegatina de la placa (detrás de todo)
part("placa", () => {
  blob(SIL, { fill: P.gInk, stroke: P.white, width: 30, behind: true, join: "round" })
})

// 2. dientes (asoman bajo la placa; sus raíces quedan escondidas)
part("mandibula", () => {
  // [x de la raíz, x de la punta, y de la punta, medio ancho]
  const TEETH = [[398, 404, 584, 17], [434, 438, 595, 18], [471, 474, 604, 19], [512, 512, 613, 20],
    [550, 548, 603, 19], [587, 584, 594, 18], [624, 619, 583, 17]]
  const tooth = ([x, tx, ty, w]) => [[x - w, 488], [x + w, 488], [x + w * 0.9, 540], [lerp(x + w * 0.7, tx, 0.5), lerp(540, ty, 0.55)],
    [tx, ty, 0], [lerp(x - w * 0.7, tx, 0.5), lerp(540, ty, 0.55)], [x - w * 0.9, 540]]
  // borde blanco de pegatina y contorno de todos los dientes
  for (const t of TEETH) blob(tooth(t), { fill: P.tBase, stroke: P.white, width: 24, behind: true })
  for (const t of TEETH) {
    const [x, tx, ty, w] = t
    const s = blob(tooth(t), { fill: P.tBase })
    clip(s, () => {
      // sombra de un tono a la derecha y luz a la izquierda
      poly([[x + 2, 480], [x + w + 4, 480], [x + w + 4, 560], [tx + 3, ty + 4], [tx, ty - 6]], { fill: P.tDark })
      brush([[x - w * 0.55, 500], [x - w * 0.5, 545], [lerp(x - w * 0.3, tx, 0.6), lerp(545, ty, 0.6)]], { width: [6, 5, 1], taper: [0, 0.6], color: P.tLight })
    })
    ink(s, { width: 3.5, weight: 1.5, color: P.tInk })
  }
})

// 3. placa trasera: banda, óvalo y cuenca del ojo
part("placa", () => {
  const body = blob(SIL, { fill: linear(0, 255, 0, 500, [P.gMid, [0.45, P.gBase], [0.8, P.gMid], [1, P.gDark]]) })
  clip(body, () => {
    // franjas de sombra bajo los nervios y luz en el borde superior de la banda
    for (const s of [-1, 1]) {
      const f = p => s < 0 ? p : [1024 - p[0], p[1]]
      brush([[-40, 330], [120, 330], [250, 322]].map(f), { width: 26, taper: [0, 0.6], color: alpha(P.gDark, 0.55) })
      brush([[-40, 388], [120, 386], [250, 392]].map(f), { width: 22, taper: [0, 0.6], color: alpha(P.gDark, 0.5) })
      brush([[-40, 446], [130, 452], [240, 480]].map(f), { width: 30, taper: [0, 0.5], color: alpha(P.gDark, 0.7) })
      brush([[-40, 276], [100, 270], [220, 256]].map(f), { width: 10, taper: [0, 0.5], color: alpha(P.gLight, 0.6) })
    }
  })
  // cuenca del ojo: fondo rojo oscuro con brillo que se apaga hacia el borde
  circle(CX, CY, EYE + 14, { fill: P.rDark })
  circle(CX, CY, EYE + 4, { fill: radial(CX, CY, EYE + 4, [[0, P.rRed], [0.8, "#a8151c"], [1, "#4a0710"]]) })
})

// 4. ojo rojo con pupila rasgada
part("iris", () => {
  const ir = circle(CX, CY, 122, { fill: radial(CX + 4, CY + 4, 122, [[0, P.rYellow], [0.3, P.rOrange], [0.62, "#f0482a"], [0.88, P.rRed], [1, "#a3141c"]], { fx: CX - 10, fy: CY - 14 }) })
  clip(ir, () => {
    // anillos concéntricos suaves
    circle(CX + 6, CY + 2, 92, { fill: "none", stroke: alpha(P.rYellow, 0.35), width: 10 })
    circle(CX - 10, CY + 6, 58, { fill: "none", stroke: alpha("#ffffff", 0.12), width: 8 })
    // venas finas que irradian desde la pupila
    seed(7)
    for (let i = 0; i < 16; i++) {
      const a = i * 22.5 + rand(-6, 6)
      const r0 = rand(34, 48), r1 = rand(92, 118), bend = rand(-14, 14)
      brush([pa(r0, a), pa((r0 + r1) / 2, a + bend * 0.6), pa(r1, a + bend)], { width: [1, 3, 1], taper: [0.4, 0.6], color: alpha("#ffb070", 0.75) })
    }
    // sombra de un tono en la parte baja derecha
    blob([pa(130, -60), pa(130, -10), pa(130, 30), pa(112, 5), pa(104, -40), pa(110, -80), pa(130, -100)], { fill: alpha("#7a0d18", 0.45), blend: "multiply" })
  })
  // pupila rasgada
  const pu = blob([[CX, 262, 0], [CX + 15, 300], [CX + 20, 345], [CX + 14, 392], [CX, 428, 0], [CX - 14, 392], [CX - 20, 345], [CX - 15, 300]], { fill: P.pupil })
  clip(pu, () => {
    brush([[CX + 6, 290], [CX + 10, 345], [CX + 5, 400]], { width: [1, 5, 1], color: alpha("#b8232a", 0.8) })
  })
  ink(pu, { width: 2, weight: 1.5, color: "#120206" })
  // brillos blancos nítidos
  ellipse(478, 309, 19, 10, { fill: P.white, rotate: -32 })
  circle(504, 287, 6, { fill: P.white })
})

// 5. párpados (dibujados plegados detrás de la placa, a los lados del ojo)
// Cada párpado se diseña cerrado (medio disco sobre el ojo) y se dibuja girado 90° y
// desplazado a su escondite. La acción "abrir" lo devuelve a su sitio con { rot: 90, y: ∓140 }.
const TU = p => [372 + (p[1] - CY), CY - (p[0] - 372) + 140, ...p.slice(2)]
const TL = p => [652 + (p[1] - CY), CY - (p[0] - 652) - 140, ...p.slice(2)]
function lid(T, up, base, dark, light) {
  const s = up ? 1 : -1               // 1 = mitad de arriba, -1 = mitad de abajo
  const R = EYE + 6
  const outline = [...arcPts(R, up ? 0 : 180, up ? 180 : 360, 40), [CX - R, CY + 5 * s], [CX + R, CY + 5 * s]]
  const L = poly(outline.map(T), { fill: base })
  clip(L, () => {
    // láminas concéntricas del párpado
    for (const r of [104, 66]) {
      const band = [...arcPts(r, up ? 0 : 180, up ? 180 : 360, 30)]
      polyline(band.map(T), { stroke: dark, width: 6, cap: "round" })
      polyline(band.map(p => [CX + (p[0] - CX) * (r + 7) / r, CY + (p[1] - CY) * (r + 7) / r]).map(T), { stroke: light, width: 3, cap: "round", opacity: 0.6 })
    }
    // sombra de un tono en el borde que se cierra y a la derecha
    poly([[CX - R, CY + 6 * s], [CX + R, CY + 6 * s], [CX + R, CY - 22 * s], [CX - R, CY - 16 * s]].map(T), { fill: dark })
    poly([...arcPts(R + 4, up ? 0 : 300, up ? 60 : 360, 12), [CX + 40, CY]].map(T), { fill: alpha(dark, 0.6) })
    // brillo marcado arriba a la izquierda
    if (up) brush(arcPts(R - 18, 150, 108, 10).map(T), { width: [3, 9, 3], taper: [0.3, 0.4], color: P.vHi })
    else brush(arcPts(R - 18, 205, 235, 8).map(T), { width: [2, 6, 2], color: alpha(P.vHi, 0.7) })
  })
  ink(L, { width: 5, weight: 1.8, color: P.vInk })
}
part("parpado_sup", () => lid(TU, true, P.vBase, P.vDark, P.vLight))
part("parpado_inf", () => lid(TL, false, shade(P.vBase, -0.15), P.vDark, P.vLight))

// 6. placa delantera: cara del óvalo con el hueco, anillo de segmentos y marco violeta
function bolt(x, y, r = 9) {
  circle(x, y, r + 3, { fill: P.tInk })
  circle(x, y, r, { fill: radial(x - 3, y - 3, r * 1.3, [P.tLight, P.tBase, P.tDark]) })
  circle(x - r * 0.35, y - r * 0.35, r * 0.28, { fill: P.white })
}
part("placa", () => {
  // cara del óvalo con el hueco del ojo
  const face = blob(OVAL, { fill: radial(400, 230, 420, [[0, P.gLight], [0.35, P.gBase], [0.75, P.gMid], [1, P.gDark]]) })
  cut(face, [circle(CX, CY, EYE, { hidden: true })])
  clip(face, () => {
    // sombra de un tono en el lado derecho y abajo
    blob([[700, 150], [790, 230], [820, 340], [790, 450], [700, 530], [560, 560], [620, 520], [720, 430], [745, 330], [710, 220], [650, 160]], { fill: P.gDark })
    // luz en el lomo superior izquierdo
    brush([[268, 300], [282, 228], [330, 178], [410, 152], [500, 146]], { width: [4, 13, 10, 4], taper: [0.3, 0.5], color: P.gHi })
    brush([[300, 470], [360, 512], [440, 535]], { width: [2, 6, 2], color: alpha(P.gLight, 0.6) })
    // sombra de oclusión alrededor del marco
    circle(CX, CY, 200, { fill: "none", stroke: alpha(P.gInk, 0.35), width: 30, blur: 10 })
  })
  // contorno de tinta de toda la silueta (más grueso en la sombra)
  ink(blob(SIL, { hidden: true }), { width: 7, weight: 1.8, color: P.gInk })

  // anillo de segmentos verdes
  for (let k = 0; k < 12; k++) {
    const a0 = k * 30 + 3.5, a1 = k * 30 + 26.5
    const outer = range(9).map(i => { const a = lerp(a0, a1, i / 8) * Math.PI / 180; return [CX + 228 * Math.cos(a), CY - 186 * Math.sin(a)] })
    const inner = range(9).map(i => { const a = lerp(a1, a0, i / 8) * Math.PI / 180; return [CX + 150 * Math.cos(a), CY - 150 * Math.sin(a)] })
    const lit = Math.cos((lerp(a0, a1, 0.5) - 135) * Math.PI / 180)   // 1 = de cara a la luz
    const sg = poly([...outer, ...inner], { fill: mix(P.gDark, P.gBase, clamp(lit * 0.5 + 0.5, 0, 1)), join: "round" })
    clip(sg, () => polyline(outer, { stroke: alpha(P.gHi, 0.5 * clamp(lit, 0, 1)), width: 8 }))
    ink(sg, { width: 4, weight: 1.6, color: P.gInk })
  }

  // marco violeta en cuatro piezas, separadas por una cruz
  const G = 26
  for (let q = 0; q < 4; q++) {
    const a0 = q * 90 + Math.asin(G / 178) * 180 / Math.PI, a1 = q * 90 + 90 - Math.asin(G / 178) * 180 / Math.PI
    const b0 = q * 90 + Math.asin(G / 120) * 180 / Math.PI, b1 = q * 90 + 90 - Math.asin(G / 120) * 180 / Math.PI
    const sq = a => 1 + 0.07 * Math.sin(2 * (a - q * 90) * Math.PI / 180) ** 2   // esquinas algo cuadradas
    const pts = [...arcPts(178, a0, a1, 24, sq), ...arcPts(120, b1, b0, 20)]
    const lit = Math.cos((q * 90 + 45 - 135) * Math.PI / 180)
    const base = mix(P.vBase, lit > 0 ? P.vLight : P.vDark, Math.abs(lit) * 0.25)
    const qd = poly(pts, { fill: base, join: "round" })
    clip(qd, () => {
      // sombra de un tono hacia abajo a la derecha
      poly(shift(pts, 9, 10), { fill: "none", stroke: P.vDark, width: 26 })
      // banda clara junto al ojo y reflejo
      polyline(arcPts(140, b0 + 4, b1 - 4, 16), { stroke: alpha(P.vLight, 0.85), width: 12, cap: "round" })
      polyline(arcPts(160, a0 + 8, a1 - 8, 16, sq), { stroke: alpha(P.vDark, 0.5), width: 5, cap: "round" })
    })
    ink(qd, { width: 5, weight: 1.8, color: P.vInk })
    // brillos blancos marcados
    const m = q * 90 + 45
    brush(arcPts(166, m + 22, m + 4, 6, sq), { width: [1.5, 5, 1.5], color: alpha(P.white, q === 1 ? 0.9 : 0.55) })
    if (q === 1) brush(arcPts(118 + 22, 172, 150, 6), { width: [2, 6, 2], color: alpha(P.white, 0.8) })
    // remache
    bolt(...pa(140, m))
  }
})

// 7. nervios violetas de los lados
// [puntos, grosor máximo, remaches (índice u a lo largo)]
const NERVES = [
  [[[-40, 300], [60, 306], [130, 307], [205, 295], [288, 283]], 22, [0.45, 0.72]],
  [[[-40, 356], [80, 352], [180, 350], [290, 352]], 15, []],
  [[[-40, 408], [40, 404], [110, 405], [195, 420], [290, 442]], 21, [0.32, 0.66]],
  [[[62, 270], [110, 270], [150, 282], [175, 296]], 9, []],
  [[[44, 452], [96, 449], [130, 438], [152, 424]], 9, []],
]
function nerves(f, dark) {
  const tone = c => dark ? shade(c, -0.18) : c
  const wf = w => u => w * (1 - 0.85 * smoothstep(0.35, 1, u))
  for (const [pts, w] of NERVES) brush(pts.map(f), { width: u => wf(w)(u) + 8, taper: [0, 0.15], color: P.vInk })
  for (const [pts, w, bolts] of NERVES) {
    const q = pts.map(f)
    brush(q, { width: wf(w), taper: [0, 0.2], color: tone(P.vBase) })
    brush(shift(q, 0, w * 0.22), { width: u => wf(w)(u) * 0.4, taper: [0, 0.3], color: tone(P.vDark), opacity: 0.8 })
    brush(shift(q, 0, -w * 0.2), { width: u => wf(w)(u) * 0.35, taper: [0.05, 0.4], color: tone(P.vLight) })
    brush(shift(q, 0, -w * 0.28).slice(0, 3), { width: [1, 3, 1], color: alpha(P.white, 0.6) })
    for (const u of bolts) {
      // punto a lo largo de la polilínea
      const i = Math.min(q.length - 2, Math.floor(u * (q.length - 1))), t = u * (q.length - 1) - i
      bolt(lerp(q[i][0], q[i + 1][0], t), lerp(q[i][1], q[i + 1][1], t), w > 20 ? 8 : 7)
    }
  }
}
part("nervio_izq", () => nerves(p => p, false))
part("nervio_der", () => nerves(p => [1024 - p[0], p[1]], true))

// ---------------- Acciones ----------------
const OPEN = { parpado_sup: { rot: 0, x: 0, y: 0 }, parpado_inf: { rot: 0, x: 0, y: 0 } }
const SHUT = { parpado_sup: { rot: 90, x: 0, y: -140 }, parpado_inf: { rot: 90, x: 0, y: 140 } }
action("abrir", { duration: 0.5, loop: false }, [
  [0, { ...SHUT, iris: { rot: 0, x: 0, y: 0 } }],
  [0.15, { parpado_sup: { rot: 86, x: 0, y: -133 }, parpado_inf: { rot: 86, x: 0, y: 133 } }],
  [0.7, { parpado_sup: { rot: 8, x: 0, y: -12 }, parpado_inf: { rot: 8, x: 0, y: 12 } }],
  [1, { ...OPEN }],
])
action("mirar", { duration: 2, loop: true }, [
  [0, { ...OPEN, iris: { rot: 0, x: 0, y: 0 }, mandibula: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
  [0.125, { mandibula: { rot: 0, x: 0, y: 7 }, nervio_izq: { rot: 2, x: -3, y: 2 }, nervio_der: { rot: -2, x: 3, y: 2 } }],
  [0.25, { iris: { rot: 0, x: -14, y: 1 }, mandibula: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
  [0.375, { mandibula: { rot: 0, x: 0, y: 7 }, nervio_izq: { rot: 2, x: -3, y: 2 }, nervio_der: { rot: -2, x: 3, y: 2 } }],
  [0.5, { iris: { rot: 0, x: 0, y: 0 }, mandibula: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
  [0.625, { mandibula: { rot: 0, x: 0, y: 7 }, nervio_izq: { rot: 2, x: -3, y: 2 }, nervio_der: { rot: -2, x: 3, y: 2 } }],
  [0.75, { iris: { rot: 0, x: 14, y: 1 }, mandibula: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
  [0.875, { mandibula: { rot: 0, x: 0, y: 7 }, nervio_izq: { rot: 2, x: -3, y: 2 }, nervio_der: { rot: -2, x: 3, y: 2 } }],
  [1, { iris: { rot: 0, x: 0, y: 0 }, mandibula: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
])
action("golpe", { duration: 0.25, loop: false }, [
  [0, { placa: { rot: 0, x: 0, y: 0 }, mandibula: { rot: 0, x: 0, y: 0 }, iris: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
  [0.2, { placa: { rot: -1.5, x: -14, y: 5 }, mandibula: { rot: 0, x: 0, y: 30 }, iris: { rot: 0, x: 6, y: -3 }, nervio_izq: { rot: -4, x: 0, y: 0 }, nervio_der: { rot: 4, x: 0, y: 0 } }],
  [0.45, { placa: { rot: 1.2, x: 11, y: -3 }, mandibula: { rot: 0, x: 0, y: 24 }, iris: { rot: 0, x: -5, y: 2 }, nervio_izq: { rot: 3, x: 0, y: 0 }, nervio_der: { rot: -3, x: 0, y: 0 } }],
  [0.7, { placa: { rot: -0.5, x: -5, y: 1 }, mandibula: { rot: 0, x: 0, y: 8 }, iris: { rot: 0, x: 2, y: 0 }, nervio_izq: { rot: -1, x: 0, y: 0 }, nervio_der: { rot: 1, x: 0, y: 0 } }],
  [1, { placa: { rot: 0, x: 0, y: 0 }, mandibula: { rot: 0, x: 0, y: 0 }, iris: { rot: 0, x: 0, y: 0 }, nervio_izq: { rot: 0, x: 0, y: 0 }, nervio_der: { rot: 0, x: 0, y: 0 } }],
])
