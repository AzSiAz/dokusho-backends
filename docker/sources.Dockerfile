FROM golang:1.24.0-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY cmd/sources ./cmd/sources
COPY pkg ./pkg

RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o sources cmd/sources/main.go

FROM golang:1.24.0-alpine

RUN apk add --no-cache curl

COPY --from=builder /app/sources /app/sources

ENTRYPOINT ["/app/sources"]
