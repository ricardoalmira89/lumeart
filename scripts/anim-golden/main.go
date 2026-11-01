// Genera los fotogramas de referencia de pxcli para cada anim.js: un hash
// FNV-1a de 64 bits por imagen (bytes RGBA por filas) y los avisos.
package main

import (
	"encoding/json"
	"fmt"
	"hash/fnv"
	"image"
	"os"
	"path/filepath"

	"pxcli/internal/anim"
)

type actionOut struct {
	Name   string   `json:"name"`
	Frames int      `json:"frames"`
	FPS    int      `json:"fps"`
	Loop   bool     `json:"loop"`
	Line   int      `json:"line"`
	OOB    []int    `json:"out_of_bounds"`
	Hashes []string `json:"images"`
}

type out struct {
	Name    string       `json:"name"`
	Cell    [2]int       `json:"cell"`
	Origin  [2]int       `json:"origin"`
	Colors  []string     `json:"colors"`
	Base    string       `json:"base"`
	Rest    string       `json:"rest"`
	Actions []actionOut  `json:"actions"`
	Issues  []anim.Issue `json:"issues"`
	Checked []anim.Issue `json:"checked"`
}

func hash(img *image.RGBA) string {
	if img == nil {
		return ""
	}
	h := fnv.New64a()
	b := img.Bounds()
	for y := 0; y < b.Dy(); y++ {
		for x := 0; x < b.Dx(); x++ {
			c := img.RGBAAt(b.Min.X+x, b.Min.Y+y)
			_, _ = h.Write([]byte{c.R, c.G, c.B, c.A})
		}
	}
	return fmt.Sprintf("%016x", h.Sum64())
}

func main() {
	dst := os.Args[1]
	for _, dir := range os.Args[2:] {
		src, err := os.ReadFile(filepath.Join(dir, "anim.js"))
		if err != nil {
			panic(err)
		}
		load := func(name string) (*image.RGBA, error) { return anim.LoadPNG(filepath.Join(dir, name)) }
		res := anim.Render(string(src), anim.Options{LoadBase: load})
		o := out{
			Name: res.Name, Cell: [2]int{res.Cell.X, res.Cell.Y}, Origin: [2]int{res.Origin.X, res.Origin.Y},
			Colors: []string{}, Base: hash(res.Base), Rest: hash(res.Rest),
		}
		for _, c := range res.Colors {
			o.Colors = append(o.Colors, anim.Hex(c))
		}
		for _, a := range res.Actions {
			var hs []string
			for _, img := range a.Images {
				hs = append(hs, hash(img))
			}
			o.Actions = append(o.Actions, actionOut{a.Name, a.Frames, a.FPS, a.Loop, a.Line, a.OutOfBounds, hs})
		}
		o.Issues = append([]anim.Issue{}, res.Issues...)
		anim.Check(res)
		o.Checked = append([]anim.Issue{}, res.Issues...)
		name := filepath.Base(dir)
		od := filepath.Join(dst, name)
		_ = os.MkdirAll(od, 0o755)
		b, _ := json.MarshalIndent(o, "", "  ")
		_ = os.WriteFile(filepath.Join(od, "pxcli.json"), append(b, '\n'), 0o644)
		fmt.Println(name, len(res.Actions), "acciones", len(o.Issues), "avisos", len(o.Checked), "con check")
	}
}
