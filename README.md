# Market Data Workers (MDWS)

A high-performance real-time market data streaming system built with Rust,
featuring WebSocket subscriptions, Redis Clusters and RabbitMQ Stream
publishing.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Market Data Workers                     │
│                                                             │
│  ┌──────────────┐         ┌──────────────────────────┐      │
│  │   Client     │         │    WebSocket Handlers    │      │
│  │              │────────▶│                          │      │
│  │  - Forex WS  │         │  - Forex Handler         │      │
│  │  - Crypto WS │         │  - Crypto Handler        │      │
│  │  - Equity WS │         │  - Equity Handler        │      │
│  └──────────────┘         └──────────┬───────────────┘      │
│                                      │                      │
│                                      │ Market Data          │
│                                      ▼                      │
│                           ┌─────────────────┐               │
│                           │  Message Parser │               │
│                           │  (deserialize)  │               │
│                           └────────┬────────┘               │
│                                    │                        │
│                                    │ WsResponse             │
│                                    ▼                        │
│                           ┌─────────────────┐               │
│                           │  RabbitMQ Stream│               │
│                           │    Publisher    │               │
│                           └────────┬────────┘               │
└──────────────────────────────────┼──────────────────────────┘
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │   RabbitMQ Streams   │
                        │                      │
                        │  - marketupdates     │
                        │  - subscriptions     │
                        └──────────────────────┘
```

## Features

- **Multi-Asset Class Support**: Forex, Crypto, and Equities via separate
  WebSocket connections
- **Event-Driven Architecture**: Asynchronous message processing with Tokio
- **RabbitMQ Stream Integration**: High-throughput message publishing with
  persistent streams
- **Type-Safe Messaging**: Strongly-typed message parsing with Serde
- **Configurable Subscriptions**: Dynamic ticker subscriptions per asset class
- **Graceful Error Handling**: Connection resilience with automatic error
  recovery
- **Production-Ready Logging**: Structured logging with tracing
