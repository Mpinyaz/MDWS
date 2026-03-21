FROM rust:slim-bullseye AS build
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config libfontconfig1-dev build-essential && rm -rf /var/lib/apt/lists/*

# Copy mdworkers since it's a dependency
COPY mdworkers/ /app/mdworkers/
# Create a dummy src/lib.rs if it doesn't exist to satisfy cargo during dependency fetch
RUN mkdir -p /app/mdworkers/src && touch /app/mdworkers/src/lib.rs

WORKDIR /app/mdanalytics
COPY mdanalytics/Cargo.toml ./
# Remove any existing lockfile to force fresh resolution in the build context
RUN rm -f Cargo.lock

# Cache dependencies with dummy binaries
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/tui.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source and build
COPY mdworkers/src /app/mdworkers/src
COPY mdanalytics/src ./src
RUN cargo build --release

# Production stage
FROM debian:bullseye-slim
WORKDIR /app

# Runtime openssl dependency
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/mdanalytics/target/release/mdanalytics .
COPY --from=build /app/mdanalytics/target/release/tui .
CMD ["./mdanalytics"]
