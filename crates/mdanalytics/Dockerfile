FROM rust:1.96-slim-bullseye AS build
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config libfontconfig1-dev build-essential && rm -rf /var/lib/apt/lists/*

# Copy all source code (respects .dockerignore)
COPY . .

# Build mdanalytics
WORKDIR /app/crates/mdanalytics
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/server /app/server && \
    cp /app/target/release/tui /app/tui

# Production stage
FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/server .
COPY --from=build /app/tui .
CMD ["./server"]
