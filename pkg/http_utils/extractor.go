package http_utils

import "net/http"

func ExtractPathParam(r *http.Request, key string, defaultValue string) string {
	value := r.PathValue(key)

	if value == "" && defaultValue != "" {
		return ""
	}

	if value == "" {
		return defaultValue
	}

	return value
}

func ExtractQueryValue(r *http.Request, key string, defaultValue string) string {
	value := r.URL.Query().Get(key)

	if value == "" && defaultValue != "" {
		return ""
	}

	if value == "" {
		return defaultValue
	}

	return value
}
