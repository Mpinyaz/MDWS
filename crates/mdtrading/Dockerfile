FROM rust:1.96-slim-bullseye AS build
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config build-essential && rm -rf /var/lib/apt/lists/*

# Copy all source code (respects .dockerignore)
COPY . .

# Build mdtrading
WORKDIR /app/crates/mdtrading
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/mdtrading /app/mdtrading

# Production stage
FROM debian:bullseye-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/mdtrading .

CMD ["./mdtrading"]
