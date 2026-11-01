// Animación "sprite" · celdas de 32x64 · la base empieza en (0, 0) dentro de cada celda.
config({ name: "sprite", base: "base.png", cell: [32, 64], origin: [0, 0], colors: [] });

// Piezas del esqueleto, en el mismo orden de la receta original.
part("piernas", () => {
  line(10, 30, 7, 44, "#1e1d1a");
  line(13, 30, 10, 44, "#1e1d1a");
  line(11, 30, 8, 44, "#7d7a52");
  line(12, 30, 9, 44, "#57553a");
  fill_rect(6, 43, 5, 4, "#1e1d1a");
  fill_rect(7, 44, 3, 2, "#7d7a52");
  line(6, 46, 5, 56, "#1e1d1a");
  line(9, 46, 8, 56, "#1e1d1a");
  line(7, 46, 6, 56, "#7d7a52");
  line(8, 46, 7, 56, "#57553a");
  fill_rect(1, 56, 10, 4, "#1e1d1a");
  fill_rect(3, 57, 7, 2, "#7d7a52");
  line(2, 58, 8, 58, "#57553a");
  line(16, 30, 20, 43, "#1e1d1a");
  line(17, 30, 21, 43, "#7d7a52");
  line(18, 30, 22, 43, "#57553a");
  fill_rect(19, 43, 5, 4, "#1e1d1a");
  fill_rect(20, 44, 3, 2, "#7d7a52");
  line(20, 46, 21, 56, "#1e1d1a");
  line(23, 46, 24, 56, "#1e1d1a");
  line(21, 46, 22, 56, "#7d7a52");
  line(22, 46, 23, 56, "#57553a");
  fill_rect(20, 56, 10, 4, "#1e1d1a");
  fill_rect(21, 57, 7, 2, "#7d7a52");
  line(21, 58, 28, 58, "#57553a");
});

part("escudo", () => {
  fill_rect(23, 21, 4, 1, "#1e1d1a");
  fill_rect(21, 22, 8, 1, "#1e1d1a");
  fill_rect(20, 23, 10, 1, "#1e1d1a");
  fill_rect(19, 24, 12, 16, "#1e1d1a");
  fill_rect(20, 40, 10, 1, "#1e1d1a");
  fill_rect(21, 41, 8, 1, "#1e1d1a");
  fill_rect(23, 42, 4, 1, "#1e1d1a");
  fill_rect(23, 22, 4, 1, "#4a4a48");
  fill_rect(21, 23, 8, 1, "#4a4a48");
  fill_rect(20, 24, 10, 16, "#4a4a48");
  fill_rect(21, 40, 8, 1, "#4a4a48");
  fill_rect(23, 41, 4, 1, "#4a4a48");
  line(23, 23, 23, 40, "#2f2f30");
  line(26, 23, 26, 41, "#2f2f30");
  line(20, 24, 20, 39, "#2f2f30");
  line(25, 22, 26, 22, "#6a6a66");
  line(27, 23, 28, 23, "#6a6a66");
  line(29, 24, 29, 31, "#6a6a66");
  line(28, 24, 28, 26, "#6a6a66");
  line(24, 28, 27, 36, "#2f2f30");
  line(25, 28, 28, 36, "#6a6a66");
});

part("pelvis", () => {
  fill_rect(14, 21, 4, 4, "#1e1d1a");
  fill_rect(9, 24, 11, 5, "#1e1d1a");
  fill_rect(10, 25, 4, 2, "#7d7a52");
  fill_rect(16, 25, 3, 2, "#57553a");
  fill_rect(11, 27, 7, 1, "#7d7a52");
  set_pixel(10, 25, "#a6a272");
  fill_rect(11, 28, 9, 11, "#1e1d1a");
  fill_rect(12, 28, 7, 10, "#4a4a48");
  fill_rect(15, 38, 2, 3, "#1e1d1a");
  fill_rect(18, 38, 2, 2, "#1e1d1a");
  set_pixel(18, 38, "#4a4a48");
  line(12, 28, 18, 28, "#6a6a66");
  line(17, 29, 17, 36, "#2f2f30");
  set_pixel(13, 32, "#2f2f30");
});

part("cabeza", () => {
  fill_rect(9, 11, 12, 12, "#1e1d1a");
  fill_rect(14, 9, 4, 3, "#1e1d1a");
  fill_rect(14, 1, 3, 1, "#1e1d1a");
  fill_rect(13, 2, 5, 1, "#1e1d1a");
  fill_rect(12, 3, 7, 1, "#1e1d1a");
  fill_rect(11, 4, 9, 4, "#1e1d1a");
  fill_rect(12, 8, 8, 1, "#1e1d1a");
  fill_rect(13, 9, 7, 1, "#1e1d1a");
  fill_rect(14, 10, 6, 1, "#1e1d1a");
  fill_rect(14, 2, 3, 1, "#7d7a52");
  fill_rect(13, 3, 5, 1, "#7d7a52");
  fill_rect(12, 4, 7, 4, "#7d7a52");
  fill_rect(13, 8, 6, 1, "#7d7a52");
  fill_rect(15, 9, 4, 1, "#7d7a52");
  set_pixel(14, 2, "#d8d3a0");
  set_pixel(15, 2, "#d8d3a0");
  set_pixel(13, 3, "#d8d3a0");
  set_pixel(14, 3, "#a6a272");
  set_pixel(12, 4, "#a6a272");
  set_pixel(13, 4, "#a6a272");
  set_pixel(12, 5, "#a6a272");
  line(12, 7, 13, 8, "#57553a");
  fill_rect(15, 5, 2, 2, "#1e1d1a");
  set_pixel(16, 7, "#2f2f30");
  set_pixel(18, 7, "#1e1d1a");
  set_pixel(18, 5, "#57553a");
  set_pixel(18, 6, "#57553a");
  set_pixel(16, 9, "#2f2f30");
  set_pixel(18, 9, "#2f2f30");
  line(15, 8, 18, 8, "#57553a");
});

part("brazo_escudo", () => {
  line(19, 12, 20, 24, "#1e1d1a");
  line(22, 12, 23, 24, "#1e1d1a");
  line(20, 12, 21, 24, "#7d7a52");
  line(21, 12, 22, 24, "#57553a");
  fill_rect(19, 24, 5, 3, "#1e1d1a");
  fill_rect(20, 25, 3, 1, "#7d7a52");
  line(19, 26, 20, 31, "#1e1d1a");
  line(22, 26, 23, 31, "#1e1d1a");
  line(20, 26, 21, 31, "#7d7a52");
  line(21, 26, 22, 31, "#57553a");
  fill_rect(19, 31, 6, 5, "#1e1d1a");
  fill_rect(20, 32, 4, 3, "#7d7a52");
  line(21, 33, 23, 33, "#57553a");
  set_pixel(20, 32, "#a6a272");
});

part("brazo_espada", () => {
  line(6, 16, 4, 25, "#1e1d1a");
  line(9, 16, 7, 25, "#1e1d1a");
  line(7, 16, 5, 25, "#7d7a52");
  line(8, 16, 6, 25, "#57553a");
  fill_rect(3, 24, 5, 3, "#1e1d1a");
  fill_rect(4, 25, 3, 1, "#7d7a52");
  line(3, 26, 3, 31, "#1e1d1a");
  line(6, 26, 6, 31, "#1e1d1a");
  line(4, 26, 4, 31, "#7d7a52");
  line(5, 26, 5, 31, "#57553a");
  line(1, 28, 6, 33, "#1e1d1a");
  line(0, 29, 5, 34, "#1e1d1a");
  line(1, 29, 5, 33, "#4a4a48");
  set_pixel(0, 28, "#2f2f30");
});

part("hoja_contorno", () => {
  line(9, 34, 20, 45, "#1e1d1a");
  line(9, 35, 19, 45, "#1e1d1a");
  line(9, 36, 18, 45, "#1e1d1a");
  line(9, 37, 17, 45, "#1e1d1a");
  line(13, 38, 14, 37, "#c9b07a");
  line(15, 40, 16, 39, "#c9b07a");
});

part("mano", () => {
  line(10, 31, 6, 35, "#1e1d1a");
  line(11, 31, 7, 36, "#1e1d1a");
  line(10, 32, 7, 35, "#4a4a48");
  fill_rect(2, 30, 6, 6, "#1e1d1a");
  fill_rect(3, 31, 4, 4, "#7d7a52");
  line(4, 32, 6, 32, "#57553a");
  line(4, 34, 6, 34, "#57553a");
  set_pixel(3, 31, "#a6a272");
});

part("hombrera_contorno", () => {
  fill_rect(6, 8, 6, 1, "#1e1d1a");
  fill_rect(5, 9, 8, 8, "#1e1d1a");
  fill_rect(6, 17, 6, 1, "#1e1d1a");
});

part("costillas", () => {
  fill_rect(10, 12, 10, 10, "#2f2f30");
  line(11, 13, 18, 13, "#7d7a52");
  line(10, 15, 19, 15, "#7d7a52");
  line(10, 17, 19, 17, "#7d7a52");
  line(11, 19, 18, 19, "#7d7a52");
  line(12, 21, 17, 21, "#7d7a52");
  line(10, 15, 12, 15, "#a6a272");
  line(10, 17, 12, 17, "#a6a272");
  line(11, 19, 12, 19, "#a6a272");
  line(17, 13, 18, 13, "#57553a");
  line(17, 15, 19, 15, "#57553a");
  line(17, 17, 19, 17, "#57553a");
  line(17, 19, 18, 19, "#57553a");
  line(15, 12, 15, 20, "#a6a272");
  line(16, 12, 16, 20, "#57553a");
  line(12, 12, 19, 19, "#4a4a48");
  line(12, 13, 19, 20, "#4a4a48");
  line(12, 12, 14, 14, "#6a6a66");
});

part("hombrera", () => {
  fill_rect(6, 9, 6, 8, "#4a4a48");
  line(6, 9, 11, 9, "#6a6a66");
  line(6, 9, 6, 15, "#6a6a66");
  line(11, 10, 11, 16, "#2f2f30");
  line(7, 16, 10, 16, "#2f2f30");
  line(9, 10, 9, 14, "#c9b07a");
  line(7, 12, 10, 12, "#c9b07a");
  set_pixel(9, 10, "#e3cfa0");
  set_pixel(7, 12, "#e3cfa0");
  set_pixel(10, 10, "#c9b07a");
});

part("hoja", () => {
  line(10, 34, 19, 43, "#6a6a66");
  line(10, 35, 18, 43, "#4a4a48");
  line(10, 36, 17, 43, "#4a4a48");
  line(11, 37, 16, 42, "#2f2f30");
  line(12, 37, 14, 35, "#c9b07a");
  line(14, 39, 16, 37, "#c9b07a");
  fill_rect(17, 41, 2, 2, "#c9b07a");
  set_pixel(18, 40, "#e3cfa0");
  set_pixel(19, 42, "#e3cfa0");
  set_pixel(18, 43, "#c9b07a");
});

part("hombrera_brillo", () => {
  set_pixel(6, 14, "#6a6a66");
});

// Esqueleto completo: el tronco se inclina (cuerpo) y el brazo con la espada se desplaza (brazo).
part("esqueleto", ({ cuerpo = [0, 0], hombro = [0, 0], brazo = [0, 0] } = {}) => {
  draw("piernas");
  move(cuerpo[0], cuerpo[1], () => {
    draw("escudo");
    draw("pelvis");
    draw("cabeza");
    draw("brazo_escudo");
  });
  move(brazo[0], brazo[1], () => {
    draw("brazo_espada");
    draw("hoja_contorno");
    draw("mano");
  });
  move(hombro[0], hombro[1], () => draw("hombrera_contorno"));
  move(cuerpo[0], cuerpo[1], () => draw("costillas"));
  move(hombro[0], hombro[1], () => draw("hombrera"));
  move(brazo[0], brazo[1], () => draw("hoja"));
  move(hombro[0], hombro[1], () => draw("hombrera_brillo"));
});

// Destello en la punta de la espada.
part("chispa", ({ x, y }) => {
  set_pixel(x, y, "#e3cfa0");
  set_pixel(x - 1, y, "#c9b07a");
  set_pixel(x + 1, y, "#c9b07a");
  set_pixel(x, y - 1, "#c9b07a");
  set_pixel(x, y + 1, "#c9b07a");
  set_pixel(x + 2, y + 2, "#d8d3a0");
  set_pixel(x - 2, y - 2, "#d8d3a0");
  set_pixel(x + 2, y - 2, "#d8d3a0");
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("esqueleto");
});

// Estocada: recoge el brazo hacia atrás, lanza la espada en diagonal y vuelve.
action("anim", { frames: 8, fps: 8, loop: true }, (f, { t }) => {
  const poses = [
    { cuerpo: [0, 0], hombro: [0, 0], brazo: [0, 0] },
    { cuerpo: [-1, 0], hombro: [0, -1], brazo: [0, -1] },
    { cuerpo: [-1, 1], hombro: [0, -1], brazo: [0, -2] },
    { cuerpo: [1, 0], hombro: [1, 0], brazo: [3, 1] },
    { cuerpo: [1, 0], hombro: [1, 1], brazo: [4, 2] },
    { cuerpo: [1, 0], hombro: [1, 0], brazo: [3, 2] },
    { cuerpo: [0, 0], hombro: [0, 0], brazo: [1, 1] },
    { cuerpo: [0, 0], hombro: [0, 0], brazo: [0, 0] },
  ];
  const p = poses[f];
  draw("esqueleto", p);
  if (f === 4) draw("chispa", { x: 21 + p.brazo[0], y: 46 + p.brazo[1] });
});
