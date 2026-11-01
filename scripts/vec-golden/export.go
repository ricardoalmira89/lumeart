package vec

// Solo para el generador de referencias de LumeArt: expone la rejilla y
// el formato de números de raster.go.

func GoldenGrid(vx, vy, vw, vh, step float64) string { return gridLines(vx, vy, vw, vh, step) }

func GoldenNum(v float64) string { return num(v) }
