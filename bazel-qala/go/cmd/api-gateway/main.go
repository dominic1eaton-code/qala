package main

import (
	"log"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"strconv"
	"strings"

	"qala/go-services/internal/platform/httpx"
	"qala/go-services/internal/platform/runtime"
)

type routeTarget struct {
	BaseURL string `json:"base_url"`
}

func main() {
	service := "api-gateway"
	port := runtime.EnvInt("PORT", 8080)

	routes := map[string]routeTarget{
		"users":           {BaseURL: envOr("USER_SERVICE_URL", "http://localhost:8081")},
		"sdes":            {BaseURL: envOr("SDE_SERVICE_URL", "http://localhost:8082")},
		"solutions":       {BaseURL: envOr("SDE_SERVICE_URL", "http://localhost:8082")},
		"factories":       {BaseURL: envOr("SDE_SERVICE_URL", "http://localhost:8082")},
		"workspaces":      {BaseURL: envOr("WORKSPACE_SERVICE_URL", "http://localhost:8083")},
		"pipelines":       {BaseURL: envOr("WORKFLOW_SERVICE_URL", "http://localhost:8084")},
		"artifacts":       {BaseURL: envOr("ARTIFACT_SERVICE_URL", "http://localhost:8085")},
		"data":            {BaseURL: envOr("DATA_SERVICE_URL", "http://localhost:8086")},
		"recommendations": {BaseURL: envOr("AI_SERVICE_URL", "http://localhost:8087")},
		"analyze_sde":     {BaseURL: envOr("AI_SERVICE_URL", "http://localhost:8087")},
		"threats":         {BaseURL: envOr("SECURITY_SERVICE_URL", "http://localhost:8088")},
		"policy":          {BaseURL: envOr("SECURITY_SERVICE_URL", "http://localhost:8088")},
		"scan_sde":        {BaseURL: envOr("SECURITY_SERVICE_URL", "http://localhost:8088")},
		"notify":          {BaseURL: envOr("NOTIFICATION_SERVICE_URL", "http://localhost:8089")},
		"notifications":   {BaseURL: envOr("NOTIFICATION_SERVICE_URL", "http://localhost:8089")},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/routes", func(w http.ResponseWriter, _ *http.Request) {
		httpx.WriteJSON(w, http.StatusOK, routes)
	})
	mux.HandleFunc("/api/", func(w http.ResponseWriter, r *http.Request) {
		path := strings.TrimPrefix(r.URL.Path, "/api/")
		parts := strings.SplitN(path, "/", 2)
		if len(parts) == 0 || parts[0] == "" {
			httpx.Error(w, http.StatusBadRequest, "invalid route")
			return
		}

		target, ok := routes[parts[0]]
		if !ok {
			httpx.Error(w, http.StatusNotFound, "no route configured")
			return
		}

		base, err := url.Parse(target.BaseURL)
		if err != nil {
			httpx.Error(w, http.StatusInternalServerError, "bad target URL")
			return
		}

		proxy := httputil.NewSingleHostReverseProxy(base)
		originalDirector := proxy.Director
		proxy.Director = func(req *http.Request) {
			originalDirector(req)
			remaining := path
			if len(parts) == 2 {
				remaining = parts[0] + "/" + parts[1]
			}
			req.URL.Path = "/" + remaining
			req.Host = base.Host
		}
		proxy.ServeHTTP(w, r)
	})

	addr := ":" + intToString(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func envOr(name, fallback string) string {
	if v := strings.TrimSpace(os.Getenv(name)); v != "" {
		return v
	}
	return fallback
}

func intToString(v int) string {
	return strconv.Itoa(v)
}
