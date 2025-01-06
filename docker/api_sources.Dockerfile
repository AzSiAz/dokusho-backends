FROM golang:1.23.4-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY cmd/api_sources ./cmd/api_sources
COPY pkg ./pkg

RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o api_sources cmd/api_sources/main.go

FROM scratch

COPY --from=builder /app/api_sources /app/api_sources
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENTRYPOINT ["/app/api_sources"]
