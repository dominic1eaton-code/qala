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

type user struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Email     string    `json:"email"`
	CreatedAt time.Time `json:"created_at"`
}

type serviceState struct {
	mu     sync.RWMutex
	users  map[string]user
	events []events.Envelope
}

func main() {
	service := "user-identity-service"
	port := runtime.EnvInt("PORT", 8081)
	state := &serviceState{users: map[string]user{}}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/users", state.handleUsers)
	mux.HandleFunc("/users/", state.handleUserByID)
	mux.HandleFunc("/events", state.handleEvents)

	addr := ":" + strconv.Itoa(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func (s *serviceState) handleUsers(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			Name  string `json:"name"`
			Email string `json:"email"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		if strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Email) == "" {
			httpx.Error(w, http.StatusBadRequest, "name and email are required")
			return
		}

		u := user{
			ID:        runtime.NewID("usr"),
			Name:      req.Name,
			Email:     req.Email,
			CreatedAt: time.Now().UTC(),
		}

		s.mu.Lock()
		s.users[u.ID] = u
		event := events.Envelope{
			EventID:       runtime.NewID("evt"),
			EventType:     "USER_CREATED",
			Topic:         events.TopicUserEvents,
			SourceService: "user-identity-service",
			OccurredAt:    time.Now().UTC(),
			Payload: map[string]interface{}{
				"user_id": u.ID,
				"email":   u.Email,
			},
		}
		s.events = append(s.events, event)
		s.mu.Unlock()
		runtime.EmitEvent(event)
		httpx.WriteJSON(w, http.StatusCreated, u)

	case http.MethodGet:
		s.mu.RLock()
		resp := make([]user, 0, len(s.users))
		for _, u := range s.users {
			resp = append(resp, u)
		}
		s.mu.RUnlock()
		httpx.WriteJSON(w, http.StatusOK, resp)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleUserByID(w http.ResponseWriter, r *http.Request) {
	id := strings.TrimPrefix(r.URL.Path, "/users/")
	if strings.TrimSpace(id) == "" {
		httpx.Error(w, http.StatusBadRequest, "user id is required")
		return
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		u, ok := s.users[id]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "user not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, u)

	case http.MethodPut:
		var req struct {
			Name  string `json:"name"`
			Email string `json:"email"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}

		s.mu.Lock()
		u, ok := s.users[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "user not found")
			return
		}
		if strings.TrimSpace(req.Name) != "" {
			u.Name = req.Name
		}
		if strings.TrimSpace(req.Email) != "" {
			u.Email = req.Email
		}
		s.users[id] = u
		event := events.Envelope{
			EventID:       runtime.NewID("evt"),
			EventType:     "USER_UPDATED",
			Topic:         events.TopicUserEvents,
			SourceService: "user-identity-service",
			OccurredAt:    time.Now().UTC(),
			Payload: map[string]interface{}{
				"user_id": id,
			},
		}
		s.events = append(s.events, event)
		s.mu.Unlock()
		runtime.EmitEvent(event)
		httpx.WriteJSON(w, http.StatusOK, u)

	case http.MethodDelete:
		s.mu.Lock()
		_, ok := s.users[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "user not found")
			return
		}
		delete(s.users, id)
		event := events.Envelope{
			EventID:       runtime.NewID("evt"),
			EventType:     "USER_DELETED",
			Topic:         events.TopicUserEvents,
			SourceService: "user-identity-service",
			OccurredAt:    time.Now().UTC(),
			Payload: map[string]interface{}{
				"user_id": id,
			},
		}
		s.events = append(s.events, event)
		s.mu.Unlock()
		runtime.EmitEvent(event)
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleEvents(w http.ResponseWriter, _ *http.Request) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	httpx.WriteJSON(w, http.StatusOK, s.events)
}
