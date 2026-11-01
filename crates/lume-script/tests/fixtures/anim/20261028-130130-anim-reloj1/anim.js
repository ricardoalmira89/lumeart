// Animación "reloj1" · celdas de 32x32 · la base empieza en (0, 0) dentro de cada celda.
config({ name: "reloj1", base: "base.png", cell: [32, 32], origin: [0, 0], colors: [] });

// Receta del dibujo base, reconstruida a partir de sus píxeles (rectángulos por color).
// Repártela en piezas con nombre para poder moverlas. El orden de dibujo hace de capas:
// lo que se dibuja después queda encima.
part("base", () => {
  fill_rect(5, 0, 22, 1, "#3a2210");
  fill_rect(4, 1, 1, 4, "#3a2210");
  set_pixel(5, 1, "#c98a45");
  fill_rect(6, 1, 7, 1, "#e8b46a");
  fill_rect(13, 1, 14, 1, "#c98a45");
  fill_rect(27, 1, 1, 4, "#3a2210");
  fill_rect(5, 2, 22, 2, "#9a5a24");
  fill_rect(5, 4, 1, 4, "#3a2210");
  fill_rect(6, 4, 20, 1, "#5a3416");
  fill_rect(26, 4, 1, 4, "#3a2210");
  fill_rect(6, 5, 4, 1, "#3a2210");
  set_pixel(10, 5, "#313131cc");
  fill_rect(11, 5, 11, 1, "#243238aa");
  fill_rect(22, 5, 4, 1, "#3a2210");
  fill_rect(6, 6, 1, 2, "#c98a45");
  fill_rect(7, 6, 1, 2, "#9a5a24");
  fill_rect(8, 6, 1, 2, "#3a2210");
  fill_rect(9, 6, 1, 5, "#8fa4ab");
  fill_rect(10, 6, 2, 1, "#ffffff");
  fill_rect(12, 6, 8, 2, "#f6d27a");
  fill_rect(20, 6, 2, 1, "#e3eef1");
  fill_rect(22, 6, 1, 5, "#8fa4ab");
  fill_rect(23, 6, 1, 2, "#3a2210");
  fill_rect(24, 6, 1, 3, "#c98a45");
  fill_rect(25, 6, 1, 2, "#9a5a24");
  fill_rect(10, 7, 2, 1, "#f6d27a");
  fill_rect(20, 7, 2, 1, "#f6d27a");
  fill_rect(4, 8, 1, 2, "#3a2210");
  fill_rect(5, 8, 3, 1, "#e8b46a");
  set_pixel(8, 8, "#9a5a24");
  fill_rect(10, 8, 10, 2, "#e0a640");
  fill_rect(20, 8, 2, 1, "#b87a28");
  set_pixel(23, 8, "#c98a45");
  set_pixel(25, 8, "#c98a45");
  set_pixel(26, 8, "#9a5a24");
  fill_rect(27, 8, 1, 2, "#3a2210");
  set_pixel(5, 9, "#9a5a24");
  fill_rect(6, 9, 3, 1, "#5a3416");
  fill_rect(20, 9, 2, 1, "#e0a640");
  set_pixel(23, 9, "#9a5a24");
  fill_rect(24, 9, 3, 1, "#5a3416");
  fill_rect(5, 10, 1, 5, "#3a2210");
  fill_rect(6, 10, 1, 5, "#c98a45");
  fill_rect(7, 10, 1, 5, "#5a3416");
  fill_rect(8, 10, 1, 5, "#3a2210");
  fill_rect(10, 10, 2, 1, "#f6d27a");
  fill_rect(12, 10, 9, 1, "#e0a640");
  set_pixel(21, 10, "#b87a28");
  fill_rect(23, 10, 1, 5, "#3a2210");
  fill_rect(24, 10, 1, 6, "#c98a45");
  fill_rect(25, 10, 1, 5, "#5a3416");
  fill_rect(26, 10, 1, 5, "#3a2210");
  set_pixel(10, 11, "#8fa4ab");
  fill_rect(11, 11, 9, 1, "#e0a640");
  set_pixel(20, 11, "#b87a28");
  set_pixel(21, 11, "#8fa4ab");
  set_pixel(11, 12, "#8fa4ab");
  fill_rect(12, 12, 7, 1, "#e0a640");
  set_pixel(19, 12, "#b87a28");
  set_pixel(20, 12, "#8fa4ab");
  set_pixel(12, 13, "#8fa4ab");
  fill_rect(13, 13, 5, 1, "#e0a640");
  set_pixel(18, 13, "#b87a28");
  set_pixel(19, 13, "#8fa4ab");
  set_pixel(13, 14, "#8fa4ab");
  fill_rect(14, 14, 3, 1, "#e0a640");
  set_pixel(17, 14, "#b87a28");
  set_pixel(18, 14, "#8fa4ab");
  fill_rect(4, 15, 1, 2, "#3a2210");
  fill_rect(5, 15, 3, 1, "#e8b46a");
  set_pixel(8, 15, "#9a5a24");
  fill_rect(14, 15, 1, 2, "#8fa4ab");
  fill_rect(15, 15, 1, 2, "#e0a640");
  set_pixel(16, 15, "#b87a28");
  fill_rect(17, 15, 1, 2, "#8fa4ab");
  set_pixel(23, 15, "#c98a45");
  set_pixel(25, 15, "#c98a45");
  set_pixel(26, 15, "#9a5a24");
  fill_rect(27, 15, 1, 2, "#3a2210");
  set_pixel(5, 16, "#9a5a24");
  fill_rect(6, 16, 3, 1, "#5a3416");
  set_pixel(23, 16, "#9a5a24");
  fill_rect(24, 16, 3, 1, "#5a3416");
  fill_rect(5, 17, 1, 5, "#3a2210");
  fill_rect(6, 17, 1, 5, "#c98a45");
  fill_rect(7, 17, 1, 5, "#5a3416");
  fill_rect(8, 17, 1, 5, "#3a2210");
  set_pixel(13, 17, "#8fa4ab");
  fill_rect(14, 17, 2, 9, "#e3eef1");
  fill_rect(17, 17, 1, 9, "#e3eef1");
  set_pixel(18, 17, "#8fa4ab");
  fill_rect(23, 17, 1, 5, "#3a2210");
  fill_rect(24, 17, 1, 6, "#c98a45");
  fill_rect(25, 17, 1, 5, "#5a3416");
  fill_rect(26, 17, 1, 5, "#3a2210");
  set_pixel(12, 18, "#8fa4ab");
  fill_rect(13, 18, 1, 8, "#e3eef1");
  fill_rect(18, 18, 1, 8, "#e3eef1");
  set_pixel(19, 18, "#8fa4ab");
  set_pixel(11, 19, "#8fa4ab");
  set_pixel(12, 19, "#e3eef1");
  fill_rect(19, 19, 1, 7, "#e3eef1");
  set_pixel(20, 19, "#8fa4ab");
  set_pixel(10, 20, "#8fa4ab");
  set_pixel(11, 20, "#e3eef1");
  set_pixel(12, 20, "#ffffff");
  fill_rect(20, 20, 1, 6, "#e3eef1");
  set_pixel(21, 20, "#8fa4ab");
  fill_rect(9, 21, 1, 5, "#8fa4ab");
  set_pixel(10, 21, "#e3eef1");
  set_pixel(11, 21, "#ffffff");
  fill_rect(12, 21, 1, 5, "#e3eef1");
  fill_rect(21, 21, 1, 5, "#e3eef1");
  fill_rect(22, 21, 1, 5, "#8fa4ab");
  fill_rect(4, 22, 1, 2, "#3a2210");
  fill_rect(5, 22, 3, 1, "#e8b46a");
  set_pixel(8, 22, "#9a5a24");
  fill_rect(10, 22, 1, 4, "#ffffff");
  fill_rect(11, 22, 1, 4, "#e3eef1");
  set_pixel(23, 22, "#c98a45");
  set_pixel(25, 22, "#c98a45");
  set_pixel(26, 22, "#9a5a24");
  fill_rect(27, 22, 1, 2, "#3a2210");
  set_pixel(5, 23, "#9a5a24");
  fill_rect(6, 23, 3, 1, "#5a3416");
  set_pixel(23, 23, "#9a5a24");
  fill_rect(24, 23, 3, 1, "#5a3416");
  fill_rect(5, 24, 1, 4, "#3a2210");
  fill_rect(6, 24, 1, 2, "#c98a45");
  fill_rect(7, 24, 1, 2, "#9a5a24");
  fill_rect(8, 24, 1, 3, "#3a2210");
  fill_rect(23, 24, 1, 3, "#3a2210");
  fill_rect(24, 24, 1, 2, "#c98a45");
  fill_rect(25, 24, 1, 2, "#9a5a24");
  fill_rect(26, 24, 1, 4, "#3a2210");
  fill_rect(6, 26, 2, 1, "#3a2210");
  set_pixel(9, 26, "#3a2210");
  fill_rect(10, 26, 5, 1, "#243238aa");
  set_pixel(15, 26, "#e0a640");
  set_pixel(17, 26, "#e0a640");
  fill_rect(18, 26, 4, 1, "#243238aa");
  set_pixel(22, 26, "#3a2210");
  fill_rect(24, 26, 2, 1, "#3a2210");
  fill_rect(4, 27, 1, 4, "#3a2210");
  fill_rect(6, 27, 7, 1, "#e8b46a");
  fill_rect(13, 27, 13, 1, "#c98a45");
  fill_rect(27, 27, 1, 4, "#3a2210");
  fill_rect(5, 28, 22, 2, "#9a5a24");
  fill_rect(5, 30, 22, 1, "#5a3416");
  fill_rect(5, 31, 22, 1, "#3a2210");
});

// Chorro de arena entre los dos bulbos. Sin fase es el chorro liso de la base;
// con fase, los granos bajan 1 px por frame y en los frames impares se abren huecos.
part("chorro", ({ fase } = {}) => {
  fill_rect(16, 16, 1, 11, "#e0a640");
  if (fase === undefined) return;
  for (let y = 16; y <= 25; y++) {
    const k = (((y - fase) % 4) + 4) % 4;
    if (k === 0) set_pixel(16, y, "#f6d27a");
    else if (k === 3) set_pixel(16, y, fase % 2 ? "#e3eef1" : "#b87a28");
  }
});

// Granos que resbalan por la superficie hacia el cuello (paso de 0 a 3).
part("granos", ({ paso }) => {
  set_pixel(11 + paso, 7, "#b87a28");
  set_pixel(20 - paso, 7, "#b87a28");
});

// Salpicadura al pie del chorro.
part("salpica", ({ alto }) => {
  if (alto > 0) {
    set_pixel(15, 25, "#f6d27a");
    set_pixel(17, 25, "#f6d27a");
  }
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("base");
  draw("chorro");
});

// Una función por acción: dibuja el frame f desde cero. t va de 0 a 1.
action("cayendo", { frames: 8, fps: 8, loop: true }, (f, { t }) => {
  draw("base");
  draw("chorro", { fase: f });
  draw("granos", { paso: Math.floor(f / 2) });
  draw("salpica", { alto: f % 4 === 1 ? 1 : 0 });
});

