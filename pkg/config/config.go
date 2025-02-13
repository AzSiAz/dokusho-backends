package config

import (
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strconv"

	"dokusho/pkg/utils"

	"github.com/google/uuid"
)

var PORT = utils.Getenv("PORT", "8080")
var LISTEN_ADDR = utils.Getenv("LISTEN_ADDR", "0.0.0.0")
var LOG_LEVEL = utils.Getenv("LOG_LEVEL", "INFO")
var USE_WHITELIST_REVERSE_PROXY = utils.Getenv("USE_WHITELIST_REVERSE_PROXY", "false") == "true"
var WHITELIST_REVERSE_PROXY_ADDR = utils.Getenv("WHITELIST_REVERSE_PROXY_ADDR", "")

var DATABASE_APP_URL = utils.Getenv("DATABASE_URL", "postgres://postgres@localhost:5433/dokusho")
var DATABASE_JOBS_URL = utils.Getenv("DATABASE_JOBS_URL", "postgres://postgres@localhost:5433/dokusho?search_path=jobs")
var DATABASE_APPLY_MIGRATIONS = utils.Getenv("DATABASE_APPLY_MIGRATIONS", "true") == "true"

var SOURCE_API_URL = utils.Getenv("SOURCE_API_URL", fmt.Sprintf("http://%s:%s", LISTEN_ADDR, PORT))
var SOURCE_USE_FLARESOLVER = utils.Getenv("SOURCE_USE_FLARESOLVER", "false") == "true"
var SOURCE_FLARESOLVER_URL = utils.Getenv("SOURCE_FLARESOLVER_URL", "")
var SOURCE_USE_MOCK = utils.Getenv("SOURCE_USE_MOCK", "true") == "true"
var SOURCE_USE_API_KEY = utils.Getenv("SOURCE_USE_API_KEY", "false") == "true"
var SOURCE_API_KEY = utils.Getenv("SOURCE_API_KEY", "")

var FILE_SERVE_URL = utils.Getenv("FILE_SERVE_URL", fmt.Sprintf("http://%s:%s", LISTEN_ADDR, PORT))
var FILE_SERVE_MOCK = utils.Getenv("FILE_SERVE_MOCK", "false") == "true"
var FILE_ROOT_DIR = utils.Getenv("FILE_ROOT_DIR", "/mnt/dokusho")

func init() {
	slog.SetLogLoggerLevel(utils.NewLogLevel(LOG_LEVEL).SlogLevel())
}

type HTTPServerBaseConfig struct {
	Port                        string
	ListenAddr                  string
	LogLevel                    string
	UseWhitelistedReverseProxy  bool
	WhitelistedReverseProxyAddr []string
}

type FileBaseConfig struct {
	FileServeURL  string
	FileServeMock bool
	FileRootDir   string
}

type SourceBaseConfig struct {
	SourceAPIURL         string
	SourceUseFlaresolver bool
	SourceFlaresolverURL string
	SourceUseMock        bool
	SourceUseAPIKey      bool
	SourceAPIKey         string
}

type DatabaseBaseConfig struct {
	DatabaseAppURL          string
	DatabaseJobsURL         string
	DatabaseApplyMigrations bool
}

type SourceConfig struct {
	*HTTPServerBaseConfig
	*SourceBaseConfig
}

func NewSourceConfig() (*SourceConfig, error) {
	bce := validateHttpServerBaseConfig()
	ssce := validateSourceServerConfig()

	err := errors.Join(bce, ssce)
	if err != nil {
		return nil, err
	}

	// TODO: Need convert dns address to ip address, usefull to allow only selected container from inside a compose network
	whitelistedAddr := utils.SplitAndTrim(WHITELIST_REVERSE_PROXY_ADDR, ",")

	return &SourceConfig{
		HTTPServerBaseConfig: &HTTPServerBaseConfig{
			Port:                        PORT,
			ListenAddr:                  LISTEN_ADDR,
			LogLevel:                    LOG_LEVEL,
			UseWhitelistedReverseProxy:  USE_WHITELIST_REVERSE_PROXY,
			WhitelistedReverseProxyAddr: whitelistedAddr,
		},
		SourceBaseConfig: &SourceBaseConfig{
			SourceAPIURL:         SOURCE_API_URL,
			SourceUseFlaresolver: SOURCE_USE_FLARESOLVER,
			SourceFlaresolverURL: SOURCE_FLARESOLVER_URL,
			SourceUseMock:        SOURCE_USE_MOCK,
			SourceUseAPIKey:      SOURCE_USE_API_KEY,
			SourceAPIKey:         SOURCE_API_KEY,
		},
	}, nil
}

func validateFileServerConfig() error {
	if FILE_ROOT_DIR == "" {
		return fmt.Errorf("FILE_ROOT_DIR is required")
	} else {
		err := os.MkdirAll(FILE_ROOT_DIR, os.ModePerm)
		if err != nil {
			return fmt.Errorf("Failed to create root dir: %w", err)
		}

		slog.Info("File root dir created, or already existing", "root_dir", FILE_ROOT_DIR)
	}

	return nil
}

func validateSourceServerConfig() error {
	if SOURCE_USE_FLARESOLVER && SOURCE_FLARESOLVER_URL == "" {
		return fmt.Errorf("SOURCE_FLARESOLVER_URL is required when SOURCE_USE_FLARESOLVER is true")
	}

	if SOURCE_USE_API_KEY && SOURCE_API_KEY == "" {
		slog.Warn("SOURCE_API_KEY is required when SOURCE_USE_API_KEY is true")
		SOURCE_API_KEY = uuid.NewString()
		slog.Info("Generated new SOURCE_API_KEY, you must set one or it will generated at every restart", "source_api_key", SOURCE_API_KEY)
	}

	return nil
}

func validateDatabaseConfig() error {
	if DATABASE_APP_URL == "" {
		return fmt.Errorf("DATABASE_APP_URL is required")
	}

	if DATABASE_JOBS_URL == "" {
		return fmt.Errorf("DATABASE_JOBS_URL is required")
	}

	return nil
}

func validateHttpServerBaseConfig() error {
	// Check if port is a valid number
	if _, err := strconv.Atoi(PORT); err != nil {
		return fmt.Errorf("PORT must be a valid number")
	}

	if USE_WHITELIST_REVERSE_PROXY && WHITELIST_REVERSE_PROXY_ADDR == "" {
		return fmt.Errorf("WHITELIST_REVERSE_PROXY_ADDR is required when USE_WHITELIST_REVERSE_PROXY is true")
	}

	if USE_WHITELIST_REVERSE_PROXY && WHITELIST_REVERSE_PROXY_ADDR != "" {
		whitelist := utils.SplitAndTrim(WHITELIST_REVERSE_PROXY_ADDR, ",")
		for _, ip := range whitelist {
			if ip == "" || ip == "::1" {
				return fmt.Errorf("WHITELIST_REVERSE_PROXY_ADDR contains an invalid IP: %s", ip)
			}
		}

		slog.Info("Using whitelisted reverse proxy address", "whitelist", whitelist)
	}

	return nil
}
