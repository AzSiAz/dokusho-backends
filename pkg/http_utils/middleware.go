package http_utils

import (
	"log/slog"
	"net/http"
	"strings"
)

func APIKeyMiddleware(useApiKey bool, apiKey string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if useApiKey {
				if r.Header.Get("X-API-KEY") != apiKey {
					http.Error(w, "Unauthorized", http.StatusUnauthorized)
					return
				}
			}

			next.ServeHTTP(w, r)
		})
	}
}

func WhitelistedReverseProxy(use bool, addrs ...string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			addr := strings.Split(r.RemoteAddr, ":")[0]

			if use {
				for _, a := range addrs {
					if addr == a {
						next.ServeHTTP(w, r)
						return
					}
				}

				slog.Error("Request from non-whitelisted reverse proxy", "addr", addr)
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			slog.Debug("Request from whitelisted reverse proxy", "addr", addr)
			next.ServeHTTP(w, r)
		})
	}
}
