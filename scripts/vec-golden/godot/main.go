// Exporta cada dibujo.js a Godot con pxcli (vec godot: un personaje con
// esqueleto o una escena por capas) en un proyecto temporal y escribe
// <salida>/<ejecución>/pxcli.json: el tipo, el .tscn y, por cada textura,
// su tamaño y una rejilla de 8x8 con el color medio (premultiplicado) de
// cada bloque.
package main

import (
	"encoding/json"
	"fmt"
	"image"
	"image/draw"
	_ "image/png"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"pxcli/internal/vec"
)

type texture struct {
	File   string `json:"file"`
	Width  int    `json:"width"`
	Height int    `json:"height"`
	Grid   []int  `json:"grid"`
}

type out struct {
	Kind     string    `json:"kind"`
	Error    string    `json:"error,omitempty"`
	Tscn     string    `json:"tscn"`
	Summary  string    `json:"summary"`
	Textures []texture `json:"textures"`
}

func grid(path string) (int, int, []int) {
	f, err := os.Open(path)
	check(err)
	defer f.Close()
	img, _, err := image.Decode(f)
	check(err)
	b := img.Bounds()
	rgba := image.NewNRGBA(b)
	draw.Draw(rgba, b, img, b.Min, draw.Src)
	w, h := b.Dx(), b.Dy()
	var g []int
	for j := 0; j < 8; j++ {
		for i := 0; i < 8; i++ {
			x0, x1 := i*w/8, (i+1)*w/8
			y0, y1 := j*h/8, (j+1)*h/8
			var sum [4]int
			n := 0
			for y := y0; y < y1; y++ {
				for x := x0; x < x1; x++ {
					o := rgba.PixOffset(x+b.Min.X, y+b.Min.Y)
					a := int(rgba.Pix[o+3])
					for c := 0; c < 3; c++ {
						sum[c] += int(rgba.Pix[o+c]) * a / 255
					}
					sum[3] += a
					n++
				}
			}
			for c := 0; c < 4; c++ {
				if n > 0 {
					g = append(g, sum[c]/n)
				} else {
					g = append(g, 0)
				}
			}
		}
	}
	return w, h, g
}

func main() {
	dest := os.Args[1]
	for _, dir := range os.Args[2:] {
		run := filepath.Base(dir)
		res, err := vec.RenderScene(filepath.Join(dir, vec.FileName))
		check(err)
		if len(res.Errors()) > 0 {
			fmt.Println(run, "con errores en dibujo.js, se salta")
			continue
		}
		tmp, err := os.MkdirTemp("", "vecgodot-")
		check(err)
		check(os.WriteFile(filepath.Join(tmp, "project.godot"), []byte("config_version=5\n"), 0o644))
		arte := filepath.Join(tmp, "arte")
		o := out{}
		var files []string
		if res.Rig != nil {
			o.Kind = "personaje"
			files, err = vec.GodotExport(res, arte, vec.GodotOptions{})
			acts := make([]string, len(res.Rig.Actions))
			for i, a := range res.Rig.Actions {
				acts[i] = a.Name
			}
			o.Summary = fmt.Sprintf("%d huesos, %d piezas, acciones: %s", len(res.Rig.Bones), len(res.Rig.Parts), strings.Join(acts, ", "))
		} else {
			o.Kind = "escena"
			var empty []string
			files, empty, err = vec.GodotSceneExport(res, arte, vec.SceneOptions{})
			if err != nil {
				goto done
			}
			elements := 0
			for _, l := range res.Scene.Layers {
				for _, p := range l.Pieces {
					if p.Element {
						elements++
					}
				}
			}
			o.Summary = fmt.Sprintf("%d texturas en texturas/ · %d capas · %d elementos", len(files)-1, len(res.Scene.Layers), elements)
			if len(empty) > 0 {
				o.Summary += "\nsin nada visible (no se exportan): " + strings.Join(empty, ", ")
			}
		}
	done:
		if err != nil {
			o.Error = err.Error()
			o.Summary = ""
			files = nil
		}
		sort.Strings(files)
		for _, f := range files {
			rel, _ := filepath.Rel(tmp, f)
			if strings.HasSuffix(f, ".tscn") {
				data, err := os.ReadFile(f)
				check(err)
				o.Tscn = string(data)
				continue
			}
			w, h, g := grid(f)
			o.Textures = append(o.Textures, texture{File: filepath.ToSlash(rel), Width: w, Height: h, Grid: g})
		}
		target := filepath.Join(dest, run)
		check(os.MkdirAll(target, 0o755))
		js, err := json.MarshalIndent(o, "", " ")
		check(err)
		check(os.WriteFile(filepath.Join(target, "pxcli.json"), append(js, '\n'), 0o644))
		os.RemoveAll(tmp)
		fmt.Println(run, o.Kind, len(o.Textures))
	}
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
