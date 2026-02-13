.PHONY: build run clean  dev air-dev install-deps tidy test

# Go build variables
BINARY_NAME=mdws
GO_FILES=$(shell find . -name "*.go" -not -path "./vendor/*")
AIR_BINARY=./$(BINARY_NAME)

# Default target
all: install-deps  build

# Install development dependencies
install-deps:
	@echo "Installing Air for hot reloading..."
	@go install github.com/cosmtrek/air@latest

build:
	@echo "Building $(BINARY_NAME)..."
	@mkdir -p bin
	@go build -o $(AIR_BINARY) ./cmd/main.go

# Run the application directly
run: build
	$(AIR_BINARY)

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf bin/

tidy:
	@go mod tidy

air-dev: install-deps tailwind
	@echo "Starting development with Air hot reloading..."
	@air
