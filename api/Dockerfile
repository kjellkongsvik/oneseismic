FROM golang:1.15-buster as builder
RUN apt update && apt install -y protobuf-compiler

WORKDIR /src
RUN apt-get update && apt-get install -y libzmq5-dev
COPY api/go.mod .
COPY api/go.sum .
RUN go mod download

COPY api api
COPY protos protos

WORKDIR /src/api
RUN go generate

RUN go test -race ./...

RUN go build

# FROM debian:buster-slim as deployer
# RUN apt-get update && apt-get install -y libzmq5
# COPY --from=builder /src/api/api /bin/api
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
RUN cp /src/api/api /bin/api

EXPOSE 8080 8081
ENTRYPOINT ["api"]
