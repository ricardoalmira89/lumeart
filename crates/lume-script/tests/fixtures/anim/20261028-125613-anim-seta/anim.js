// Animación "seta" · celdas de 40x40 · la base empieza en (4, 8) dentro de cada celda.
config({ name: "seta", base: "base.png", cell: [40, 40], origin: [4, 8], colors: [] });

// Receta del dibujo base, reconstruida a partir de sus píxeles (rectángulos por color).
// Repártela en piezas con nombre para poder moverlas. El orden de dibujo hace de capas:
// lo que se dibuja después queda encima.
part("cap", () => {
  fill_rect(11, 5, 10, 1, "#3b1f26");
  fill_rect(8, 6, 3, 1, "#3b1f26");
  fill_rect(11, 6, 6, 1, "#ef5b5b");
  fill_rect(17, 6, 4, 2, "#d62f3a");
  fill_rect(21, 6, 3, 1, "#3b1f26");
  fill_rect(6, 7, 2, 1, "#3b1f26");
  fill_rect(8, 7, 6, 1, "#ef5b5b");
  fill_rect(14, 7, 3, 7, "#d62f3a");
  fill_rect(21, 7, 3, 1, "#d62f3a");
  fill_rect(24, 7, 2, 1, "#3b1f26");
  fill_rect(5, 8, 1, 2, "#3b1f26");
  fill_rect(6, 8, 4, 2, "#ef5b5b");
  fill_rect(10, 8, 4, 1, "#d62f3a");
  fill_rect(17, 8, 1, 5, "#d62f3a");
  fill_rect(18, 8, 3, 3, "#ffffff");
  fill_rect(21, 8, 5, 2, "#9c1f2e");
  fill_rect(26, 8, 1, 2, "#3b1f26");
  fill_rect(10, 9, 4, 3, "#ffffff");
  fill_rect(4, 10, 1, 3, "#3b1f26");
  fill_rect(5, 10, 3, 3, "#ef5b5b");
  fill_rect(8, 10, 2, 4, "#d62f3a");
  fill_rect(21, 10, 1, 3, "#d62f3a");
  fill_rect(22, 10, 5, 3, "#9c1f2e");
  fill_rect(27, 10, 1, 3, "#3b1f26");
  fill_rect(18, 11, 1, 2, "#d62f3a");
  fill_rect(19, 11, 2, 1, "#e8d9d9");
  fill_rect(10, 12, 1, 4, "#d62f3a");
  fill_rect(11, 12, 2, 1, "#e8d9d9");
  fill_rect(13, 12, 1, 4, "#d62f3a");
  fill_rect(19, 12, 2, 1, "#d62f3a");
  fill_rect(3, 13, 1, 6, "#3b1f26");
  fill_rect(4, 13, 2, 3, "#ef5b5b");
  fill_rect(6, 13, 2, 1, "#d62f3a");
  fill_rect(11, 13, 2, 3, "#d62f3a");
  fill_rect(17, 13, 5, 1, "#9c1f2e");
  fill_rect(22, 13, 2, 2, "#ffffff");
  fill_rect(24, 13, 1, 3, "#e8d9d9");
  fill_rect(25, 13, 3, 5, "#9c1f2e");
  fill_rect(28, 13, 1, 6, "#3b1f26");
  fill_rect(6, 14, 3, 3, "#ffffff");
  fill_rect(9, 14, 1, 2, "#d62f3a");
  fill_rect(14, 14, 4, 2, "#ffffff");
  fill_rect(18, 14, 4, 4, "#9c1f2e");
  fill_rect(22, 15, 2, 1, "#e8d9d9");
  fill_rect(4, 16, 2, 2, "#9c1f2e");
  fill_rect(9, 16, 5, 2, "#9c1f2e");
  set_pixel(14, 16, "#ffffff");
  fill_rect(15, 16, 3, 1, "#e8d9d9");
  fill_rect(22, 16, 3, 2, "#9c1f2e");
  set_pixel(6, 17, "#9c1f2e");
  set_pixel(7, 17, "#e8d9d9");
  set_pixel(8, 17, "#9c1f2e");
  fill_rect(14, 17, 4, 1, "#9c1f2e");
  fill_rect(5, 18, 3, 1, "#c9a77a");
  fill_rect(8, 18, 16, 1, "#a88860");
  fill_rect(24, 18, 3, 1, "#c9a77a");
  fill_rect(4, 19, 2, 1, "#3b1f26");
  fill_rect(6, 19, 20, 2, "#c9a77a");
  fill_rect(26, 19, 2, 1, "#3b1f26");
  set_pixel(4, 20, "#3b1f26");
  set_pixel(5, 20, "#c9a77a");
  set_pixel(26, 20, "#c9a77a");
  set_pixel(27, 20, "#3b1f26");
  fill_rect(5, 21, 7, 1, "#3b1f26");
  fill_rect(20, 21, 7, 1, "#3b1f26");
});

part("stem", () => {
  fill_rect(12, 21, 2, 5, "#fff5d9");
  fill_rect(14, 21, 3, 7, "#f3e2b8");
  fill_rect(17, 21, 3, 7, "#d4b98a");
  fill_rect(11, 22, 1, 4, "#3b1f26");
  fill_rect(20, 22, 1, 4, "#3b1f26");
  fill_rect(10, 26, 1, 3, "#3b1f26");
  fill_rect(11, 26, 3, 2, "#f3e2b8");
  fill_rect(20, 26, 1, 2, "#d4b98a");
  fill_rect(21, 26, 1, 3, "#3b1f26");
  fill_rect(11, 29, 10, 1, "#3b1f26");
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("cap");
  draw("stem");
});

action("rebote", { frames: 6, fps: 8, loop: true }, (f, { t }) => {
  const alto = [0, -3, -6, -5, -2, 0][f];
  if (f === 0) {
    // aplastamiento al tocar el suelo: el sombrero se hunde sobre el tallo
    draw("stem");
    move(0, 2, () => draw("cap"));
  } else {
    move(0, alto, () => {
      draw("cap");
      draw("stem");
    });
  }
});

action("balanceo", { frames: 4, fps: 6, loop: true }, (f, { t }) => {
  const dx = [0, 1, 0, -1][f];
  draw("stem");
  move(dx, 0, () => draw("cap"));
});

