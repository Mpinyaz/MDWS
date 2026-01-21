package main

import (
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	. "websockets/internal/core"
	"websockets/utils"
)

func main() {
	cfg, err := utils.LoadEnv()
	if err != nil {
		log.Fatal(err)
	}

	// Create manager

	manager := NewManager(cfg)
	manager.setupEventHandlers()

	// Register REST API forex handlers
	manager.RegisterHandler("forex_subscribe", HandleForexSubscribeREST)
	manager.RegisterHandler("forex_unsubscribe", HandleForexUnsubscribeREST)

	go manager.Run()

	// HTTP handlers
	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		ServeWS(manager, w, r)
	})

	http.HandleFunc("/health", healthCheckHandler(manager))

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"message": "WebSocket Forex Server (REST API)",
			"ws_url": "ws://localhost:8080/ws",
			"data_source": "Tiingo REST API (Free tier compatible)",
			"update_interval": "5 seconds"
		}`))
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	go func() {
		log.Printf("🚀 WebSocket server starting on :%s", port)
		log.Println("📊 Forex data via Tiingo REST API (5 second polling)")
		if err := http.ListenAndServe(":"+port, nil); err != nil {
			log.Fatal("ListenAndServe: ", err)
		}
	}()

	log.Println("✅ Server is ready. Press Ctrl+C to stop")

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)
	<-sigChan

	log.Println("\n🛑 Shutting down gracefully...")
	if forexPoller != nil {
		forexPoller.Stop()
	}
	log.Println("👋 Server stopped")
}
