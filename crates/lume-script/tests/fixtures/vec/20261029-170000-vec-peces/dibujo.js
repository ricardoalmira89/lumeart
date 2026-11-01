// dibujo.js: peces para cazar bajo el agua (miran a la derecha). Cada uno se exporta por separado
// con margen arriba y abajo: el juego ondula la cola con un shader.
// Sardina: caja 0,0 → 280,100 (el pez, de x 0 a 262, centrado en y 50).
// Dorada: caja 0,120 → 460,380 (de x 0 a 440, centrada en y 250).
canvas({ width: 480, height: 400, background: null })
setLight(-1, -1)

// ---------- sardina: huso estrecho, lomo azul verdoso, flanco plateado, vientre blanco ----------
function sardina() {
  const y = 50
  // cola ahorquillada, detrás del cuerpo
  blob([[46, y - 4], [22, y - 22], [4, y - 34], [12, y - 14], [8, y], [12, y + 14], [4, y + 34], [22, y + 22], [46, y + 4]],
    { fill: linear(0, y - 34, 0, y + 34, [[0, "#3c6d86"], [0.5, "#86b6c4"], [1, "#4f7f96"]]), opacity: 0.92, tension: 0.6 })
  const cuerpo = blob([[262, y], [248, y - 11], [214, y - 21], [160, y - 25], [104, y - 23], [64, y - 15], [40, y - 6],
    [40, y + 6], [64, y + 14], [104, y + 21], [160, y + 23], [214, y + 19], [248, y + 10]],
    { fill: linear(0, y - 25, 0, y + 23, [[0, "#244e66"], [0.32, "#4f8aa0"], [0.5, "#a9d0da"], [0.7, "#dcedf1"], [1, "#f6fbfc"]]) })
  clip(cuerpo, () => {
    // brillo del flanco y una hilera de manchas oscuras tras el opérculo
    brush([[70, y - 4], [140, y - 9], [220, y - 8]], { width: [2, 6, 2], fill: "#ffffff", opacity: 0.35 })
    for (const [i, x] of [196, 182, 168, 154].entries()) circle(x, y - 10 + i * 0.4, 3.2, { fill: "#1d3d52", opacity: 0.55 })
    line(60, y - 3, 236, y - 4, { stroke: "#3c6f86", width: 1.2, opacity: 0.5 })
    grain({ rect: [0, 0, 280, 100], opacity: 0.14 })
  })
  // aletas: dorsal, pectoral y ventral, translúcidas
  blob([[150, y - 23], [132, y - 36], [118, y - 22]], { fill: "#4a7c93", opacity: 0.8, tension: 0.4 })
  blob([[214, y + 6], [196, y + 15], [190, y + 10], [206, y + 4]], { fill: "#9cc4cf", opacity: 0.8, tension: 0.5 })
  blob([[120, y + 21], [106, y + 30], [100, y + 21]], { fill: "#8db7c3", opacity: 0.7, tension: 0.4 })
  // opérculo, boca y ojo
  curve([[226, y - 16], [218, y], [226, y + 15]], { stroke: "#2b5b72", width: 1.6, opacity: 0.7 })
  curve([[262, y], [252, y + 3], [244, y + 3]], { stroke: "#2a4b5c", width: 1.4, opacity: 0.8 })
  circle(238, y - 3, 6.5, { fill: "#e9f2f2" })
  circle(239, y - 3, 4, { fill: "#0f1a22" })
  circle(237.5, y - 4.6, 1.4, { fill: "#ffffff" })
}

// ---------- dorada: cuerpo alto y ovalado, gris plata, banda dorada entre los ojos ----------
function dorada() {
  const y = 250
  blob([[92, y - 4.7], [52, y - 46.8], [16, y - 70.2], [34, y - 23.4], [26, y], [34, y + 23.4], [16, y + 70.2], [52, y + 46.8], [92, y + 4.7]],
    { fill: linear(0, y - 70.2, 0, y + 70.2, [[0, "#48525c"], [0.5, "#7d8b95"], [1, "#4e5963"]]), opacity: 0.95, tension: 0.6 })
  // aleta dorsal larga y anal, detrás del cuerpo
  blob([[350, y - 71.8], [322, y - 90.5], [262, y - 98.3], [196, y - 90.5], [140, y - 67.1], [210, y - 71.8], [290, y - 76.4]],
    { fill: "#55606b", opacity: 0.9, tension: 0.5 })
  blob([[300, y + 68.6], [262, y + 87.4], [208, y + 82.7], [168, y + 60.8]], { fill: "#66727c", opacity: 0.85, tension: 0.5 })
  const cuerpo = blob([[440, y + 1.6], [428, y - 23.4], [398, y - 56.2], [342, y - 78], [270, y - 82.7], [196, y - 71.8],
    [140, y - 46.8], [100, y - 14], [96, y], [100, y + 14], [140, y + 43.7], [196, y + 67.1], [270, y + 76.4], [342, y + 70.2],
    [398, y + 48.4], [428, y + 21.8]],
    { fill: linear(0, y - 82.7, 0, y + 76.4, [[0, "#4d5d69"], [0.3, "#8e9da6"], [0.55, "#c9d3d7"], [0.8, "#e8eded"], [1, "#f4f6f5"]]) })
  clip(cuerpo, () => {
    // rayas longitudinales finas, brillo y la mancha oscura del opérculo
    for (let k = -3; k <= 3; k++)
      curve([[120, y + k * 17], [260, y + k * 20 - 3], [380, y + k * 16 - 5]], { stroke: "#5e6d78", width: 1.6, opacity: 0.28 })
    brush([[150, y - 23.4], [260, y - 35.9], [360, y - 31.2]], { width: [3, 12, 3], fill: "#ffffff", opacity: 0.3 })
    ellipse(362, y - 23.4, 14, 10, { fill: "#2a3038", opacity: 0.6 })
    grain({ rect: [0, 120, 460, 260], opacity: 0.14 })
  })
  // banda dorada sobre la frente y aleta pectoral
  brush([[418, y - 35.9], [400, y - 48.4], [380, y - 54.6]], { width: [4, 9, 3], fill: "#e5b44a", opacity: 0.95 })
  blob([[352, y + 7.8], [318, y + 26.5], [304, y + 20.3], [338, y + 3.1]], { fill: "#9aa6ad", opacity: 0.85, tension: 0.5 })
  // opérculo, boca (labios gruesos) y ojo
  curve([[376, y - 49.9], [362, y - 15.6], [374, y + 23.4]], { stroke: "#4a5761", width: 2.2, opacity: 0.6 })
  curve([[440, y + 1.6], [428, y + 7.8], [414, y + 7.8]], { stroke: "#3a434b", width: 3, opacity: 0.8 })
  circle(402, y - 20.3, 12, { fill: "#d9c690" })
  circle(403, y - 20.3, 8, { fill: "#11161c" })
  circle(400, y - 23, 2.6, { fill: "#ffffff" })
}

sardina()
dorada()
