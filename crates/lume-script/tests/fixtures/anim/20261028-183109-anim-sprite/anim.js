// Animación "sprite" · celdas de 48x48 · la base empieza en (0, 0) dentro de cada celda.
config({ name: "sprite", base: "base.png", cell: [48, 48], origin: [0, 0], colors: ["#9c6b3c", "#5a3a20"] });

// Plataforma y agujero.
part("plataforma", () => {
  fill_rect(4, 26, 40, 18, "#56685f");
  fill_rect(3, 28, 5, 13, "#d9952e");
  fill_rect(40, 28, 5, 13, "#d9952e");
  fill_rect(3, 28, 5, 1, "#f2c063");
  fill_rect(40, 28, 5, 1, "#f2c063");
  fill_rect(3, 40, 5, 1, "#a8681e");
  fill_rect(40, 40, 5, 1, "#a8681e");
  fill_rect(8, 38, 11, 6, "#d9952e");
  fill_rect(29, 38, 11, 6, "#d9952e");
  fill_rect(8, 38, 11, 1, "#f2c063");
  fill_rect(29, 38, 11, 1, "#f2c063");
  fill_rect(3, 41, 9, 4, "#9fb5ad");
  fill_rect(36, 41, 9, 4, "#9fb5ad");
  fill_rect(3, 41, 9, 1, "#c4d4cc");
  fill_rect(36, 41, 9, 1, "#c4d4cc");
  fill_rect(3, 44, 9, 1, "#6f8580");
  fill_rect(36, 44, 9, 1, "#6f8580");
  fill_rect(8, 44, 28, 1, "#a8681e");
  fill_rect(17, 29, 15, 1, "#34403c");
  fill_rect(15, 30, 19, 5, "#1f2624");
  fill_rect(17, 35, 15, 1, "#1f2624");
  fill_rect(15, 30, 19, 1, "#34403c");
});

// Broca. fase 0 = rayas como en la base, fase 1 = rayas invertidas (giro).
part("broca", ({ fase = 0 } = {}) => {
  fill_rect(18, 24, 13, 3, "#9fb5ad");
  fill_rect(19, 27, 11, 3, "#9fb5ad");
  fill_rect(20, 30, 9, 2, "#9fb5ad");
  fill_rect(21, 32, 7, 2, "#9fb5ad");
  fill_rect(22, 34, 5, 2, "#6f8580");
  if (fase === 0) {
    line(18, 26, 30, 24, "#6f8580");
    line(19, 29, 29, 27, "#6f8580");
    line(20, 31, 28, 30, "#6f8580");
    line(21, 33, 27, 32, "#6f8580");
  } else {
    line(18, 24, 30, 26, "#6f8580");
    line(19, 27, 29, 29, "#6f8580");
    line(20, 30, 28, 31, "#6f8580");
    line(21, 32, 27, 33, "#6f8580");
  }
  fill_rect(18, 24, 2, 3, "#c4d4cc");
  fill_rect(19, 27, 2, 3, "#c4d4cc");
  fill_rect(28, 24, 3, 2, "#6f8580");
});

// Motor central (columna con tapa), con su contorno superior.
part("motor", () => {
  fill_rect(18, 10, 13, 14, "#9fb5ad");
  fill_rect(18, 10, 3, 14, "#c4d4cc");
  fill_rect(28, 10, 3, 14, "#6f8580");
  fill_rect(19, 6, 11, 4, "#c4d4cc");
  fill_rect(20, 5, 9, 1, "#c4d4cc");
  fill_rect(19, 9, 11, 1, "#9fb5ad");
  fill_rect(18, 10, 13, 1, "#6f8580");
  fill_rect(18, 6, 3, 3, "#d9952e");
  fill_rect(28, 6, 3, 3, "#a8681e");
  fill_rect(22, 5, 5, 1, "#d9952e");
  fill_rect(18, 6, 3, 1, "#f2c063");
  fill_rect(18, 15, 13, 1, "#6f8580");
  fill_rect(18, 20, 13, 1, "#6f8580");
  fill_rect(22, 16, 5, 7, "#d9952e");
  fill_rect(22, 16, 5, 1, "#f2c063");
  fill_rect(26, 17, 1, 6, "#a8681e");
  fill_rect(23, 18, 3, 1, "#46554f");
  fill_rect(23, 20, 3, 1, "#46554f");
  line(20, 4, 28, 4, "#252b29");
  set_pixel(19, 5, "#252b29");
  set_pixel(29, 5, "#252b29");
  line(17, 6, 17, 9, "#252b29");
  line(31, 6, 31, 9, "#252b29");
});

// Estructura: guías, brazos, patas y bloque frontal.
part("estructura", () => {
  fill_rect(17, 12, 2, 6, "#d9952e");
  fill_rect(30, 12, 2, 6, "#a8681e");
  fill_rect(13, 11, 5, 2, "#9fb5ad");
  fill_rect(13, 11, 5, 1, "#c4d4cc");
  fill_rect(31, 11, 5, 2, "#6f8580");
  line(11, 16, 7, 25, "#9fb5ad");
  line(12, 16, 8, 25, "#9fb5ad");
  line(13, 16, 9, 25, "#6f8580");
  line(10, 16, 6, 25, "#c4d4cc");
  line(37, 16, 41, 25, "#9fb5ad");
  line(36, 16, 40, 25, "#9fb5ad");
  line(35, 16, 39, 25, "#6f8580");
  line(38, 16, 42, 25, "#6f8580");
  fill_rect(10, 10, 5, 5, "#d9952e");
  fill_rect(10, 10, 5, 1, "#f2c063");
  fill_rect(10, 10, 1, 5, "#f2c063");
  fill_rect(11, 11, 3, 3, "#9fb5ad");
  set_pixel(12, 12, "#c4d4cc");
  fill_rect(34, 10, 5, 5, "#d9952e");
  fill_rect(34, 10, 5, 1, "#f2c063");
  fill_rect(38, 10, 1, 5, "#a8681e");
  fill_rect(35, 11, 3, 3, "#9fb5ad");
  set_pixel(36, 12, "#6f8580");
  fill_rect(5, 24, 5, 3, "#9fb5ad");
  fill_rect(5, 24, 5, 1, "#c4d4cc");
  set_pixel(7, 25, "#46554f");
  fill_rect(39, 24, 5, 3, "#9fb5ad");
  fill_rect(39, 24, 5, 1, "#c4d4cc");
  set_pixel(41, 25, "#46554f");
  fill_rect(14, 20, 4, 2, "#9fb5ad");
  fill_rect(14, 20, 4, 1, "#c4d4cc");
  fill_rect(31, 20, 4, 2, "#6f8580");
  line(12, 25, 10, 38, "#9fb5ad");
  line(13, 25, 11, 38, "#9fb5ad");
  line(14, 25, 12, 38, "#6f8580");
  line(11, 25, 9, 38, "#c4d4cc");
  line(36, 25, 38, 38, "#9fb5ad");
  line(35, 25, 37, 38, "#9fb5ad");
  line(34, 25, 36, 38, "#6f8580");
  line(37, 25, 39, 38, "#6f8580");
  line(10, 31, 13, 31, "#6f8580");
  line(35, 31, 38, 31, "#46554f");
  fill_rect(10, 19, 5, 5, "#d9952e");
  fill_rect(10, 19, 5, 1, "#f2c063");
  fill_rect(10, 19, 1, 5, "#f2c063");
  fill_rect(11, 20, 3, 3, "#9fb5ad");
  set_pixel(12, 21, "#c4d4cc");
  fill_rect(34, 19, 5, 5, "#d9952e");
  fill_rect(34, 19, 5, 1, "#f2c063");
  fill_rect(38, 19, 1, 5, "#a8681e");
  fill_rect(35, 20, 3, 3, "#9fb5ad");
  set_pixel(36, 21, "#6f8580");
  fill_rect(7, 37, 7, 4, "#9fb5ad");
  fill_rect(7, 37, 7, 1, "#c4d4cc");
  fill_rect(7, 40, 7, 1, "#6f8580");
  set_pixel(10, 38, "#46554f");
  fill_rect(35, 37, 7, 4, "#9fb5ad");
  fill_rect(35, 37, 7, 1, "#c4d4cc");
  fill_rect(35, 40, 7, 1, "#6f8580");
  set_pixel(38, 38, "#46554f");
  fill_rect(19, 37, 11, 8, "#d9952e");
  fill_rect(19, 37, 1, 8, "#f2c063");
  fill_rect(29, 37, 1, 8, "#a8681e");
  fill_rect(21, 40, 7, 4, "#f2c063");
  fill_rect(22, 41, 5, 3, "#1f2624");
});

// Contorno de la estructura y la plataforma.
part("contorno", () => {
  line(10, 9, 14, 9, "#252b29");
  line(34, 9, 38, 9, "#252b29");
  line(9, 10, 9, 14, "#252b29");
  line(39, 10, 39, 14, "#252b29");
  line(9, 15, 5, 23, "#252b29");
  line(39, 15, 43, 23, "#252b29");
  line(4, 24, 4, 27, "#252b29");
  line(44, 24, 44, 27, "#252b29");
  line(2, 28, 2, 44, "#252b29");
  line(45, 28, 45, 44, "#252b29");
  line(3, 27, 3, 27, "#252b29");
  set_pixel(44, 27, "#252b29");
  line(3, 45, 44, 45, "#252b29");
  line(9, 19, 9, 23, "#252b29");
  line(39, 19, 39, 23, "#252b29");
  line(15, 24, 17, 24, "#252b29");
  line(31, 24, 33, 24, "#252b29");
  line(10, 25, 13, 25, "#252b29");
  line(35, 25, 38, 25, "#252b29");
  fill_rect(19, 37, 11, 1, "#f2c063");
  line(18, 37, 18, 44, "#252b29");
  line(30, 37, 30, 44, "#252b29");
  line(19, 36, 29, 36, "#252b29");
  set_pixel(21, 40, "#d9952e");
  set_pixel(27, 40, "#d9952e");
});

// Tierra que sale del agujero en arco, a ambos lados.
part("tierra", ({ f }) => {
  for (let i = 0; i < 6; i++) {
    const edad = (f + i * 3) % 6;
    const lado = i % 2 === 0 ? -1 : 1;
    const vx = 2 + (i % 2);
    const x = (lado < 0 ? 17 : 31) + lado * vx * edad;
    const y = 29 - 4 * edad + edad * edad;
    const c = i % 3 === 0 ? "#5a3a20" : "#9c6b3c";
    fill_rect(x, y, edad < 4 ? 2 : 1, edad < 4 ? 2 : 1, c);
  }
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("plataforma");
  draw("broca");
  draw("motor");
  draw("estructura");
  draw("contorno");
});

// Sube (anticipación), baja de golpe, taladra vibrando y soltando tierra, y vuelve.
action("anim", { frames: 16, fps: 16, loop: true }, (f, { t }) => {
  const dy = cycle([0, -1, -2, -2, -1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 0, 0], f);
  const taladrando = f >= 5 && f <= 13;
  const vib = taladrando ? (f % 2 === 0 ? 0 : 1) : 0;
  draw("plataforma");
  move(0, dy + vib, () => {
    draw("broca", { fase: f % 2 });
    draw("motor");
  });
  draw("estructura");
  draw("contorno");
  if (taladrando) draw("tierra", { f });
});
