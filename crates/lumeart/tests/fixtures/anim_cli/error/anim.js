config({ cell: [8, 8] });
action("a", { frames: 2 }, (f) => {
  fill_rect(0, 0, 2, 2, "nocolor");
});
action("b", { frames: 2 }, (f) => { set_pixel(f, f, "#fff"); });
