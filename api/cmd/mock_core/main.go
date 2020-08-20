package main

import (
	"math/rand"
	"os"

	"github.com/equinor/oneseismic/api/oneseismic"
	"github.com/equinor/oneseismic/api/server"
	"github.com/joho/godotenv"
	"github.com/kataras/golog"
)

func init() {
	godotenv.Load() // nolint, silently ignore missing or invalid .env
}

func main() {
	golog.SetLevel(os.Getenv("LOG_LEVEL"))
	var tiles []*oneseismic.SliceTile
	r := rand.New(rand.NewSource(99))
	for i := 0; i < 30; i++ {
		v := make([]float32, 2500)
		for i := range v {
			v[i] = r.Float32()
		}
		tile := []*oneseismic.SliceTile{
			{
				Layout: &oneseismic.SliceLayout{
					ChunkSize:  1,
					Iterations: 0,
				},
				V: v,
			},
		}
		tiles = append(tiles, tile...)
	}

	slice := &oneseismic.SliceResponse{
		Tiles:      tiles,
		SliceShape: &oneseismic.SliceShape{Dim0: 201, Dim1: 720},
	}
	golog.Debug("core mock")
	server.CoreMock(os.Getenv("ZMQ_REQ_ADDR"), os.Getenv("ZMQ_REP_ADDR"), os.Getenv("ZMQ_FAILURE_ADDR"), slice, 100)
}
