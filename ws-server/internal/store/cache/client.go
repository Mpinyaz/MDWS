package cache

import (
	"context"
	"fmt"

	"websockets/internal/models"

	"github.com/go-redis/redis"
	"github.com/google/uuid"
)

type ClientStore struct {
	rdb *RedisStore
}

func NewClientStore(rdb *RedisStore) *ClientStore {
	return &ClientStore{rdb: rdb}
}

func clientKey(clientID string, asset models.AssetClass) string {
	return fmt.Sprintf("client:{%s}:%s", clientID, asset.String())
}

func symbolKey(asset models.AssetClass, symbol string) string {
	return fmt.Sprintf("symbol:{%s}:%s", asset.String(), symbol)
}

func toInterfaceSlice(ss []string) []interface{} {
	out := make([]interface{}, len(ss))
	for i, s := range ss {
		out[i] = s
	}
	return out
}

func (c *ClientStore) addClientToSymbols(
	ctx context.Context,
	clientID string,
	asset models.AssetClass,
	symbols []string,
) error {
	if len(symbols) == 0 {
		return nil
	}

	_, err := c.rdb.Client.Pipelined(ctx, func(p redis.Pipeliner) error {
		for _, symbol := range symbols {
			p.SAdd(ctx, symbolKey(asset, symbol), clientID)
		}
		return nil
	})

	return err
}

func (c *ClientStore) removeClientFromSymbols(
	ctx context.Context,
	clientID string,
	asset models.AssetClass,
	symbols []string,
) error {
	if len(symbols) == 0 {
		return nil
	}

	_, err := c.rdb.Client.Pipelined(ctx, func(p redis.Pipeliner) error {
		for _, symbol := range symbols {
			p.SRem(ctx, symbolKey(asset, symbol), clientID)
		}
		return nil
	})

	return err
}

func (c *ClientStore) SetClientSubs(sub ClientSub) error {
	ctx := context.Background()
	clientID := sub.ID.String()

	process := func(asset models.AssetClass, newSymbols *[]string) error {
		if newSymbols == nil {
			return nil
		}

		ck := clientKey(clientID, asset)

		oldSymbols, err := c.rdb.Client.SMembers(ctx, ck).Result()
		if err != nil && err != redis.Nil {
			return err
		}

		if err := c.removeClientFromSymbols(ctx, clientID, asset, oldSymbols); err != nil {
			return err
		}

		if err := c.rdb.Client.Del(ctx, ck).Err(); err != nil {
			return err
		}

		if len(*newSymbols) > 0 {
			if err := c.rdb.Client.SAdd(ctx, ck, toInterfaceSlice(*newSymbols)...).Err(); err != nil {
				return err
			}
			return c.addClientToSymbols(ctx, clientID, asset, *newSymbols)
		}

		return nil
	}

	if err := process(models.Forex, sub.Forex); err != nil {
		return err
	}
	if err := process(models.Equity, sub.Equity); err != nil {
		return err
	}
	if err := process(models.Crypto, sub.Crypto); err != nil {
		return err
	}

	return nil
}

func (c *ClientStore) PatchClientSub(asset models.AssetClass, sub ClientSub) error {
	ctx := context.Background()
	clientID := sub.ID.String()

	var symbols *[]string
	switch asset {
	case models.Forex:
		symbols = sub.Forex
	case models.Equity:
		symbols = sub.Equity
	case models.Crypto:
		symbols = sub.Crypto
	}

	if symbols == nil || len(*symbols) == 0 {
		return nil
	}

	ck := clientKey(clientID, asset)

	if err := c.rdb.Client.SAdd(ctx, ck, toInterfaceSlice(*symbols)...).Err(); err != nil {
		return err
	}

	return c.addClientToSymbols(ctx, clientID, asset, *symbols)
}

func (c *ClientStore) RemoveClientSubs(clientID uuid.UUID) error {
	ctx := context.Background()
	id := clientID.String()

	assets := []models.AssetClass{
		models.Forex,
		models.Equity,
		models.Crypto,
	}

	_, err := c.rdb.Client.Pipelined(ctx, func(p redis.Pipeliner) error {
		for _, asset := range assets {
			ck := clientKey(id, asset)

			symbols, err := c.rdb.Client.SMembers(ctx, ck).Result()
			if err != nil && err != redis.Nil {
				return err
			}

			for _, symbol := range symbols {
				p.SRem(ctx, symbolKey(asset, symbol), id)
			}

			p.Del(ctx, ck)
		}
		return nil
	})

	return err
}

func (c *ClientStore) RemoveClientSubsByAsset(
	clientID uuid.UUID,
	asset models.AssetClass,
) error {
	ctx := context.Background()
	id := clientID.String()

	ck := clientKey(id, asset)

	symbols, err := c.rdb.Client.SMembers(ctx, ck).Result()
	if err != nil && err != redis.Nil {
		return err
	}

	_, err = c.rdb.Client.Pipelined(ctx, func(p redis.Pipeliner) error {
		for _, symbol := range symbols {
			p.SRem(ctx, symbolKey(asset, symbol), id)
		}
		p.Del(ctx, ck)
		return nil
	})

	return err
}
