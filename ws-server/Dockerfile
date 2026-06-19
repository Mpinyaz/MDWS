FROM golang:latest AS build

WORKDIR /app

# Copy dependency files first
COPY go.mod go.sum ./
RUN CGO_ENABLED=0 go mod download

# Now copy full source
COPY . .

# Build binary
RUN CGO_ENABLED=0 go build -o mdws cmd/main.go

# Production stage
FROM alpine:latest

WORKDIR /app
COPY --from=build /app/mdws .

EXPOSE 8080
CMD ["./mdws"]
