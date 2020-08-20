package server

import (
	"log"
	"net/url"

	"github.com/equinor/oneseismic/api/oneseismic"
	"github.com/kataras/iris/v12"
	"github.com/pebbe/zmq4"
	"google.golang.org/protobuf/proto"
)

// Register endpoints for oneseismic
func Register(
	app *iris.Application,
	storageEndpoint url.URL,
	zmqReqAddr,
	zmqRepAddr string,
	zmqFailureAddr string,
) {
	sc := storeController{&storageURL{storageEndpoint}}

	sessions := newSessions()
	go sessions.Run(zmqReqAddr, zmqRepAddr, zmqFailureAddr)

	app.Get("/", sc.list)
	app.Get("/{guid:string}", sc.services)
	app.Get("/{guid:string}/slice", sc.dimensions)
	app.Get("/{guid:string}/slice/{dimension:int32}", sc.lines)

	slice := sliceController {
		slicer: &slicer {
			endpoint: storageEndpoint.String(),
			sessions: sessions,
		},
	}
	app.Get("/{guid:string}/slice/{dim:int32}/{lineno:int32}", slice.get)

}

func CoreMock(
	reqNdpt string,
	repNdpt string,
	failureAddr string,
	slice *oneseismic.SliceResponse,
	numPartials int,
) {
	in, _ := zmq4.NewSocket(zmq4.PULL)
	in.Connect(reqNdpt)

	out, _ := zmq4.NewSocket(zmq4.ROUTER)
	out.SetRouterMandatory(1)
	out.Connect(repNdpt)

	for {
		m, _ := in.RecvMessageBytes(0)
		proc := process{}
		err := proc.loadZMQ(m)
		if err != nil {
			msg := "Broken process (loadZMQ) in core emulation: %s"
			log.Fatalf(msg, err.Error())
		}
		fr := oneseismic.FetchResponse{Requestid: proc.pid}
		fr.Function = &oneseismic.FetchResponse_Slice{
			Slice: slice,
		}

		bytes, _ := proto.Marshal(&fr)
		for i := 0; i < numPartials; i++ {
			partial := routedPartialResult{
				address: proc.address,
				partial: partialResult{
					pid:     proc.pid,
					n:       i,
					m:       numPartials,
					payload: bytes,
				},
			}

			_, err = partial.sendZMQ(out)

			for err == zmq4.EHOSTUNREACH {
				_, err = out.SendMessage(m)
			}
		}
	}
}
