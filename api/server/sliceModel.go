package server

import (
	"fmt"

	"github.com/equinor/oneseismic/api/oneseismic"
	"github.com/kataras/golog"
	"google.golang.org/protobuf/proto"
)

type slicer struct {
	endpoint string
	sessions *sessions
}

type SR struct {
	V []float32
	Shape0 int32
	Shape1 int32
 }

func (s *slicer) fetchSlice(
	guid string,
	dim int32,
	lineno int32,
	requestid string,
	token string,
) (*SR, error) {

	msg := oneseismic.ApiRequest{
		Requestid:       requestid,
		Token:           token,
		Guid:            guid,
		StorageEndpoint: s.endpoint,
		Shape: &oneseismic.FragmentShape{
			Dim0: 64,
			Dim1: 64,
			Dim2: 64,
		},
		Function: &oneseismic.ApiRequest_Slice{
			Slice: &oneseismic.ApiSlice{
				Dim:    dim,
				Lineno: lineno,
			},
		},
	}

	req, err := proto.Marshal(&msg)
	if err != nil {
		return nil, fmt.Errorf("Marshalling oneseisimc.ApiRequest: %w", err)
	}

	proc := process{pid: requestid, request: req}
	fr := oneseismic.FetchResponse{}
	golog.Infof("%v: sceduling", requestid)
	io := s.sessions.Schedule(&proc)
	golog.Infof("%v: sceduled", requestid)

	/*
	 * Read and parse messages as they come, and consider the process complete
	 * when the reply-channel closes.
	 *
	 * Right now, the result is assembled here and returned in one piece to
	 * users, so it never looks like a parallelised job. This is so that we can
	 * experiment with chunk sizes, worker nodes, load etc. without having to
	 * be bothered with a more complex protocol between API and users, and so
	 * that previously-written clients still work. In the future, this will
	 * probably change and partial results will be transmitted.
	 *
	 * TODO: This gives weak failure handling, and Session needs a way to
	 * signal failed processes
	 */
	var tiles []*oneseismic.SliceTile
	for partial := range io.out {
		golog.Infof("%v: reply", requestid)

		err = proto.Unmarshal(partial.payload, &fr)

		if err != nil {
			return nil, fmt.Errorf("could not create slice response: %w", err)
		}

		slice := fr.GetSlice()
		// TODO: cancel job on failure channel
		if slice == nil {
			switch x := fr.Function.(type) {
			default:
				msg := "%s Expected FetchResponse.Function = %T; was %T"
				golog.Errorf(msg, requestid, slice, x)
				return nil, fmt.Errorf("internal error")
			}
		}

		tiles = append(tiles, slice.GetTiles()...)
	}

	/*
	 * On successful runs, there are no messages on this channel, and the loop
	 * turns into a no-op.
	 */
	for failure := range io.err {
		return nil, newFailure(failure)
	}

	fr.GetSlice().Tiles = tiles
	slice := fr.GetSlice()
	dim0 := slice.SliceShape.Dim0
	dim1 := slice.SliceShape.Dim1
 
	sr := SR{
		Shape0: dim0,
		Shape1: dim1,
		V: make([]float32, dim0*dim1),
	}
 	return &sr, nil
}
