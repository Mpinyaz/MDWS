package core

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/google/uuid"
)

type (
	HealthResponse struct {
		Status           string `json:"status,omitempty"`
		ConnectedClients int    `json:"connected_clients,omitempty"`
		Uptime           string `json:"uptime,omitempty"`
		Timestamp        string `json:"timestamp,omitempty"`
	}
)

var startTime = time.Now()

func HandleBroadcast(c *Client, event Event) error {
	log.Printf("Broadcasting message from %s: %s", c.ID, event.Payload)
	c.Mgmt.Broadcast <- &event
	return nil
}

func HandlePing(c *Client, event Event) error {
	log.Printf("Ping received from %s", c.ID)

	pongEvent := &Event{
		Type:    "pong",
		Payload: json.RawMessage(`{"timestamp":"` + time.Now().Format(time.RFC3339) + `"}`),
		Time:    time.Now(),
	}

	c.Send <- pongEvent
	return nil
}

func healthCheckHandler(hub *Manager) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")

		uptime := time.Since(startTime)
		hub.Mu.RLock()
		count := len(hub.Clients)
		hub.Mu.RUnlock()

		response := HealthResponse{
			Status:           "ok",
			ConnectedClients: count,
			Uptime:           uptime.String(),
			Timestamp:        time.Now().Format(time.RFC3339),
		}

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(response)
	}
}

// ServeWS handles websocket requests from clients
func ServeWS(hub *Manager, w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Upgrade failed: %v", err)
		return
	}

	clientID := uuid.New()

	client := &Client{
		Mgmt: hub,
		Conn: conn,
		Send: make(chan *Event, 256),
		ID:   clientID,
		Done: make(chan struct{}),
	}

	client.Mgmt.Register <- client

	// Start read and write pumps in separate goroutines
	go client.writePump()
	go client.readPump()
}
