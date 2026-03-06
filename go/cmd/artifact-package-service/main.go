package main

import (
	"log"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"qala/go-services/internal/platform/events"
	"qala/go-services/internal/platform/httpx"
	"qala/go-services/internal/platform/runtime"
)

type artifact struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Version   string    `json:"version"`
	SDEID     string    `json:"sde_id"`
	FilePath  string    `json:"file_path"`
	CreatedAt time.Time `json:"created_at"`
}

type serviceState struct {
	mu        sync.RWMutex
	artifacts map[string]artifact
	events    []events.Envelope
}

func main() {
	service := "artifact-package-service"
	port := runtime.EnvInt("PORT", 8085)
	state := &serviceState{artifacts: map[string]artifact{}}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/artifacts/upload", state.handleUpload)
	mux.HandleFunc("/artifacts", state.handleArtifacts)
	mux.HandleFunc("/artifacts/", state.handleArtifactByPath)
	mux.HandleFunc("/events", state.handleEvents)

	addr := ":" + strconv.Itoa(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func (s *serviceState) handleUpload(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req struct {
		Name     string `json:"name"`
		Version  string `json:"version"`
		SDEID    string `json:"sde_id"`
		FilePath string `json:"file_path"`
	}
	if err := httpx.ReadJSON(r, &req); err != nil {
		httpx.Error(w, http.StatusBadRequest, "invalid JSON")
		return
	}
	if strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.SDEID) == "" || strings.TrimSpace(req.FilePath) == "" {
		httpx.Error(w, http.StatusBadRequest, "name, sde_id and file_path are required")
		return
	}
	version := req.Version
	if strings.TrimSpace(version) == "" {
		version = "v1"
	}

	item := artifact{
		ID:        runtime.NewID("art"),
		Name:      req.Name,
		Version:   version,
		SDEID:     req.SDEID,
		FilePath:  req.FilePath,
		CreatedAt: time.Now().UTC(),
	}

	s.mu.Lock()
	s.artifacts[item.ID] = item
	s.emit("ARTIFACT_STORED", map[string]interface{}{
		"artifact_id": item.ID,
		"sde_id":      item.SDEID,
		"version":     item.Version,
	})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusCreated, item)
}

func (s *serviceState) handleArtifacts(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}
	s.mu.RLock()
	resp := make([]artifact, 0, len(s.artifacts))
	for _, a := range s.artifacts {
		resp = append(resp, a)
	}
	s.mu.RUnlock()
	httpx.WriteJSON(w, http.StatusOK, resp)
}

func (s *serviceState) handleArtifactByPath(w http.ResponseWriter, r *http.Request) {
	parts := strings.Split(strings.Trim(strings.TrimPrefix(r.URL.Path, "/artifacts/"), "/"), "/")
	if len(parts) == 0 || parts[0] == "" {
		httpx.Error(w, http.StatusBadRequest, "artifact id is required")
		return
	}
	id := parts[0]
	action := ""
	if len(parts) > 1 {
		action = parts[1]
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		item, ok := s.artifacts[id]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "artifact not found")
			return
		}
		if action == "download" {
			httpx.WriteJSON(w, http.StatusOK, map[string]interface{}{
				"artifact_id": id,
				"download_url": "/artifacts/" + id + "/binary",
				"metadata":    item,
			})
			return
		}
		httpx.WriteJSON(w, http.StatusOK, item)

	case http.MethodDelete:
		if action != "" {
			httpx.Error(w, http.StatusNotFound, "unsupported action")
			return
		}
		s.mu.Lock()
		_, ok := s.artifacts[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "artifact not found")
			return
		}
		delete(s.artifacts, id)
		s.emit("ARTIFACT_DELETED", map[string]interface{}{"artifact_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) emit(eventType string, payload map[string]interface{}) {
	event := events.Envelope{
		EventID:       runtime.NewID("evt"),
		EventType:     eventType,
		Topic:         events.TopicArtifactEvents,
		SourceService: "artifact-package-service",
		OccurredAt:    time.Now().UTC(),
		Payload:       payload,
	}
	s.events = append(s.events, event)
	runtime.EmitEvent(event)
}

func (s *serviceState) handleEvents(w http.ResponseWriter, _ *http.Request) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	httpx.WriteJSON(w, http.StatusOK, s.events)
}
