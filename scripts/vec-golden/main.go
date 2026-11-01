// Genera las referencias de rasterizado de pxcli para los fixtures de
// lume-raster: cada caso de cases.json pintado con vec.Rasterize (Chrome),
// el texto de la rejilla (gridLines) y el formato de los números (num).
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"pxcli/internal/vec"
)

type options struct {
	Width      int        `json:"width"`
	Crop       [4]float64 `json:"crop"`
	Grid       float64    `json:"grid"`
	Background string     `json:"background"`
	Overlay    string     `json:"overlay"`
}

type caseIn struct {
	Name    string  `json:"name"`
	SVG     string  `json:"svg"`
	Width   int     `json:"width"`
	Height  int     `json:"height"`
	Options options `json:"options"`
}

type input struct {
	Cases   []caseIn     `json:"cases"`
	Grids   [][5]float64 `json:"grids"`
	Numbers []float64    `json:"numbers"`
}

type gridOut struct {
	Box  [5]float64 `json:"box"`
	Text string     `json:"text"`
}

type numOut struct {
	Value float64 `json:"value"`
	Text  string  `json:"text"`
}

type output struct {
	Grids   []gridOut `json:"grids"`
	Numbers []numOut  `json:"numbers"`
}

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "uso: vecgolden <carpeta de fixtures>")
		os.Exit(2)
	}
	dir := os.Args[1]
	data, err := os.ReadFile(filepath.Join(dir, "cases.json"))
	check(err)
	var in input
	check(json.Unmarshal(data, &in))
	for _, c := range in.Cases {
		svg, err := os.ReadFile(filepath.Join(dir, c.SVG))
		check(err)
		o := vec.RasterOptions{
			Width:      c.Options.Width,
			Crop:       c.Options.Crop,
			Grid:       c.Options.Grid,
			Background: c.Options.Background,
			Overlay:    c.Options.Overlay,
		}
		png, err := vec.Rasterize(string(svg), c.Width, c.Height, o)
		check(err)
		check(os.WriteFile(filepath.Join(dir, c.Name+".png"), png, 0o644))
		fmt.Println(c.Name)
	}
	var out output
	for _, g := range in.Grids {
		out.Grids = append(out.Grids, gridOut{Box: g, Text: vec.GoldenGrid(g[0], g[1], g[2], g[3], g[4])})
	}
	for _, v := range in.Numbers {
		out.Numbers = append(out.Numbers, numOut{Value: v, Text: vec.GoldenNum(v)})
	}
	js, err := json.MarshalIndent(out, "", "  ")
	check(err)
	check(os.WriteFile(filepath.Join(dir, "pxcli.json"), append(js, '\n'), 0o644))
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
