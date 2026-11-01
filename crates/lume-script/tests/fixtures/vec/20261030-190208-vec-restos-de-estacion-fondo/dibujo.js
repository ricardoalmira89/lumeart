// Restos de estación espacial (fondo lejano, parallax). Poco contraste, sin contornos.
canvas({ width: 1024, height: 1024, background: null })
seed(7)

// Paleta contenida: metal gris azulado, sombras azul marino, acentos apagados
const P = {
  luz: "#727d92",     // cara iluminada (contenida)
  medio: "#4e596f",   // metal base
  sombra: "#2c344b",  // lado en sombra azul
  hondo: "#191f32",   // interiores y huecos
  frio: "#43598a",    // contraluz frío
  quema: "#15151e",   // quemaduras
  ventana: "#a97d52", // luz cálida tenue
  panel: "#29365a",   // células solares
  panelLuz: "#4b5c80",
  colmena: "#3f5a40", // restos orgánicos
  colmenaLuz: "#5f7a4c",
}

// Todo se dibuja en un marco local horizontal y se gira -30° (eje de abajo-izquierda a arriba-derecha).
// En local, la luz viene de arriba (y negativa).
const R = 95, CY = 512
const vgrad = (top, bot) => linear(0, CY - R, 0, CY + R, [[0, top], [0.35, mix(top, P.medio, 0.4)], [0.7, P.sombra], [1, bot]])

element("estacion", { pivot: [512, 512] }, () => {
  group({ move: [0, -40], rotate: -30, around: [512, 512], scale: 0.9 }, () => {
    layer("paneles solares", () => {
      // mástiles de celosía y dos paneles doblados con agujeros
      const viga = (x1, y1, x2, y2, w, col) => {
        line(x1, y1, x2, y2, { stroke: col || P.sombra, width: w, cap: "butt" })
        line(x1, y1, x2, y2, { stroke: P.luz, width: 1.5, opacity: 0.3, dash: "10 8" })
      }
      const at = (q, u, v) => {
        const a = [lerp(q[0][0], q[1][0], u), lerp(q[0][1], q[1][1], u)]
        const b = [lerp(q[3][0], q[2][0], u), lerp(q[3][1], q[2][1], u)]
        return [lerp(a[0], b[0], v), lerp(a[1], b[1], v)]
      }
      const panel = (q, col, nu, nv, holes) => {
        const s = poly(q, { fill: linear(q[0][0], q[0][1], q[2][0], q[2][1], [shade(col, 0.15), col, shade(col, -0.3)]) })
        if (holes.length) cut(s, holes.map(h => blob(jitter(ring(...at(q, h[0], h[1]), h[2], h[2] * 0.8, 9), h[2] * 0.3), { hidden: true })))
        clip(s, () => {
          for (let i = 1; i < nu; i++) line(...at(q, i / nu, 0), ...at(q, i / nu, 1), { stroke: P.hondo, width: 2, opacity: 0.55 })
          for (let j = 1; j < nv; j++) line(...at(q, 0, j / nv), ...at(q, 1, j / nv), { stroke: P.hondo, width: 2, opacity: 0.55 })
          // reflejo suave del cielo en las células
          line(...at(q, 0, 0.15), ...at(q, 1, 0.45), { stroke: P.panelLuz, width: 30, opacity: 0.25, blur: 12 })
          // chamuscado
          blob(roughen(ring(...at(q, rand(0.3, 0.7), rand(0.3, 0.7)), 40, 30, 9), 10), { fill: P.quema, opacity: 0.4, blur: 8 })
        })
        // marco lateral tenue
        polyline([q[0], q[1]], { stroke: P.medio, width: 3, opacity: 0.6 })
        return s
      }
      // mástil superior y panel doblado en dos tramos
      viga(300, CY - R + 4, 300, 330, 12)
      panel([[200, 330], [400, 330], [410, 200], [205, 205]], P.panel, 5, 3, [[0.7, 0.4, 16]])
      panel([[205, 205], [410, 200], [460, 70], [250, 90]], shade(P.panel, -0.15), 5, 3, [[0.3, 0.5, 22], [0.75, 0.75, 10]])
      // mástil inferior torcido y panel roto colgando
      viga(320, CY + R - 4, 300, 690, 12)
      panel([[215, 690], [395, 700], [370, 830], [180, 805]], shade(P.panel, -0.1), 4, 3, [[0.55, 0.45, 26], [0.15, 0.8, 12]])
      panel([[180, 805], [370, 830], [300, 905], [140, 870]], shade(P.panel, -0.3), 4, 2, [[0.6, 0.5, 14]])
    })
    layer("anillo roto", () => {
      // tramo del anillo habitable con radio y radio partido
      // cinta del anillo vista en escorzo: dos bordes elípticos desplazados a lo largo del eje
      const cx = 680, rx = 175, ry = 345
      const a0 = 22, a1 = 150
      const pt = (a, k) => {
        const r = a * Math.PI / 180
        return [cx + k * 46 + (rx - k * 12) * Math.cos(r), CY + (ry - k * 16) * Math.sin(r)]
      }
      const kin = 1
      const outer = range(14).map(i => pt(lerp(a0, a1, i / 13), 0))
      const inner = range(14).map(i => pt(lerp(a1, a0, i / 13), 1))
      // radio que une el anillo al módulo
      rect(678, CY + R - 10, 26, 250, { fill: linear(678, 0, 704, 0, [P.medio, P.hondo]) })
      for (let y = CY + R + 20; y < CY + R + 230; y += 30) line(680, y, 702, y + 14, { stroke: P.hondo, width: 2, opacity: 0.5 })
      // radio partido hacia arriba
      poly([[728, CY - R + 10], [752, CY - R + 10], [770, 300], [760, 288, 0], [752, 296], [740, 280], [736, 300]], { fill: linear(728, 0, 760, 0, [P.luz, P.sombra]) })
      // band del anillo con bordes rotos
      const brk = (p, q) => range(4).map(i => { const t = (i + 1) / 5; return [lerp(p[0], q[0], t) + rand(-14, 14), lerp(p[1], q[1], t) + rand(-10, 10), 0] })
      const pts = [...outer, ...brk(outer[13], inner[0]), ...inner, ...brk(inner[13], outer[0])]
      const anillo = poly(pts, { fill: linear(0, 650, 0, CY + ry, [P.medio, P.sombra, P.hondo]) })
      clip(anillo, () => {
        // cara superior iluminada (franja fina por dentro)
        curve(inner.map(p => [p[0], p[1] - 4]), { stroke: P.luz, width: 10, opacity: 0.35, blur: 3 })
        // juntas y ventanas
        for (let i = 1; i < 13; i++) {
          const a = lerp(a0, a1, i / 13)
          line(...pt(a, 1), ...pt(a, kin), { stroke: P.hondo, width: 2.5, opacity: 0.5 })
          const m = pt(a + 4, 0.5)
          const on = i === 9
          rect(m[0] - 4, m[1] - 4, 8, 8, { fill: on ? P.ventana : P.hondo, opacity: on ? 0.7 : 0.6, r: 1 })
          if (on) circle(m[0], m[1], 10, { fill: P.ventana, opacity: 0.2, blur: 5 })
        }
        curve(outer, { stroke: P.frio, width: 8, opacity: 0.4, blur: 3 })
        blob(roughen(ring(...pt(60, 0.5), 50, 36, 9), 10), { fill: P.quema, opacity: 0.5, blur: 8 })
      })
    })
    layer("modulo", () => {
      // dos trozos del cilindro central, interior expuesto, costillas, ventanas, quemaduras
      // borde roto: puntos de arriba abajo con picos
      const jag = (x, amp, n, s) => range(n + 1).map(i => {
        const t = clamp(i / n + (i > 0 && i < n ? rand(-0.04, 0.04) : 0), 0, 1)
        const d = Math.sin(t * 6.1 + x) * amp * 0.5 + (i % 2 ? rand(0.2, 1) : rand(-0.6, 0)) * amp
        return [x + d * s, CY - R + t * 2 * R, 0]
      })
      const pieza = (x0, x1, jagL, jagR) => {
        const top = [[x0, CY - R], [x1, CY - R]]
        const der = jagR ? jag(x1, 28, 11, 1) : [[x1, CY - R], [x1 + 22, CY - 50, 1], [x1 + 30, CY], [x1 + 22, CY + 50, 1], [x1, CY + R]]
        const izq = jagL ? jag(x0, 28, 11, -1).reverse() : [[x0, CY + R], [x0 - 22, CY + 50, 1], [x0 - 30, CY], [x0 - 22, CY - 50, 1], [x0, CY - R]]
        return path("M " + [...der, ...izq].map(p => p[0].toFixed(1) + " " + p[1].toFixed(1)).join(" L ") + " Z", { fill: vgrad(P.luz, P.hondo) })
      }
      const cuerpo = (x0, x1, jagL, jagR, ventanas) => {
        const s = pieza(x0, x1, jagL, jagR)
        clip(s, () => {
          // costillas y juntas de placas
          for (let x = x0 + 20; x < x1; x += 44) {
            curve([[x, CY - R], [x + 9, CY], [x, CY + R]], { stroke: P.hondo, width: 3, opacity: 0.35 })
            curve([[x + 3, CY - R], [x + 12, CY], [x + 3, CY + R]], { stroke: P.luz, width: 1.5, opacity: 0.18 })
          }
          // bandas longitudinales
          for (const v of [-62, 40, 74]) line(x0 - 40, CY + v, x1 + 40, CY + v, { stroke: P.hondo, width: 2, opacity: 0.3 })
          // brillo especular amplio y suave en la parte alta
          rect(x0 - 40, CY - R + 14, x1 - x0 + 80, 18, { fill: P.luz, opacity: 0.35, blur: 8 })
          // contraluz frío por abajo
          rect(x0 - 40, CY + R - 12, x1 - x0 + 80, 14, { fill: P.frio, opacity: 0.45, blur: 6 })
          // ventanas: fila apagada, unas pocas encendidas
          for (let x = x0 + 34, k = 0; x < x1 - 20; x += 22, k++) {
            const on = ventanas.includes(k)
            rect(x + 6, CY - 22, 10, 14, { fill: on ? P.ventana : P.hondo, opacity: on ? 0.75 : 0.7, r: 2 })
            if (on) ellipse(x + 11, CY - 15, 16, 14, { fill: P.ventana, opacity: 0.18, blur: 6 })
          }
          // quemaduras oscuras
          for (let i = 0; i < 4; i++) {
            const cx = rand(x0 + 30, x1 - 30), cy = CY + rand(-60, 60)
            blob(roughen(ring(cx, cy, rand(20, 45), rand(14, 30), 9), 8), { fill: P.quema, opacity: 0.45, blur: 6, blend: "multiply" })
          }
        })
        return s
      }
      // interior hueco visto en el corte (detrás de cada borde roto)
      const hueco = (x, dir) => {
        blob(ring(x - dir * 4, CY, 18, R - 8, 10), { fill: mix(P.hondo, P.sombra, 0.4) })
        for (const v of [-40, 0, 40]) line(x - 14, CY + v, x + 10, CY + v, { stroke: P.sombra, width: 4, opacity: 0.8 })
        ellipse(x, CY + 30, 8, 44, { fill: P.frio, opacity: 0.25, blur: 8 })
      }
      hueco(470, 1); hueco(548, -1)
      const A = cuerpo(230, 470, false, true, [3])
      const B = cuerpo(548, 800, true, false, [2, 6])
      // tapas de los extremos
      ellipse(222, CY, 26, R - 4, { fill: linear(0, CY - R, 0, CY + R, [P.luz, P.sombra]), opacity: 0.9 })
      circle(222, CY, 22, { fill: P.sombra, opacity: 0.8 })
      // morro de acoplamiento en el extremo derecho
      blob([[808, CY - 40], [870, CY - 26], [884, CY], [870, CY + 26], [808, CY + 40]], { fill: vgrad(P.luz, P.hondo) })
      rect(880, CY - 10, 40, 20, { fill: vgrad(P.medio, P.hondo), r: 4 })
      line(918, CY, 960, CY - 30, { stroke: P.medio, width: 3, cap: "round" })
    })
    layer("colmena", () => {
      // restos orgánicos verdes en la esquina del trozo derecho
      const masas = [[778, CY + 76, 30, 17], [748, CY + 90, 22, 12], [806, CY + 52, 15, 22], [712, CY + 112, 12, 8]]
      // costra resinosa extendida sobre el casco
      blob(roughen(ring(768, CY + 78, 70, 26, 12, 8), 9), { fill: mix(P.colmena, P.sombra, 0.4), opacity: 0.7, blur: 2 })
      // tentáculos resecos que se agarran al casco
      for (let i = 0; i < 5; i++) {
        const sx = rand(730, 800), sy = CY + rand(60, 95)
        const ex = sx + rand(-90, 10), ey = sy + rand(-120, -50)
        brush([[sx, sy], [lerp(sx, ex, 0.3) + rand(-25, 25), lerp(sy, ey, 0.3)], [lerp(sx, ex, 0.7) + rand(-25, 25), lerp(sy, ey, 0.7)], [ex, ey]], { width: [rand(9, 13), 6, 3], color: mix(P.colmena, P.sombra, 0.5), taper: [0, 0.6] })
      }
      for (const [x, y, rx, ry] of masas) {
        const s = blob(roughen(ring(x, y, rx, ry, 10), rx * 0.18), { fill: radial(x - rx * 0.3, y - ry * 0.5, rx * 1.4, [P.colmenaLuz, P.colmena, mix(P.colmena, P.hondo, 0.6)]) })
        clip(s, () => {
          // celdillas de la colmena
          for (let k = 0; k < 7; k++) ellipse(x + rand(-rx, rx) * 0.7, y + rand(-ry, ry) * 0.7, rand(2.5, 4.5), rand(2, 3.5), { fill: mix(P.colmena, P.hondo, 0.6), opacity: 0.6 })
          blob(ring(x - rx * 0.2, y - ry * 0.4, rx * 0.5, ry * 0.25, 7), { fill: P.colmenaLuz, opacity: 0.35, blur: 4 })
          ellipse(x, y + ry * 0.8, rx, ry * 0.5, { fill: P.frio, opacity: 0.35, blur: 6 })
        })
      }
    })
    layer("restos sueltos", () => {
      // cables colgando del corte entre los dos trozos
      for (let i = 0; i < 6; i++) {
        const y0 = CY + rand(-70, 70), y1 = CY + rand(-70, 70)
        const sag = rand(30, 90)
        const col = pick([P.sombra, P.hondo, mix(P.medio, P.sombra, 0.5)])
        if (i < 3) curve([[470, y0], [505, y0 + sag], [545, y1]], { stroke: col, width: rand(2.5, 4.5), cap: "round" })
        else brush([[480, y0], [505 + rand(-20, 20), y0 + sag], [rand(470, 530), y0 + sag + rand(40, 90)]], { width: rand(2.5, 4), color: col, taper: [0, 0.3] })
      }
      // cable suelto largo a la deriva
      brush([[870, 640], [920, 610], [950, 680], [990, 670]], { width: 5, color: P.sombra, taper: [0.1, 0.5] })
      // vigas de celosía sueltas
      const truss = (x, y, l, ang, w) => group({ rotate: ang, around: [x, y] }, () => {
        rect(x - l / 2, y - w / 2, l, w, { fill: "none", stroke: P.sombra, width: 3 })
        for (let t = 0; t < l; t += w) line(x - l / 2 + t, y - w / 2, x - l / 2 + t + w, y + w / 2, { stroke: P.sombra, width: 2 })
        line(x - l / 2, y - w / 2, x + l / 2, y - w / 2, { stroke: P.medio, width: 2, opacity: 0.6 })
      })
      truss(120, 560, 110, 25, 14)
      truss(560, 280, 90, -40, 12)
      truss(930, 470, 70, 70, 10)
      truss(520, 850, 80, 15, 12)
      // placas de casco arrancadas
      const placa = (x, y, s, ang) => {
        const pts = turn(jitter([[x - s, y - s * 0.6], [x + s, y - s * 0.7], [x + s * 0.8, y + s * 0.6], [x - s * 0.9, y + s * 0.5]], s * 0.2), ang)
        poly(pts, { fill: linear(x, y - s, x, y + s, [P.medio, P.hondo]) })
      }
      placa(470, 300, 18, 20); placa(140, 700, 14, -30); placa(900, 800, 16, 50); placa(600, 940, 12, 10); placa(70, 430, 10, 70)
      // esquirlas pequeñas
      for (let i = 0; i < 18; i++) {
        const a = rand(0, 360), d = rand(300, 440)
        const [x, y] = polar(540, CY, d, a)
        poly(jitter(ring(x, y, rand(3, 7), rand(2, 5), 4, rand(0, 90)), 1.5), { fill: pick([P.sombra, P.medio, P.hondo]), opacity: 0.85 })
      }
    })
  })
})
