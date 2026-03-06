package main

import (
	"log"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"sync"
	"time"

	"qala/go-services/internal/platform/events"
	"qala/go-services/internal/platform/httpx"
	"qala/go-services/internal/platform/runtime"
)

type workspace struct {
	ID        string    `json:"id"`
	OwnerID   string    `json:"owner_id"`
	Name      string    `json:"name"`
	Type      string    `json:"type"`
	CreatedAt time.Time `json:"created_at"`
}

type fileContent struct {
	ID         string    `json:"id"`
	WorkspaceID string   `json:"workspace_id"`
	FilePath   string    `json:"file_path"`
	Content    string    `json:"content"`
	Version    int       `json:"version"`
	UpdatedAt  time.Time `json:"updated_at"`
}

type serviceState struct {
	mu         sync.RWMutex
	workspaces map[string]workspace
	files      map[string]map[string]fileContent
	events     []events.Envelope
}

func main() {
	service := "workspace-cms-service"
	port := runtime.EnvInt("PORT", 8083)
	state := &serviceState{
		workspaces: map[string]workspace{},
		files:      map[string]map[string]fileContent{},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/workspaces", state.handleWorkspaces)
	mux.HandleFunc("/workspaces/", state.handleWorkspaceByPath)
	mux.HandleFunc("/events", state.handleEvents)

	addr := ":" + strconv.Itoa(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func (s *serviceState) handleWorkspaces(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			OwnerID string `json:"owner_id"`
			Name    string `json:"name"`
			Type    string `json:"type"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		if strings.TrimSpace(req.OwnerID) == "" || strings.TrimSpace(req.Name) == "" {
			httpx.Error(w, http.StatusBadRequest, "owner_id and name are required")
			return
		}
		kind := req.Type
		if strings.TrimSpace(kind) == "" {
			kind = "personal"
		}

		item := workspace{
			ID:        runtime.NewID("ws"),
			OwnerID:   req.OwnerID,
			Name:      req.Name,
			Type:      kind,
			CreatedAt: time.Now().UTC(),
		}
		s.mu.Lock()
		s.workspaces[item.ID] = item
		s.files[item.ID] = map[string]fileContent{}
		s.emit("WORKSPACE_CREATED", map[string]interface{}{"workspace_id": item.ID})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusCreated, item)

	case http.MethodGet:
		s.mu.RLock()
		resp := make([]workspace, 0, len(s.workspaces))
		for _, ws := range s.workspaces {
			resp = append(resp, ws)
		}
		s.mu.RUnlock()
		httpx.WriteJSON(w, http.StatusOK, resp)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleWorkspaceByPath(w http.ResponseWriter, r *http.Request) {
	trimmed := strings.Trim(strings.TrimPrefix(r.URL.Path, "/workspaces/"), "/")
	parts := strings.Split(trimmed, "/")
	if len(parts) == 0 || parts[0] == "" {
		httpx.Error(w, http.StatusBadRequest, "workspace id is required")
		return
	}
	id := parts[0]

	if len(parts) > 1 && parts[1] == "content" {
		s.handleContentPath(w, r, id, parts[2:])
		return
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		ws, ok := s.workspaces[id]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "workspace not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, ws)

	case http.MethodPut:
		var req struct {
			Name string `json:"name"`
			Type string `json:"type"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		s.mu.Lock()
		ws, ok := s.workspaces[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "workspace not found")
			return
		}
		if strings.TrimSpace(req.Name) != "" {
			ws.Name = req.Name
		}
		if strings.TrimSpace(req.Type) != "" {
			ws.Type = req.Type
		}
		s.workspaces[id] = ws
		s.emit("WORKSPACE_UPDATED", map[string]interface{}{"workspace_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, ws)

	case http.MethodDelete:
		s.mu.Lock()
		_, ok := s.workspaces[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "workspace not found")
			return
		}
		delete(s.workspaces, id)
		delete(s.files, id)
		s.emit("WORKSPACE_DELETED", map[string]interface{}{"workspace_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleContentPath(w http.ResponseWriter, r *http.Request, workspaceID string, remainder []string) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			FilePath string `json:"file_path"`
			Content  string `json:"content"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		if strings.TrimSpace(req.FilePath) == "" {
			httpx.Error(w, http.StatusBadRequest, "file_path is required")
			return
		}

		s.mu.Lock()
		if _, ok := s.workspaces[workspaceID]; !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "workspace not found")
			return
		}
		if s.files[workspaceID] == nil {
			s.files[workspaceID] = map[string]fileContent{}
		}
		current, exists := s.files[workspaceID][req.FilePath]
		nextVersion := 1
		if exists {
			nextVersion = current.Version + 1
		}
		entry := fileContent{
			ID:          runtime.NewID("file"),
			WorkspaceID: workspaceID,
			FilePath:    req.FilePath,
			Content:     req.Content,
			Version:     nextVersion,
			UpdatedAt:   time.Now().UTC(),
		}
		s.files[workspaceID][req.FilePath] = entry
		s.emit("CONTENT_UPDATED", map[string]interface{}{
			"workspace_id": workspaceID,
			"file_path":    req.FilePath,
			"version":      nextVersion,
		})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusCreated, entry)

	case http.MethodGet:
		if len(remainder) == 0 {
			httpx.Error(w, http.StatusBadRequest, "file path is required")
			return
		}
		filePath, err := url.PathUnescape(strings.Join(remainder, "/"))
		if err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid file path")
			return
		}
		s.mu.RLock()
		files := s.files[workspaceID]
		entry, ok := files[filePath]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "file not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, entry)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) emit(eventType string, payload map[string]interface{}) {
	event := events.Envelope{
		EventID:       runtime.NewID("evt"),
		EventType:     eventType,
		Topic:         events.TopicCMSEvents,
		SourceService: "workspace-cms-service",
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
