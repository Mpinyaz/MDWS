FROM rust:slim-bullseye AS build
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config libfontconfig1-dev build-essential && rm -rf /var/lib/apt/lists/*

# Copy all source code (respects .dockerignore)
COPY . .

# Build mdanalytics
WORKDIR /app/mdanalytics
RUN cargo build --release

# Production stage
FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/mdanalytics/target/release/server .
COPY --from=build /app/mdanalytics/target/release/tui .
CMD ["./server"]
