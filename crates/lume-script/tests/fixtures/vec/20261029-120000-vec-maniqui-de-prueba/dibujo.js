// Maniquí de prueba para el esqueleto: de perfil, mirando a la derecha.
canvas({ width: 400, height: 480, background: null })
setLight(-1, -1)
const P = { piel: "#e2b48c", lejos: "#b9876a", ropa: "#3f6e8c", ropaLejos: "#2c5068", pelo: "#3a2a22" }
rig({ origin: [200, 455] })

// esqueleto: la articulación de cada hueso, en la pose de reposo
bone("cadera", { at: [200, 268] })
bone("torso", { parent: "cadera", at: [200, 262] })
bone("cabeza", { parent: "torso", at: [206, 150] })
for (const [lado, dx] of [["izq", -6], ["der", 4]]) {
  bone("brazo_" + lado, { parent: "torso", at: [202 + dx, 170] })
  bone("antebrazo_" + lado, { parent: "brazo_" + lado, at: [202 + dx, 232] })
  bone("mano_" + lado, { parent: "antebrazo_" + lado, at: [204 + dx, 290] })
  bone("muslo_" + lado, { parent: "cadera", at: [200 + dx, 272] })
  bone("pierna_" + lado, { parent: "muslo_" + lado, at: [201 + dx, 360] })
  bone("pie_" + lado, { parent: "pierna_" + lado, at: [200 + dx, 440] })
}

// una cápsula entre dos articulaciones, con sombra de cómic y tinta
function capsula(a, b, ancho, color) {
  const f = brush([a, b], { width: ancho, taper: 0, color })
  cel(f, { depth: 0.3 })
  ink(f, { width: 2.2, weight: 2 })
}
function pie(x, y, color) {
  const f = blob([[x - 10, y - 14], [x + 6, y - 12], [x + 30, y - 4, 0.6], [x + 32, y + 2, 0.4], [x - 12, y + 2, 0.4]], { fill: color })
  cel(f, { depth: 0.3 }); ink(f, { width: 2.2 })
}

// piezas, de atrás hacia delante: el lado lejano primero y más oscuro
part("brazo_izq", () => capsula([196, 170], [196, 232], 24, P.ropaLejos))
part("antebrazo_izq", () => capsula([196, 232], [198, 290], 20, P.lejos))
part("mano_izq", () => { const m = circle(199, 298, 12, { fill: P.lejos }); cel(m); ink(m, { width: 2 }) })
part("muslo_izq", () => capsula([194, 272], [195, 360], 30, P.ropaLejos))
part("pierna_izq", () => capsula([195, 360], [194, 440], 24, P.ropaLejos))
part("pie_izq", () => pie(194, 455, "#2b2b2b"))
part("cadera", () => { const c = rect(172, 250, 60, 40, { fill: P.ropa, r: 14 }); cel(c, { depth: 0.25 }); ink(c, { width: 2.4 }) })
part("torso", () => {
  const t = blob([[176, 260], [174, 205], [182, 168], [204, 156], [226, 166], [234, 205], [228, 262, 0.5]], { fill: P.ropa })
  cel(t, { depth: 0.22, highlight: true }); ink(t, { width: 2.6, weight: 2.2 })
})
part("cabeza", () => {
  capsula([206, 150], [207, 132], 18, P.piel)
  // contorno de la cabeza con la nariz de perfil integrada (entre los puntos de 80° y 120°)
  const cr = ring(210, 108, 34, 38, 9)
  const c = blob([...cr.slice(0, 3), [246, 108, 0.6], [256, 120, 0.8], [252, 126, 0.8], [244, 125, 0.2], [243, 131, 0.6], [240, 138, 0.7], ...cr.slice(4)], { fill: P.piel })
  cel(c, { depth: 0.2, highlight: true }); ink(c, { width: 2.6 })
  const pelo = blob([[178, 104], [182, 78], [206, 68], [232, 76], [242, 96], [226, 88], [200, 90], [188, 112, 0]], { fill: P.pelo }); ink(pelo, { width: 2 })
  circle(230, 106, 4, { fill: "#1b1b1b" })
  brush([[245, 122], [249, 119], [251, 122]], { width: 2.5, color: "#7a4a3a" })
})
part("muslo_der", () => capsula([204, 272], [205, 360], 30, P.ropa))
part("pierna_der", () => capsula([205, 360], [204, 440], 24, P.ropa))
part("pie_der", () => pie(204, 455, "#3b3b3b"))
part("brazo_der", () => capsula([206, 170], [206, 232], 24, P.ropa))
part("antebrazo_der", () => capsula([206, 232], [208, 290], 20, P.piel))
part("mano_der", () => { const m = circle(209, 298, 12, { fill: P.piel }); cel(m); ink(m, { width: 2 }) })

// reposo: respira (el pecho sube, la cabeza compensa)
action("reposo", { duration: 2.4, loop: true }, [
  [0, {}],
  [0.5, { cadera: { y: 2 }, torso: -2, cabeza: 2, brazo_der: -3, brazo_izq: -3, antebrazo_der: -4, antebrazo_izq: -4 }],
])

// andar: contacto, bajada, paso y subida, y la otra pierna intercambiando izquierda y derecha.
// La cadera baja lo justo para que el pie de apoyo no se despegue del suelo; el pie avanza
// unos 37 px por clave, así que el personaje recorre 296 px por segundo (sin patinar).
const contacto = { cadera: { y: 8.5 }, torso: -4, cabeza: 3, muslo_der: -26, pierna_der: 4, pie_der: 10, muslo_izq: 20, pierna_izq: 18, pie_izq: -8,
  brazo_der: 22, antebrazo_der: -10, brazo_izq: -24, antebrazo_izq: -22 }
const bajada = { cadera: { y: 11 }, torso: -5, cabeza: 4, muslo_der: -28, pierna_der: 36, pie_der: -8, muslo_izq: 12, pierna_izq: 50, pie_izq: 6,
  brazo_der: 14, antebrazo_der: -8, brazo_izq: -16, antebrazo_izq: -16 }
const paso = { cadera: { y: 0 }, torso: -4, cabeza: 3, muslo_der: 0, pierna_der: 4, pie_der: -2, muslo_izq: -18, pierna_izq: 56, pie_izq: 0,
  brazo_der: 0, antebrazo_der: -6, brazo_izq: 0, antebrazo_izq: -6 }
const subida = { cadera: { y: -7 }, torso: -3, cabeza: 2, muslo_der: 12, pierna_der: 8, pie_der: 10, muslo_izq: -30, pierna_izq: 26, pie_izq: -4,
  brazo_der: -12, antebrazo_der: -14, brazo_izq: 12, antebrazo_izq: -8 }
action("andar", { duration: 1.0, loop: true, move: 296 }, [
  [0, contacto], [0.125, bajada], [0.25, paso], [0.375, subida],
  [0.5, swapSides(contacto)], [0.625, swapSides(bajada)], [0.75, swapSides(paso)], [0.875, swapSides(subida)],
])
