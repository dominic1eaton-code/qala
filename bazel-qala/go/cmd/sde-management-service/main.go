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

type sde struct {
	ID              string    `json:"id"`
	OwnerID         string    `json:"owner_id"`
	TemplateID      string    `json:"template_id"`
	Status          string    `json:"status"`
	SnapshotVersion int       `json:"snapshot_version"`
	CreatedAt       time.Time `json:"created_at"`
}

type snapshot struct {
	ID          string    `json:"id"`
	SDEID       string    `json:"sde_id"`
	Version     int       `json:"version"`
	Description string    `json:"description"`
	CreatedAt   time.Time `json:"created_at"`
}

type serviceState struct {
	mu        sync.RWMutex
	sdes      map[string]sde
	snapshots map[string][]snapshot
	events    []events.Envelope
}

func main() {
	service := "sde-management-service"
	port := runtime.EnvInt("PORT", 8082)
	state := &serviceState{
		sdes:      map[string]sde{},
		snapshots: map[string][]snapshot{},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/sdes", state.handleSDEs)
	mux.HandleFunc("/sdes/", state.handleSDEByPath)
	mux.HandleFunc("/events", state.handleEvents)

	addr := ":" + strconv.Itoa(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func (s *serviceState) handleSDEs(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			OwnerID    string `json:"owner_id"`
			TemplateID string `json:"template_id"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		if strings.TrimSpace(req.OwnerID) == "" || strings.TrimSpace(req.TemplateID) == "" {
			httpx.Error(w, http.StatusBadRequest, "owner_id and template_id are required")
			return
		}

		item := sde{
			ID:              runtime.NewID("sde"),
			OwnerID:         req.OwnerID,
			TemplateID:      req.TemplateID,
			Status:          "created",
			SnapshotVersion: 0,
			CreatedAt:       time.Now().UTC(),
		}
		s.mu.Lock()
		s.sdes[item.ID] = item
		s.emit("SDE_PROVISIONED", map[string]interface{}{"sde_id": item.ID})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusCreated, item)

	case http.MethodGet:
		s.mu.RLock()
		resp := make([]sde, 0, len(s.sdes))
		for _, v := range s.sdes {
			resp = append(resp, v)
		}
		s.mu.RUnlock()
		httpx.WriteJSON(w, http.StatusOK, resp)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleSDEByPath(w http.ResponseWriter, r *http.Request) {
	parts := strings.Split(strings.Trim(strings.TrimPrefix(r.URL.Path, "/sdes/"), "/"), "/")
	if len(parts) == 0 || parts[0] == "" {
		httpx.Error(w, http.StatusBadRequest, "sde id is required")
		return
	}
	id := parts[0]
	action := ""
	if len(parts) > 1 {
		action = parts[1]
	}

	if action == "snapshot" && r.Method == http.MethodPost {
		s.handleSnapshot(w, r, id)
		return
	}
	if action == "rollback" && r.Method == http.MethodPost {
		s.handleRollback(w, r, id)
		return
	}
	if action != "" {
		httpx.Error(w, http.StatusNotFound, "unsupported action")
		return
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		item, ok := s.sdes[id]
		snaps := s.snapshots[id]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "sde not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, map[string]interface{}{
			"sde":       item,
			"snapshots": snaps,
		})

	case http.MethodPatch:
		var req struct {
			Status string `json:"status"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}

		s.mu.Lock()
		item, ok := s.sdes[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "sde not found")
			return
		}
		if strings.TrimSpace(req.Status) != "" {
			item.Status = req.Status
		}
		s.sdes[id] = item
		s.emit("SDE_UPDATED", map[string]interface{}{"sde_id": id, "status": item.Status})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, item)

	case http.MethodDelete:
		s.mu.Lock()
		_, ok := s.sdes[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "sde not found")
			return
		}
		delete(s.sdes, id)
		delete(s.snapshots, id)
		s.emit("SDE_DELETED", map[string]interface{}{"sde_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleSnapshot(w http.ResponseWriter, r *http.Request, sdeID string) {
	var req struct {
		Description string `json:"description"`
	}
	if err := httpx.ReadJSON(r, &req); err != nil {
		httpx.Error(w, http.StatusBadRequest, "invalid JSON")
		return
	}

	s.mu.Lock()
	item, ok := s.sdes[sdeID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "sde not found")
		return
	}
	item.SnapshotVersion++
	s.sdes[sdeID] = item

	snap := snapshot{
		ID:          runtime.NewID("snap"),
		SDEID:       sdeID,
		Version:     item.SnapshotVersion,
		Description: req.Description,
		CreatedAt:   time.Now().UTC(),
	}
	s.snapshots[sdeID] = append(s.snapshots[sdeID], snap)
	s.emit("SNAPSHOT_CREATED", map[string]interface{}{"sde_id": sdeID, "snapshot_id": snap.ID, "version": snap.Version})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusCreated, snap)
}

func (s *serviceState) handleRollback(w http.ResponseWriter, r *http.Request, sdeID string) {
	var req struct {
		SnapshotID string `json:"snapshot_id"`
	}
	if err := httpx.ReadJSON(r, &req); err != nil {
		httpx.Error(w, http.StatusBadRequest, "invalid JSON")
		return
	}

	s.mu.Lock()
	item, ok := s.sdes[sdeID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "sde not found")
		return
	}
	snaps := s.snapshots[sdeID]
	found := false
	for _, snap := range snaps {
		if snap.ID == req.SnapshotID {
			item.SnapshotVersion = snap.Version
			item.Status = "rolled_back"
			found = true
			break
		}
	}
	if !found {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "snapshot not found")
		return
	}
	s.sdes[sdeID] = item
	s.emit("SDE_ROLLBACK", map[string]interface{}{"sde_id": sdeID, "snapshot_id": req.SnapshotID})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusOK, item)
}

func (s *serviceState) emit(eventType string, payload map[string]interface{}) {
	event := events.Envelope{
		EventID:       runtime.NewID("evt"),
		EventType:     eventType,
		Topic:         events.TopicSDEEvents,
		SourceService: "sde-management-service",
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
