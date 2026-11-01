// Animación "impacto" · celdas de 40x40 · la base empieza en (4, 4) dentro de cada celda.
config({ name: "impacto", base: "base.png", cell: [40, 40], origin: [4, 4], colors: [] });

// Rayos largos en cian y amarillo que salen del centro (16, 16).
part("rayos", () => {
  line(16, 16, 16, 4, "#00c8ff");
  line(15, 15, 15, 9, "#00c8ff");
  line(17, 15, 17, 10, "#00c8ff");
  line(16, 16, 25, 8, "#ffd84a");
  line(17, 16, 23, 10, "#ffd84a");
  line(16, 15, 21, 10, "#ffd84a");
  line(16, 16, 28, 17, "#00c8ff");
  line(17, 15, 24, 15, "#00c8ff");
  line(17, 17, 23, 17, "#00c8ff");
  line(16, 16, 23, 24, "#00c8ff");
  line(17, 16, 21, 20, "#00c8ff");
  line(15, 17, 15, 23, "#ffd84a");
  line(17, 17, 16, 22, "#ffd84a");
  line(16, 16, 6, 23, "#00c8ff");
  line(15, 16, 9, 20, "#00c8ff");
  line(15, 18, 10, 21, "#00c8ff");
  line(16, 16, 3, 14, "#00c8ff");
  line(15, 15, 8, 14, "#00c8ff");
  line(15, 17, 9, 16, "#00c8ff");
  line(16, 16, 9, 9, "#ffd84a");
  line(15, 16, 11, 12, "#ffd84a");
  line(16, 15, 16, 7, "#5ff4ff");
  line(16, 14, 16, 10, "#c8ffff");
  line(17, 15, 22, 10, "#fff6a0");
  line(17, 16, 25, 16, "#5ff4ff");
  line(17, 16, 21, 16, "#c8ffff");
  line(17, 17, 21, 21, "#5ff4ff");
  line(16, 17, 15, 25, "#fff6a0");
  line(15, 17, 9, 21, "#5ff4ff");
  line(15, 16, 6, 15, "#5ff4ff");
  line(15, 16, 10, 15, "#c8ffff");
  line(15, 15, 11, 11, "#fff6a0");
});

// Núcleo blanco del impacto.
part("nucleo", () => {
  fill_rect(13, 13, 6, 6, "#5ff4ff");
  set_pixel(13, 13, "transparent");
  set_pixel(18, 13, "transparent");
  set_pixel(13, 18, "transparent");
  set_pixel(18, 18, "transparent");
  fill_rect(14, 14, 4, 4, "#c8ffff");
  fill_rect(15, 14, 2, 4, "#ffffff");
  fill_rect(14, 15, 4, 2, "#ffffff");
});

// Puntas oscuras de los rayos principales.
part("puntas", () => {
  set_pixel(16, 4, "#0a8cff");
  set_pixel(3, 14, "#0a8cff");
  set_pixel(28, 17, "#0a8cff");
});

// Chispitas sueltas; d las aleja del centro.
part("chispitas", ({ d = 0 } = {}) => {
  set_pixel(24 + d, 4 - d, "#5ff4ff");
  set_pixel(25 + d, 4 - d, "#00c8ff");
  set_pixel(5 - d, 8 - d, "#fff6a0");
  set_pixel(26 + d, 24 + d, "#5ff4ff");
  set_pixel(9 - d, 27 + d, "#ffd84a");
  set_pixel(9 - d, 26 + d, "#fff6a0");
});

// Cola amarilla hacia abajo.
part("cola", () => {
  line(15, 24, 15, 28, "#ffd84a");
  line(15, 23, 15, 25, "#fff6a0");
});

part("brillos", () => {
  set_pixel(16, 13, "#ffffff");
  set_pixel(19, 16, "#c8ffff");
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("rayos");
  draw("nucleo");
  draw("puntas");
  draw("chispitas");
  draw("cola");
  draw("brillos");
});

action("chispa", { frames: 4, fps: 20, loop: false }, (f, { t }) => {
  if (f === 0) {
    // Destello inicial: punto blanco con cruz corta.
    line(16, 12, 16, 20, "#5ff4ff");
    line(12, 16, 20, 16, "#5ff4ff");
    line(16, 13, 16, 19, "#c8ffff");
    line(13, 16, 19, 16, "#c8ffff");
    fill_rect(15, 15, 2, 2, "#ffffff");
    set_pixel(16, 14, "#ffffff");
    set_pixel(14, 16, "#ffffff");
  } else if (f === 1) {
    draw("rayos");
    draw("nucleo");
    draw("puntas");
    draw("chispitas");
    draw("cola");
    draw("brillos");
  } else if (f === 2) {
    // Se apaga: rayos más cortos y oscuros, núcleo pequeño, chispitas alejándose.
    line(16, 15, 16, 8, "#00c8ff");
    line(17, 16, 24, 16, "#00c8ff");
    line(15, 16, 7, 15, "#00c8ff");
    line(16, 17, 15, 23, "#00c8ff");
    line(17, 15, 21, 11, "#ffd84a");
    line(15, 15, 12, 12, "#ffd84a");
    line(17, 17, 21, 21, "#00c8ff");
    line(15, 17, 10, 20, "#00c8ff");
    set_pixel(16, 8, "#0a8cff");
    set_pixel(24, 16, "#0a8cff");
    set_pixel(7, 15, "#0a8cff");
    set_pixel(15, 23, "#0a8cff");
    fill_rect(15, 15, 2, 2, "#c8ffff");
    set_pixel(16, 15, "#ffffff");
    draw("chispitas", { d: 2 });
  } else {
    // Restos: chispitas sueltas a punto de desaparecer.
    set_pixel(28, 1, "#00c8ff");
    set_pixel(2, 5, "#fff6a0");
    set_pixel(29, 27, "#5ff4ff");
    set_pixel(6, 30, "#ffd84a");
  }
});
