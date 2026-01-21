package utils

import (
	"fmt"
	"strings"

	"github.com/spf13/viper"
)

type Config struct {
	TiingoWSURL   string
	TiingoAPIKey  string
	RedisAddrs    []string
	RedisPassword string
}

func LoadEnv() (*Config, error) {
	v := viper.New()

	// Enable ENV variables
	v.AutomaticEnv()

	// Optional: allow ENV names like REDIS_PASSWORD
	v.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
	var redisAddrs []string
	for i := 1; i <= 6; i++ {
		host := v.GetString(fmt.Sprintf("REDIS_NODE_%d_HOST", i))
		port := v.GetString(fmt.Sprintf("REDIS_NODE_%d_PORT", i))
		if host != "" && port != "" {
			redisAddrs = append(redisAddrs, fmt.Sprintf("%s:%s", host, port))
		}

	}
	if len(redisAddrs) == 0 {
		return nil, fmt.Errorf("no Redis nodes configured in environment")
	}

	cfg := &Config{
		TiingoWSURL:   v.GetString("TIINGO_WS_URL"),
		TiingoAPIKey:  v.GetString("TIINGO_API_KEY"),
		RedisAddrs:    redisAddrs,
		RedisPassword: v.GetString("REDIS_PASSWORD"),
	}

	return cfg, nil
}

func ToInterfaceSlice(ss []string) []interface{} {
	out := make([]interface{}, len(ss))
	for i, s := range ss {
		out[i] = s
	}
	return out
}
