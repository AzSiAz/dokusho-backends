package client

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

type flareSolverHealthCheckResponse struct {
	Status string `json:"status"`
}

type FlareSolverInstruction struct {
	CMD               string `json:"cmd"`
	URL               string `json:"url"`
	Session           string `json:"session"`
	SessionTTLMinutes int    `json:"session_ttl_minutes"`
	MaxTimeout        int    `json:"max_timeout"`
	Cookies           []struct {
		Name  string `json:"name"`
		Value string `json:"value"`
	} `json:"cookies"`
	ReturnOnlyCookies bool `json:"returnOnlyCookies"`
	Proxy             struct {
		URL string `json:"url"`
	} `json:"proxy"`
}

type FlareSolverResponse struct {
	Status   string `json:"status"`
	Message  string `json:"message"`
	Solution struct {
		URL      string            `json:"url"`
		Response string            `json:"response"`
		Headers  map[string]string `json:"headers"`
	} `json:"solution"`
}

type FlareSolverClient struct {
	URL string
}

func NewFlareSolverClient(url string) *FlareSolverClient {
	return &FlareSolverClient{
		URL: url,
	}
}

func (f *FlareSolverClient) Bypass(url string) (string, error) {
	instruction := FlareSolverInstruction{
		CMD: "request.get",
		URL: url,
	}

	jsonBody, err := json.Marshal(instruction)
	if err != nil {
		return "", fmt.Errorf("Failed to marshal flaresolverr instruction: %w", err)
	}

	resp, err := http.Post(fmt.Sprintf("%s/solve", f.URL), "application/json", strings.NewReader(string(jsonBody)))
	if err != nil || resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("Failed to connect to flaresolver, or flaresolver is not healthy")
	}

	defer resp.Body.Close()

	html, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("Failed to read health check response body: %w", err)
	}

	var response FlareSolverResponse
	err = json.Unmarshal(html, &response)
	if err != nil {
		return "", fmt.Errorf("Failed to unmarshal health check response: %w", err)
	}

	if strings.ToLower(response.Status) != "ok" {
		return "", fmt.Errorf("Flaresolver is not healthy, status: %s - %s", response.Status, response.Message)
	}

	return response.Solution.Response, nil
}

func (f *FlareSolverClient) Ping() error {
	resp, err := http.Get(fmt.Sprintf("%s/health", f.URL))
	if err != nil || resp.StatusCode != http.StatusOK {
		return fmt.Errorf("Failed to connect to flaresolver, or flaresolver is not healthy")
	}

	defer resp.Body.Close()

	html, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("Failed to read health check response body: %w", err)
	}

	var response flareSolverHealthCheckResponse
	err = json.Unmarshal(html, &response)
	if err != nil {
		return fmt.Errorf("Failed to unmarshal health check response: %w", err)
	}

	if strings.ToLower(response.Status) != "ok" {
		return fmt.Errorf("Flaresolver is not healthy, status: %s", response.Status)
	}

	return nil
}
