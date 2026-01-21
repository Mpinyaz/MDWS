package core

import (
	"encoding/json"
	"log"
	"net/http"
	"sync"
	"time"

	"websockets/utils"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
)

const (
	writeWait      = 10 * time.Second
	pongWait       = 60 * time.Second
	pingPeriod     = (pongWait * 9) / 10
	maxMessageSize = 8192 // Increased for JSON payloads
)

var upgrader = websocket.Upgrader{
	CheckOrigin:     func(r *http.Request) bool { return true },
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

type Event struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload"`
	From    uuid.UUID       `json:"from,omitempty"`
	Time    time.Time       `json:"time"`
}

type Manager struct {
	Clients       map[*Client]bool
	Mu            sync.RWMutex
	Broadcast     chan *Event
	Register      chan *Client
	Unregister    chan *Client
	Handlers      map[string]EventHandler
	configuration *utils.Config
}

type Client struct {
	Mgmt *Manager
	Conn *websocket.Conn
	Send chan *Event
	ID   uuid.UUID
	Done chan struct{}
}

type EventHandler func(c *Client, event Event) error

func NewManager(cfg *utils.Config) *Manager {
	return &Manager{
		Clients:       make(map[*Client]bool),
		Broadcast:     make(chan *Event),
		Register:      make(chan *Client),
		Unregister:    make(chan *Client),
		Handlers:      make(map[string]EventHandler),
		configuration: cfg,
	}
}

func (m *Manager) setupEventHandlers() {
	m.Handlers["broadcast"] = HandleBroadcast
	m.Handlers["ping"] = HandlePing
}

func (m *Manager) RegisterHandler(eventType string, handler EventHandler) {
	m.Handlers[eventType] = handler
}

func (m *Manager) NumClients() int {
	m.Mu.RLock()
	defer m.Mu.RUnlock()
	return len(m.Clients)
}

func (m *Manager) Run() {
	for {
		select {
		case client := <-m.Register:
			m.Mu.Lock()
			m.Clients[client] = true
			m.Mu.Unlock()
			log.Printf("Client %s registered. Total clients: %d", client.ID, m.NumClients())

		case client := <-m.Unregister:
			m.Mu.Lock()
			if _, ok := m.Clients[client]; ok {
				delete(m.Clients, client)
				close(client.Done)
				close(client.Send)
				log.Printf("Client %s unregistered. Total clients: %d", client.ID, m.NumClients())
			}
			m.Mu.Unlock()

		case message := <-m.Broadcast:
			m.Mu.RLock() // allow multiple readers for iteration
			for client := range m.Clients {
				select {
				case client.Send <- message:
				default:
					close(client.Send)
					m.Mu.RUnlock() // unlock temporarily to delete safely
					m.Mu.Lock()
					delete(m.Clients, client)
					m.Mu.Unlock()
					log.Printf("Client %s removed due to slow consumption", client.ID)
					m.Mu.RLock() // re-lock for remaining iteration
				}
			}
			m.Mu.RUnlock()
		}
	}
}

// readPump handles incoming messages from the websocket connection
func (c *Client) readPump() {
	defer func() {
		c.Mgmt.Unregister <- c
		c.Conn.Close()
	}()

	c.Conn.SetReadLimit(maxMessageSize)
	c.Conn.SetReadDeadline(time.Now().Add(pongWait))
	c.Conn.SetPongHandler(func(string) error {
		c.Conn.SetReadDeadline(time.Now().Add(pongWait))
		return nil
	})

	for {
		var event Event
		err := c.Conn.ReadJSON(&event)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("Read error from client %s: %v", c.ID, err)
			}
			break
		}

		event.From = c.ID
		event.Time = time.Now()

		log.Printf("Received from %s: type=%s, payload=%s", c.ID, event.Type, string(event.Payload))

		// Route to appropriate handler
		if handler, ok := c.Mgmt.Handlers[event.Type]; ok {
			if err := handler(c, event); err != nil {
				log.Printf("Handler error for event type '%s': %v", event.Type, err)

				// Send error response back to client
				errorEvent := &Event{
					Type:    "error",
					Payload: json.RawMessage(`{"message":"` + err.Error() + `"}`),
					Time:    time.Now(),
				}
				c.Send <- errorEvent
			}
		} else {
			log.Printf("Unknown event received from %s: type=%s, payload=%s", c.ID, event.Type, string(event.Payload))

			unknownEvent := &Event{
				Type:    "unknown_event",
				Payload: json.RawMessage(`{"message":"Unknown event type received","type":"` + event.Type + `"}`),
				Time:    time.Now(),
			}

			c.Send <- unknownEvent
		}
	}
}

// writePump handles outgoing messages to the websocket connection
func (c *Client) writePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		ticker.Stop()
		c.Conn.Close()
	}()

	for {
		select {
		case <-c.Done:
			return

		case message, ok := <-c.Send:
			c.Conn.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				c.Conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			w, err := c.Conn.NextWriter(websocket.TextMessage)
			if err != nil {
				log.Printf("Write error to client %s: %v", c.ID, err)
				return
			}
			json.NewEncoder(w).Encode(message)
			w.Close()
		case <-ticker.C:
			c.Conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				log.Println("ping error:", err)
				return
			}
		}
	}
}
