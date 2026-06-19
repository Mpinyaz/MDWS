FROM rust:1.96-alpine AS build
WORKDIR /app

# Install required system dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Copy all source code (respects .dockerignore)
COPY . .

# Build mdworkers
WORKDIR /app/crates/mdworkers
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin server && \
    cp /app/target/release/server /app/server

# Production stage
FROM alpine:latest
WORKDIR /app
COPY --from=build /app/server .
CMD ["./server"]
