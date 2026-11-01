// Animación "sprite" · celdas de 128x128 · la base empieza en (0, 0) dentro de cada celda.
config({ name: "sprite", base: "base.png", cell: [128, 128], origin: [0, 0], colors: [] });

// Piezas sacadas de la receta del dibujo base, con los mismos comandos.

part("pierna_izq", () => {
  fill_rect(54, 104, 8, 15, "#3b2a2a");
  fill_rect(55, 104, 6, 14, "#f2c29b");
  fill_rect(55, 113, 6, 5, "#f4f4f4");
  line(55, 114, 60, 114, "#5aa0d8");
});

part("pierna_der", () => {
  fill_rect(67, 104, 8, 15, "#3b2a2a");
  fill_rect(68, 104, 6, 14, "#f2c29b");
  fill_rect(68, 113, 6, 5, "#f4f4f4");
  line(68, 114, 73, 114, "#5aa0d8");
});

part("zapato_izq", () => {
  fill_rect(49, 117, 14, 9, "#3b2a2a");
  fill_rect(52, 118, 10, 4, "#c94a3a");
  fill_rect(50, 119, 2, 3, "#c94a3a");
  set_pixel(51, 118, "#c94a3a");
  fill_rect(50, 122, 12, 3, "#f4f4f4");
  set_pixel(49, 117, "transparent");
  set_pixel(49, 118, "transparent");
  set_pixel(50, 117, "transparent");
  set_pixel(49, 125, "transparent");
  set_pixel(62, 125, "transparent");
  line(52, 118, 56, 118, "#e2705e");
  set_pixel(56, 119, "#f4f4f4");
  set_pixel(58, 119, "#f4f4f4");
  line(50, 124, 61, 124, "#b8bcc4");
});

part("zapato_der", () => {
  fill_rect(66, 117, 14, 9, "#3b2a2a");
  fill_rect(67, 118, 10, 4, "#c94a3a");
  fill_rect(77, 119, 2, 3, "#c94a3a");
  set_pixel(77, 118, "#c94a3a");
  fill_rect(67, 122, 12, 3, "#f4f4f4");
  set_pixel(79, 117, "transparent");
  set_pixel(79, 118, "transparent");
  set_pixel(78, 117, "transparent");
  set_pixel(66, 125, "transparent");
  set_pixel(79, 125, "transparent");
  line(67, 118, 71, 118, "#e2705e");
  set_pixel(70, 119, "#f4f4f4");
  set_pixel(72, 119, "#f4f4f4");
  line(67, 124, 78, 124, "#b8bcc4");
});

part("pierna_izq_det", () => {
  line(60, 104, 60, 112, "#d79a76");
  line(55, 105, 55, 111, "#fbdcbf");
  line(60, 113, 60, 117, "#c8ccd4");
});

part("pierna_der_det", () => {
  line(73, 104, 73, 112, "#d79a76");
  line(68, 105, 68, 111, "#fbdcbf");
  line(73, 113, 73, 117, "#c8ccd4");
});

part("brazo_izq", () => {
  fill_rect(46, 74, 7, 19, "#3b2a2a");
  fill_rect(47, 74, 5, 18, "#f2c29b");
  fill_rect(45, 90, 8, 10, "#3b2a2a");
  fill_rect(46, 91, 6, 8, "#f2c29b");
  set_pixel(45, 90, "transparent");
  set_pixel(45, 99, "transparent");
});

part("brazo_der", () => {
  fill_rect(76, 74, 7, 19, "#3b2a2a");
  fill_rect(77, 74, 5, 18, "#f2c29b");
  fill_rect(76, 90, 8, 10, "#3b2a2a");
  fill_rect(77, 91, 6, 8, "#f2c29b");
  set_pixel(83, 90, "transparent");
  set_pixel(83, 99, "transparent");
});

part("brazo_izq_det", () => {
  line(51, 75, 51, 91, "#d79a76");
  line(51, 92, 51, 98, "#d79a76");
  line(47, 92, 47, 96, "#fbdcbf");
  line(47, 90, 50, 90, "#f2c29b");
  set_pixel(51, 90, "#d79a76");
});

part("brazo_der_det", () => {
  line(81, 75, 81, 91, "#d79a76");
  line(82, 91, 82, 98, "#d79a76");
  line(77, 76, 77, 90, "#fbdcbf");
  line(78, 90, 80, 90, "#f2c29b");
  set_pixel(77, 90, "#fbdcbf");
  set_pixel(81, 90, "#d79a76");
});

part("shorts", () => {
  fill_rect(52, 83, 25, 13, "#3b2a2a");
  fill_rect(53, 84, 23, 11, "#d6b27a");
  fill_rect(52, 95, 13, 11, "#3b2a2a");
  fill_rect(53, 95, 11, 10, "#d6b27a");
  fill_rect(64, 95, 13, 11, "#3b2a2a");
  fill_rect(65, 95, 11, 10, "#d6b27a");
});

part("torso", () => {
  fill_rect(52, 60, 25, 26, "#3b2a2a");
  fill_rect(45, 63, 9, 13, "#3b2a2a");
  fill_rect(47, 61, 7, 2, "#3b2a2a");
  fill_rect(75, 63, 9, 13, "#3b2a2a");
  fill_rect(75, 61, 7, 2, "#3b2a2a");
  fill_rect(46, 64, 8, 11, "#5aa0d8");
  fill_rect(48, 62, 6, 2, "#5aa0d8");
  fill_rect(75, 64, 8, 11, "#5aa0d8");
  fill_rect(75, 62, 6, 2, "#5aa0d8");
  fill_rect(53, 61, 23, 24, "#5aa0d8");
  fill_rect(58, 55, 13, 8, "#3b2a2a");
  fill_rect(59, 55, 11, 7, "#d79a76");
  line(58, 62, 70, 62, "#3d78b0");
  line(59, 63, 69, 63, "#3d78b0");
  line(54, 64, 54, 74, "#8cc4ec");
  line(47, 64, 51, 64, "#8cc4ec");
  line(55, 62, 57, 62, "#8cc4ec");
  fill_rect(72, 64, 4, 20, "#3d78b0");
  fill_rect(79, 64, 4, 11, "#3d78b0");
  line(75, 66, 75, 74, "#3d78b0");
  line(46, 74, 53, 74, "#3d78b0");
  line(75, 74, 82, 74, "#3d78b0");
  line(53, 82, 71, 82, "#3d78b0");
  line(59, 60, 69, 60, "#c27f5c");
  line(58, 70, 62, 75, "#5aa0d8");
});

part("shorts_det", () => {
  line(53, 84, 75, 84, "#b08a52");
  line(53, 85, 75, 85, "#c4a066");
  fill_rect(72, 86, 4, 9, "#b08a52");
  fill_rect(72, 95, 4, 10, "#b08a52");
  fill_rect(60, 95, 4, 10, "#b08a52");
  line(64, 87, 64, 95, "#8f6c3c");
  set_pixel(64, 86, "#f0e2c0");
  line(54, 86, 57, 91, "#8f6c3c");
  line(74, 86, 71, 91, "#8f6c3c");
  line(53, 104, 63, 104, "#b08a52");
  line(65, 104, 75, 104, "#b08a52");
});

part("cabeza", () => {
  fill_rect(39, 33, 7, 14, "#3b2a2a");
  fill_rect(40, 34, 5, 12, "#f2c29b");
  set_pixel(39, 33, "transparent");
  set_pixel(39, 46, "transparent");
  fill_rect(84, 33, 7, 14, "#3b2a2a");
  fill_rect(85, 34, 5, 12, "#f2c29b");
  set_pixel(90, 33, "transparent");
  set_pixel(90, 46, "transparent");
  line(42, 37, 42, 42, "#d79a76");
  line(87, 37, 87, 42, "#d79a76");
  fill_rect(44, 19, 42, 32, "#3b2a2a");
  fill_rect(45, 51, 40, 3, "#3b2a2a");
  fill_rect(46, 54, 38, 2, "#3b2a2a");
  fill_rect(48, 56, 34, 2, "#3b2a2a");
  fill_rect(51, 58, 28, 1, "#3b2a2a");
  fill_rect(54, 59, 22, 1, "#3b2a2a");
  fill_rect(45, 20, 40, 30, "#f2c29b");
  fill_rect(46, 50, 38, 3, "#f2c29b");
  fill_rect(47, 53, 36, 2, "#f2c29b");
  fill_rect(49, 55, 32, 2, "#f2c29b");
  fill_rect(52, 57, 26, 1, "#f2c29b");
  fill_rect(55, 58, 20, 1, "#f2c29b");
  fill_rect(46, 4, 38, 6, "#3b2a2a");
  fill_rect(49, 2, 10, 4, "#3b2a2a");
  fill_rect(60, 1, 11, 5, "#3b2a2a");
  fill_rect(71, 2, 10, 4, "#3b2a2a");
  fill_rect(42, 8, 46, 14, "#3b2a2a");
  fill_rect(40, 12, 50, 11, "#3b2a2a");
  fill_rect(40, 21, 8, 12, "#3b2a2a");
  fill_rect(82, 21, 8, 12, "#3b2a2a");
  fill_rect(47, 5, 36, 4, "#7a4a2a");
  fill_rect(50, 3, 8, 2, "#7a4a2a");
  fill_rect(61, 2, 9, 3, "#7a4a2a");
  fill_rect(72, 3, 8, 2, "#7a4a2a");
  fill_rect(43, 9, 44, 12, "#7a4a2a");
  fill_rect(41, 13, 48, 9, "#7a4a2a");
  fill_rect(41, 22, 6, 10, "#7a4a2a");
  fill_rect(47, 21, 6, 1, "#7a4a2a");
  fill_rect(58, 21, 8, 1, "#7a4a2a");
  fill_rect(71, 21, 8, 1, "#7a4a2a");
  set_pixel(40, 12, "transparent");
  set_pixel(89, 12, "transparent");
  set_pixel(42, 8, "transparent");
  set_pixel(87, 8, "transparent");
  set_pixel(46, 4, "transparent");
  set_pixel(83, 4, "transparent");
  fill_rect(50, 25, 10, 2, "#4a2a18");
  line(51, 24, 58, 24, "#4a2a18");
  fill_rect(70, 25, 10, 2, "#4a2a18");
  line(71, 24, 78, 24, "#4a2a18");
  fill_rect(52, 31, 9, 7, "#ffffff");
  line(53, 30, 59, 30, "#ffffff");
  line(53, 38, 59, 38, "#ffffff");
  line(53, 29, 59, 29, "#3b2a2a");
  set_pixel(52, 30, "#3b2a2a");
  set_pixel(60, 30, "#3b2a2a");
  line(51, 31, 51, 37, "#3b2a2a");
  line(61, 31, 61, 37, "#3b2a2a");
  set_pixel(52, 38, "#3b2a2a");
  set_pixel(60, 38, "#3b2a2a");
  line(53, 39, 59, 39, "#3b2a2a");
  fill_rect(55, 31, 5, 6, "#6b3e22");
  fill_rect(56, 32, 3, 4, "#1e1410");
  set_pixel(56, 32, "#ffffff");
  set_pixel(57, 32, "#ffffff");
  set_pixel(58, 35, "#a0704a");
  fill_rect(69, 31, 9, 7, "#ffffff");
  line(70, 30, 76, 30, "#ffffff");
  line(70, 38, 76, 38, "#ffffff");
  line(70, 29, 76, 29, "#3b2a2a");
  set_pixel(69, 30, "#3b2a2a");
  set_pixel(77, 30, "#3b2a2a");
  line(68, 31, 68, 37, "#3b2a2a");
  line(78, 31, 78, 37, "#3b2a2a");
  set_pixel(69, 38, "#3b2a2a");
  set_pixel(77, 38, "#3b2a2a");
  line(70, 39, 76, 39, "#3b2a2a");
  fill_rect(70, 31, 5, 6, "#6b3e22");
  fill_rect(71, 32, 3, 4, "#1e1410");
  set_pixel(71, 32, "#ffffff");
  set_pixel(72, 32, "#ffffff");
  set_pixel(73, 35, "#a0704a");
  line(53, 37, 59, 37, "#e6e6ee");
  line(70, 37, 76, 37, "#e6e6ee");
  fill_rect(62, 40, 5, 3, "#e8b08a");
  set_pixel(63, 40, "#fbdcbf");
  line(62, 43, 66, 43, "#c27f5c");
  set_pixel(61, 42, "#c27f5c");
  set_pixel(67, 42, "#c27f5c");
  set_pixel(63, 42, "#a8603f");
  set_pixel(65, 42, "#a8603f");
  set_pixel(49, 41, "#c47d55");
  set_pixel(51, 42, "#c47d55");
  set_pixel(53, 41, "#c47d55");
  set_pixel(76, 41, "#c47d55");
  set_pixel(78, 42, "#c47d55");
  set_pixel(80, 41, "#c47d55");
  fill_rect(48, 44, 4, 2, "#eda68c");
  fill_rect(78, 44, 4, 2, "#eda68c");
  fill_rect(57, 46, 15, 3, "#6e1f24");
  fill_rect(58, 49, 13, 2, "#6e1f24");
  fill_rect(60, 51, 9, 1, "#6e1f24");
  line(58, 46, 70, 46, "#ffffff");
  fill_rect(61, 49, 7, 2, "#e2717a");
  fill_rect(62, 51, 5, 1, "#e2717a");
  line(56, 45, 72, 45, "#3b2a2a");
  set_pixel(55, 44, "#3b2a2a");
  set_pixel(73, 44, "#3b2a2a");
  line(56, 46, 56, 48, "#3b2a2a");
  line(72, 46, 72, 48, "#3b2a2a");
  line(57, 49, 57, 50, "#3b2a2a");
  line(71, 49, 71, 50, "#3b2a2a");
  set_pixel(58, 51, "#3b2a2a");
  set_pixel(59, 51, "#3b2a2a");
  set_pixel(69, 51, "#3b2a2a");
  set_pixel(70, 51, "#3b2a2a");
  line(60, 52, 68, 52, "#3b2a2a");
  line(45, 22, 84, 22, "#d79a76");
  line(82, 23, 82, 52, "#e0a985");
  line(83, 23, 83, 50, "#d79a76");
  line(84, 23, 84, 49, "#d79a76");
  line(81, 52, 82, 52, "#d79a76");
  line(80, 54, 82, 54, "#d79a76");
  line(78, 56, 80, 56, "#d79a76");
  line(74, 58, 77, 57, "#d79a76");
  line(46, 23, 46, 34, "#fbdcbf");
  line(88, 35, 88, 44, "#d79a76");
  line(89, 35, 89, 44, "#d79a76");
  line(43, 35, 43, 44, "#fbdcbf");
  line(50, 6, 56, 6, "#a86b3c");
  line(46, 10, 54, 10, "#a86b3c");
  line(45, 11, 47, 11, "#a86b3c");
  line(44, 15, 50, 15, "#a86b3c");
  line(57, 9, 63, 9, "#a86b3c");
  line(62, 3, 66, 3, "#a86b3c");
  line(66, 12, 72, 12, "#a86b3c");
  line(42, 18, 45, 18, "#a86b3c");
  line(75, 8, 79, 8, "#a86b3c");
  line(52, 13, 56, 17, "#5a321c");
  line(64, 13, 66, 18, "#5a321c");
  line(76, 12, 78, 18, "#5a321c");
  line(84, 9, 86, 13, "#5a321c");
  fill_rect(85, 14, 4, 8, "#5a321c");
  fill_rect(84, 22, 5, 10, "#5a321c");
  line(43, 19, 52, 20, "#5a321c");
  line(54, 20, 70, 20, "#5a321c");
  line(72, 20, 82, 20, "#5a321c");
  line(42, 24, 42, 30, "#5a321c");
  set_pixel(49, 2, "transparent");
  set_pixel(58, 2, "transparent");
  set_pixel(60, 1, "transparent");
  set_pixel(70, 1, "transparent");
  set_pixel(71, 2, "transparent");
  set_pixel(80, 2, "transparent");
  set_pixel(50, 3, "#3b2a2a");
  set_pixel(57, 3, "#3b2a2a");
  set_pixel(61, 2, "#3b2a2a");
  set_pixel(69, 2, "#3b2a2a");
  set_pixel(72, 3, "#3b2a2a");
  set_pixel(79, 3, "#3b2a2a");
  fill_rect(47, 21, 5, 2, "#7a4a2a");
  line(47, 23, 51, 23, "#3b2a2a");
  set_pixel(52, 22, "#3b2a2a");
  fill_rect(61, 21, 5, 1, "#7a4a2a");
  line(61, 22, 65, 22, "#3b2a2a");
  fill_rect(78, 21, 5, 2, "#7a4a2a");
  line(78, 23, 82, 23, "#3b2a2a");
  set_pixel(77, 22, "#3b2a2a");
  line(42, 20, 42, 24, "#5a321c");
  set_pixel(39, 17, "#3b2a2a");
  set_pixel(39, 18, "#3b2a2a");
  set_pixel(40, 17, "#7a4a2a");
  set_pixel(40, 18, "#7a4a2a");
  set_pixel(90, 17, "#3b2a2a");
  set_pixel(90, 18, "#3b2a2a");
  set_pixel(89, 17, "#5a321c");
  set_pixel(89, 18, "#5a321c");
  set_pixel(40, 26, "transparent");
  set_pixel(41, 26, "#3b2a2a");
  set_pixel(89, 26, "transparent");
  set_pixel(88, 26, "#3b2a2a");
  line(47, 20, 51, 20, "#7a4a2a");
  line(61, 20, 65, 20, "#7a4a2a");
  line(78, 20, 82, 20, "#5a321c");
  line(78, 21, 82, 22, "#5a321c");
});

// Ojos cerrados: tapa los ojos con piel y dibuja la línea del párpado.
part("parpados", () => {
  fill_rect(51, 29, 11, 11, "#f2c29b");
  fill_rect(68, 29, 11, 11, "#f2c29b");
  line(52, 35, 60, 35, "#3b2a2a");
  line(69, 35, 77, 35, "#3b2a2a");
});

// Mechones de arriba (filas 1-4): borra los originales y los redibuja desplazados.
// Con todos los desplazamientos a 0 da exactamente la base.
part("mechones", ({ a = [0, 0], b = [0, 0], c = [0, 0] }) => {
  fill_rect(44, 0, 42, 5, "transparent");
  line(47, 4, 82, 4, "#3b2a2a");
  move(a[0], a[1], () => {
    fill_rect(49, 2, 10, 3, "#3b2a2a");
    fill_rect(50, 3, 8, 2, "#7a4a2a");
    set_pixel(49, 2, "transparent");
    set_pixel(58, 2, "transparent");
    set_pixel(50, 3, "#3b2a2a");
    set_pixel(57, 3, "#3b2a2a");
  });
  move(c[0], c[1], () => {
    fill_rect(71, 2, 10, 3, "#3b2a2a");
    fill_rect(72, 3, 8, 2, "#7a4a2a");
    set_pixel(71, 2, "transparent");
    set_pixel(80, 2, "transparent");
    set_pixel(72, 3, "#3b2a2a");
    set_pixel(79, 3, "#3b2a2a");
  });
  move(b[0], b[1], () => {
    fill_rect(60, 1, 11, 4, "#3b2a2a");
    fill_rect(61, 2, 9, 3, "#7a4a2a");
    line(62, 3, 66, 3, "#a86b3c");
    set_pixel(60, 1, "transparent");
    set_pixel(70, 1, "transparent");
    set_pixel(61, 2, "#3b2a2a");
    set_pixel(69, 2, "#3b2a2a");
  });
});

// Variantes de la boca: 1 = entreabierta, 2 = cerrada sonriendo.
part("boca", ({ forma = 1 }) => {
  fill_rect(55, 44, 19, 9, "#f2c29b");
  if (forma === 1) {
    set_pixel(56, 44, "#3b2a2a");
    set_pixel(72, 44, "#3b2a2a");
    line(57, 45, 71, 45, "#3b2a2a");
    fill_rect(58, 46, 13, 3, "#6e1f24");
    line(58, 46, 70, 46, "#ffffff");
    fill_rect(61, 48, 7, 1, "#e2717a");
    line(57, 46, 57, 48, "#3b2a2a");
    line(71, 46, 71, 48, "#3b2a2a");
    set_pixel(58, 48, "#3b2a2a");
    set_pixel(70, 48, "#3b2a2a");
    line(59, 49, 69, 49, "#3b2a2a");
  } else {
    set_pixel(56, 45, "#3b2a2a");
    set_pixel(72, 45, "#3b2a2a");
    line(57, 46, 58, 46, "#3b2a2a");
    line(70, 46, 71, 46, "#3b2a2a");
    line(59, 47, 69, 47, "#3b2a2a");
  }
});

// Dibuja el personaje con desplazamientos verticales por grupo.
// o.exacto respeta el orden original de capas (solo para la pose de reposo).
function figura(o = {}) {
  const g = (k) => o[k] || 0;
  const c = g("cuerpo");
  const pi = g("pi"), pd = g("pd");
  const bi = c + g("bi"), bd = c + g("bd");
  const cab = c + g("cabeza");
  move(0, pi, () => { draw("pierna_izq"); });
  move(0, pd, () => { draw("pierna_der"); });
  move(0, pi, () => { draw("zapato_izq"); });
  move(0, pd, () => { draw("zapato_der"); });
  if (!o.exacto) {
    move(0, pi, () => { draw("pierna_izq_det"); });
    move(0, pd, () => { draw("pierna_der_det"); });
    move(0, bi, () => { draw("brazo_izq"); draw("brazo_izq_det"); });
    move(0, bd, () => { draw("brazo_der"); draw("brazo_der_det"); });
  } else {
    draw("brazo_izq");
    draw("brazo_der");
  }
  move(0, c, () => { draw("shorts"); draw("torso"); });
  move(0, cab, () => {
    draw("cabeza");
    if (o.ojos_cerrados) draw("parpados");
    if (o.boca) draw("boca", { forma: o.boca });
    if (o.pelo) draw("mechones", o.pelo);
  });
  move(0, c, () => { draw("shorts_det"); });
  if (o.exacto) {
    draw("brazo_izq_det");
    draw("brazo_der_det");
    draw("pierna_izq_det");
    draw("pierna_der_det");
  }
}

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  figura({ exacto: true });
});

// Respiración: el cuerpo baja 1 px y la cabeza lo sigue con un frame de retraso.
// Parpadea en el frame 2, la boca se cierra poco a poco y los mechones se mecen
// con retraso respecto a la cabeza.
action("reposo", { frames: 4, fps: 6, loop: true }, (f, { t }) => {
  figura({
    cuerpo: [0, 1, 1, 0][f],
    cabeza: [0, 0, 0, 1][f] - [0, 1, 0, 0][f],
    ojos_cerrados: f === 2,
    boca: [0, 1, 2, 1][f],
    pelo: [
      null,
      { a: [-1, 0], b: [1, 0], c: [1, 0] },
      { a: [-1, -1], b: [1, -1], c: [2, -1] },
      { a: [0, 0], b: [0, -1], c: [1, 0] },
    ][f],
  });
});

// Paso de frente: cada pie se levanta por turnos, los brazos se balancean
// al contrario y el cuerpo baja en los contactos.
action("andar", { frames: 6, fps: 10, loop: true }, (f, { t }) => {
  const pi = [0, -3, -1, 0, 0, 0][f];
  const pd = [0, 0, 0, 0, -3, -1][f];
  figura({
    cuerpo: [1, 0, 0, 1, 0, 0][f],
    pi,
    pd,
    bi: [0, 1, 1, 0, -1, -1][f],
    bd: [0, -1, -1, 0, 1, 1][f],
  });
});
