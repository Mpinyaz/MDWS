package cache

import "github.com/google/uuid"

type ClientSub struct {
	ID     uuid.UUID `json:"id,omitempty"`
	Forex  *[]string `json:"forex,omitempty"`
	Equity *[]string `json:"equity,omitempty"`
	Crypto *[]string `json:"crypto,omitempty"`
}
