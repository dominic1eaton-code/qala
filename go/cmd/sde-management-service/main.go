package main

import (
	"fmt"
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

type solutionType string

const (
	solutionTypeApplication solutionType = "application"
	solutionTypeSystem      solutionType = "system"
	solutionTypeGood        solutionType = "good"
	solutionTypeProduct     solutionType = "product"
	solutionTypeService     solutionType = "service"
	solutionTypePlatform    solutionType = "platform"
)

type messageKind string

const (
	messageKindEvent messageKind = "event"
	messageKindState messageKind = "state"
)

type dataType string

const (
	dataTypeBool    dataType = "bool"
	dataTypeString  dataType = "string"
	dataTypeInt     dataType = "int"
	dataTypeCustom  dataType = "custom"
	dataTypeVarchar dataType = "varchar"
	dataTypeFloat   dataType = "float"
	dataTypeDouble  dataType = "double"
	dataTypeNull    dataType = "null"
	dataTypeArray   dataType = "array"
	dataTypeChar    dataType = "char"
	dataTypeTuple   dataType = "tuple"
	dataTypeSet     dataType = "set"
	dataTypeMap     dataType = "map"
	dataTypeObject  dataType = "object"
	dataTypePointer dataType = "pointer"
	dataTypeDate    dataType = "date"
)

type solution struct {
	ID        string       `json:"id"`
	Name      string       `json:"name"`
	Type      solutionType `json:"type"`
	Systems   []systemNode `json:"systems"`
	CreatedAt time.Time    `json:"created_at"`
}

type systemNode struct {
	ID           string            `json:"id"`
	Name         string            `json:"name"`
	Applications []applicationNode `json:"applications"`
}

type applicationNode struct {
	ID        string        `json:"id"`
	Name      string        `json:"name"`
	Processes []processNode `json:"processes"`
}

type processNode struct {
	ID         string          `json:"id"`
	Name       string          `json:"name"`
	Components []componentNode `json:"components"`
}

type componentNode struct {
	ID         string          `json:"id"`
	Name       string          `json:"name"`
	Interfaces []interfaceNode `json:"interfaces"`
}

type interfaceNode struct {
	ID       string        `json:"id"`
	Name     string        `json:"name"`
	Imports  []string      `json:"imports"`
	Exports  []string      `json:"exports"`
	Messages []messageNode `json:"messages"`
}

type messageNode struct {
	ID             string          `json:"id"`
	Name           string          `json:"name"`
	Kind           messageKind     `json:"kind"`
	Imported       bool            `json:"imported"`
	Exported       bool            `json:"exported"`
	DataStructures []dataStructure `json:"data_structures"`
}

type dataStructure struct {
	ID   string     `json:"id"`
	Name string     `json:"name"`
	Data []dataItem `json:"data"`
}

type dataItem struct {
	Name       string   `json:"name"`
	Type       dataType `json:"type"`
	CustomType string   `json:"custom_type,omitempty"`
}

type sde struct {
	ID                string                 `json:"id"`
	OwnerID           string                 `json:"owner_id"`
	TemplateID        string                 `json:"template_id"`
	Status            string                 `json:"status"`
	SnapshotVersion   int                    `json:"snapshot_version"`
	DeploymentTargets []string               `json:"deployment_targets"`
	Profiles          []string               `json:"profiles"`
	Settings          map[string]string      `json:"settings"`
	Parameters        map[string]string      `json:"parameters"`
	Options           map[string]interface{} `json:"options"`
	SolutionIDs       []string               `json:"solution_ids"`
	CreatedAt         time.Time              `json:"created_at"`
}

type snapshot struct {
	ID          string    `json:"id"`
	SDEID       string    `json:"sde_id"`
	Version     int       `json:"version"`
	Description string    `json:"description"`
	CreatedAt   time.Time `json:"created_at"`
}

type solutionFactory struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	SDEIDs      []string  `json:"sde_ids"`
	CreatedAt   time.Time `json:"created_at"`
}

type serviceState struct {
	mu        sync.RWMutex
	sdes      map[string]sde
	snapshots map[string][]snapshot
	solutions map[string]solution
	factories map[string]solutionFactory
	events    []events.Envelope
}

func main() {
	service := "sde-management-service"
	port := runtime.EnvInt("PORT", 8082)
	state := &serviceState{
		sdes:      map[string]sde{},
		snapshots: map[string][]snapshot{},
		solutions: map[string]solution{},
		factories: map[string]solutionFactory{},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", runtime.HealthHandler(service))
	mux.HandleFunc("/sdes", state.handleSDEs)
	mux.HandleFunc("/sdes/", state.handleSDEByPath)
	mux.HandleFunc("/solutions", state.handleSolutions)
	mux.HandleFunc("/solutions/", state.handleSolutionByPath)
	mux.HandleFunc("/factories", state.handleFactories)
	mux.HandleFunc("/factories/", state.handleFactoryByPath)
	mux.HandleFunc("/events", state.handleEvents)

	addr := ":" + strconv.Itoa(port)
	log.Printf("[%s] listening on %s", service, addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func (s *serviceState) handleSDEs(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			OwnerID           string                 `json:"owner_id"`
			TemplateID        string                 `json:"template_id"`
			DeploymentTargets []string               `json:"deployment_targets"`
			Profiles          []string               `json:"profiles"`
			Settings          map[string]string      `json:"settings"`
			Parameters        map[string]string      `json:"parameters"`
			Options           map[string]interface{} `json:"options"`
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
			ID:                runtime.NewID("sde"),
			OwnerID:           req.OwnerID,
			TemplateID:        req.TemplateID,
			Status:            "created",
			SnapshotVersion:   0,
			DeploymentTargets: copyStringSlice(req.DeploymentTargets),
			Profiles:          copyStringSlice(req.Profiles),
			Settings:          copyStringMap(req.Settings),
			Parameters:        copyStringMap(req.Parameters),
			Options:           copyAnyMap(req.Options),
			SolutionIDs:       []string{},
			CreatedAt:         time.Now().UTC(),
		}
		s.mu.Lock()
		s.sdes[item.ID] = item
		s.emit("SDE_PROVISIONED", map[string]interface{}{
			"sde_id":             item.ID,
			"deployment_targets": item.DeploymentTargets,
		})
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
	if action == "solutions" && r.Method == http.MethodPost && len(parts) == 2 {
		s.handleSDESolutionLink(w, r, id)
		return
	}
	if action == "solutions" && r.Method == http.MethodDelete && len(parts) == 3 {
		s.handleSDESolutionUnlink(w, r, id, parts[2])
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
		linkedSolutions := make([]solution, 0, len(item.SolutionIDs))
		for _, solutionID := range item.SolutionIDs {
			if sol, exists := s.solutions[solutionID]; exists {
				linkedSolutions = append(linkedSolutions, sol)
			}
		}
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "sde not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, map[string]interface{}{
			"sde":       item,
			"snapshots": snaps,
			"solutions": linkedSolutions,
		})

	case http.MethodPatch:
		var req struct {
			Status            *string                `json:"status"`
			DeploymentTargets *[]string              `json:"deployment_targets"`
			Profiles          *[]string              `json:"profiles"`
			Settings          map[string]string      `json:"settings"`
			Parameters        map[string]string      `json:"parameters"`
			Options           map[string]interface{} `json:"options"`
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
		if req.Status != nil {
			status := strings.TrimSpace(*req.Status)
			if status != "" {
				item.Status = status
			}
		}
		if req.DeploymentTargets != nil {
			item.DeploymentTargets = copyStringSlice(*req.DeploymentTargets)
		}
		if req.Profiles != nil {
			item.Profiles = copyStringSlice(*req.Profiles)
		}
		if req.Settings != nil {
			item.Settings = copyStringMap(req.Settings)
		}
		if req.Parameters != nil {
			item.Parameters = copyStringMap(req.Parameters)
		}
		if req.Options != nil {
			item.Options = copyAnyMap(req.Options)
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
		for factoryID, factory := range s.factories {
			factory.SDEIDs = removeString(factory.SDEIDs, id)
			s.factories[factoryID] = factory
		}
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

func (s *serviceState) handleSDESolutionLink(w http.ResponseWriter, r *http.Request, sdeID string) {
	var req struct {
		SolutionID string `json:"solution_id"`
	}
	if err := httpx.ReadJSON(r, &req); err != nil {
		httpx.Error(w, http.StatusBadRequest, "invalid JSON")
		return
	}
	if strings.TrimSpace(req.SolutionID) == "" {
		httpx.Error(w, http.StatusBadRequest, "solution_id is required")
		return
	}

	s.mu.Lock()
	item, ok := s.sdes[sdeID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "sde not found")
		return
	}
	if _, ok := s.solutions[req.SolutionID]; !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "solution not found")
		return
	}
	item.SolutionIDs = appendUnique(item.SolutionIDs, req.SolutionID)
	s.sdes[sdeID] = item
	s.emit("SDE_SOLUTION_LINKED", map[string]interface{}{
		"sde_id":      sdeID,
		"solution_id": req.SolutionID,
	})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusOK, item)
}

func (s *serviceState) handleSDESolutionUnlink(w http.ResponseWriter, _ *http.Request, sdeID, solutionID string) {
	s.mu.Lock()
	item, ok := s.sdes[sdeID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "sde not found")
		return
	}
	item.SolutionIDs = removeString(item.SolutionIDs, solutionID)
	s.sdes[sdeID] = item
	s.emit("SDE_SOLUTION_UNLINKED", map[string]interface{}{
		"sde_id":      sdeID,
		"solution_id": solutionID,
	})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusOK, item)
}

func (s *serviceState) handleSolutions(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			Name    string       `json:"name"`
			Type    solutionType `json:"type"`
			Systems []systemNode `json:"systems"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}

		item := solution{
			ID:        runtime.NewID("sol"),
			Name:      req.Name,
			Type:      req.Type,
			Systems:   req.Systems,
			CreatedAt: time.Now().UTC(),
		}
		if err := normalizeAndValidateSolution(&item); err != nil {
			httpx.Error(w, http.StatusBadRequest, err.Error())
			return
		}

		s.mu.Lock()
		s.solutions[item.ID] = item
		s.emit("SOLUTION_CREATED", map[string]interface{}{
			"solution_id": item.ID,
			"type":        item.Type,
		})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusCreated, item)

	case http.MethodGet:
		s.mu.RLock()
		resp := make([]solution, 0, len(s.solutions))
		for _, item := range s.solutions {
			resp = append(resp, item)
		}
		s.mu.RUnlock()
		httpx.WriteJSON(w, http.StatusOK, resp)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleSolutionByPath(w http.ResponseWriter, r *http.Request) {
	id := strings.Trim(strings.TrimPrefix(r.URL.Path, "/solutions/"), "/")
	if id == "" {
		httpx.Error(w, http.StatusBadRequest, "solution id is required")
		return
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		item, ok := s.solutions[id]
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "solution not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, item)

	case http.MethodPatch:
		var req struct {
			Name    *string      `json:"name"`
			Type    *solutionType `json:"type"`
			Systems *[]systemNode `json:"systems"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}

		s.mu.Lock()
		item, ok := s.solutions[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "solution not found")
			return
		}
		if req.Name != nil {
			item.Name = *req.Name
		}
		if req.Type != nil {
			item.Type = *req.Type
		}
		if req.Systems != nil {
			item.Systems = *req.Systems
		}
		if err := normalizeAndValidateSolution(&item); err != nil {
			s.mu.Unlock()
			httpx.Error(w, http.StatusBadRequest, err.Error())
			return
		}
		s.solutions[id] = item
		s.emit("SOLUTION_UPDATED", map[string]interface{}{
			"solution_id": id,
			"type":        item.Type,
		})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, item)

	case http.MethodDelete:
		s.mu.Lock()
		if _, ok := s.solutions[id]; !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "solution not found")
			return
		}
		delete(s.solutions, id)
		for sdeID, sdeRecord := range s.sdes {
			sdeRecord.SolutionIDs = removeString(sdeRecord.SolutionIDs, id)
			s.sdes[sdeID] = sdeRecord
		}
		s.emit("SOLUTION_DELETED", map[string]interface{}{"solution_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleFactories(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		var req struct {
			Name        string   `json:"name"`
			Description string   `json:"description"`
			SDEIDs      []string `json:"sde_ids"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}
		name := strings.TrimSpace(req.Name)
		if name == "" {
			httpx.Error(w, http.StatusBadRequest, "name is required")
			return
		}

		s.mu.Lock()
		sdeIDs := uniqueStrings(req.SDEIDs)
		for _, sdeID := range sdeIDs {
			if _, ok := s.sdes[sdeID]; !ok {
				s.mu.Unlock()
				httpx.Error(w, http.StatusBadRequest, "all sde_ids must reference existing sdes")
				return
			}
		}
		item := solutionFactory{
			ID:          runtime.NewID("sf"),
			Name:        name,
			Description: req.Description,
			SDEIDs:      sdeIDs,
			CreatedAt:   time.Now().UTC(),
		}
		s.factories[item.ID] = item
		s.emit("SOLUTION_FACTORY_CREATED", map[string]interface{}{
			"factory_id": item.ID,
			"sde_count":  len(item.SDEIDs),
		})
		s.mu.Unlock()

		httpx.WriteJSON(w, http.StatusCreated, item)

	case http.MethodGet:
		s.mu.RLock()
		resp := make([]solutionFactory, 0, len(s.factories))
		for _, item := range s.factories {
			resp = append(resp, item)
		}
		s.mu.RUnlock()
		httpx.WriteJSON(w, http.StatusOK, resp)

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleFactoryByPath(w http.ResponseWriter, r *http.Request) {
	parts := strings.Split(strings.Trim(strings.TrimPrefix(r.URL.Path, "/factories/"), "/"), "/")
	if len(parts) == 0 || parts[0] == "" {
		httpx.Error(w, http.StatusBadRequest, "factory id is required")
		return
	}
	id := parts[0]
	action := ""
	if len(parts) > 1 {
		action = parts[1]
	}

	if action == "sdes" && r.Method == http.MethodPost && len(parts) == 2 {
		s.handleFactorySDELink(w, r, id)
		return
	}
	if action == "sdes" && r.Method == http.MethodDelete && len(parts) == 3 {
		s.handleFactorySDEUnlink(w, r, id, parts[2])
		return
	}
	if action != "" {
		httpx.Error(w, http.StatusNotFound, "unsupported action")
		return
	}

	switch r.Method {
	case http.MethodGet:
		s.mu.RLock()
		item, ok := s.factories[id]
		attached := make([]sde, 0, len(item.SDEIDs))
		for _, sdeID := range item.SDEIDs {
			if sdeRecord, exists := s.sdes[sdeID]; exists {
				attached = append(attached, sdeRecord)
			}
		}
		s.mu.RUnlock()
		if !ok {
			httpx.Error(w, http.StatusNotFound, "factory not found")
			return
		}
		httpx.WriteJSON(w, http.StatusOK, map[string]interface{}{
			"factory": item,
			"sdes":    attached,
		})

	case http.MethodPatch:
		var req struct {
			Name        *string   `json:"name"`
			Description *string   `json:"description"`
			SDEIDs      *[]string `json:"sde_ids"`
		}
		if err := httpx.ReadJSON(r, &req); err != nil {
			httpx.Error(w, http.StatusBadRequest, "invalid JSON")
			return
		}

		s.mu.Lock()
		item, ok := s.factories[id]
		if !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "factory not found")
			return
		}
		if req.Name != nil {
			name := strings.TrimSpace(*req.Name)
			if name == "" {
				s.mu.Unlock()
				httpx.Error(w, http.StatusBadRequest, "name cannot be empty")
				return
			}
			item.Name = name
		}
		if req.Description != nil {
			item.Description = *req.Description
		}
		if req.SDEIDs != nil {
			candidate := uniqueStrings(*req.SDEIDs)
			for _, sdeID := range candidate {
				if _, exists := s.sdes[sdeID]; !exists {
					s.mu.Unlock()
					httpx.Error(w, http.StatusBadRequest, "all sde_ids must reference existing sdes")
					return
				}
			}
			item.SDEIDs = candidate
		}
		s.factories[id] = item
		s.emit("SOLUTION_FACTORY_UPDATED", map[string]interface{}{"factory_id": id})
		s.mu.Unlock()

		httpx.WriteJSON(w, http.StatusOK, item)

	case http.MethodDelete:
		s.mu.Lock()
		if _, ok := s.factories[id]; !ok {
			s.mu.Unlock()
			httpx.Error(w, http.StatusNotFound, "factory not found")
			return
		}
		delete(s.factories, id)
		s.emit("SOLUTION_FACTORY_DELETED", map[string]interface{}{"factory_id": id})
		s.mu.Unlock()
		httpx.WriteJSON(w, http.StatusOK, map[string]string{"status": "deleted"})

	default:
		httpx.Error(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

func (s *serviceState) handleFactorySDELink(w http.ResponseWriter, r *http.Request, factoryID string) {
	var req struct {
		SDEID string `json:"sde_id"`
	}
	if err := httpx.ReadJSON(r, &req); err != nil {
		httpx.Error(w, http.StatusBadRequest, "invalid JSON")
		return
	}
	if strings.TrimSpace(req.SDEID) == "" {
		httpx.Error(w, http.StatusBadRequest, "sde_id is required")
		return
	}

	s.mu.Lock()
	item, ok := s.factories[factoryID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "factory not found")
		return
	}
	if _, exists := s.sdes[req.SDEID]; !exists {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "sde not found")
		return
	}
	item.SDEIDs = appendUnique(item.SDEIDs, req.SDEID)
	s.factories[factoryID] = item
	s.emit("SOLUTION_FACTORY_SDE_LINKED", map[string]interface{}{
		"factory_id": factoryID,
		"sde_id":     req.SDEID,
	})
	s.mu.Unlock()

	httpx.WriteJSON(w, http.StatusOK, item)
}

func (s *serviceState) handleFactorySDEUnlink(w http.ResponseWriter, _ *http.Request, factoryID, sdeID string) {
	s.mu.Lock()
	item, ok := s.factories[factoryID]
	if !ok {
		s.mu.Unlock()
		httpx.Error(w, http.StatusNotFound, "factory not found")
		return
	}
	item.SDEIDs = removeString(item.SDEIDs, sdeID)
	s.factories[factoryID] = item
	s.emit("SOLUTION_FACTORY_SDE_UNLINKED", map[string]interface{}{
		"factory_id": factoryID,
		"sde_id":     sdeID,
	})
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

func normalizeAndValidateSolution(item *solution) error {
	item.Name = strings.TrimSpace(item.Name)
	item.Type = solutionType(strings.ToLower(strings.TrimSpace(string(item.Type))))
	if item.Name == "" {
		return fmt.Errorf("name is required")
	}
	if !isValidSolutionType(item.Type) {
		return fmt.Errorf("invalid type; allowed types: application, system, good, product, service, platform")
	}

	for systemIdx := range item.Systems {
		system := &item.Systems[systemIdx]
		if strings.TrimSpace(system.ID) == "" {
			system.ID = runtime.NewID("sys")
		}
		system.Name = strings.TrimSpace(system.Name)
		if system.Name == "" {
			return fmt.Errorf("systems[%d].name is required", systemIdx)
		}
		for appIdx := range system.Applications {
			app := &system.Applications[appIdx]
			if strings.TrimSpace(app.ID) == "" {
				app.ID = runtime.NewID("app")
			}
			app.Name = strings.TrimSpace(app.Name)
			if app.Name == "" {
				return fmt.Errorf("systems[%d].applications[%d].name is required", systemIdx, appIdx)
			}
			for processIdx := range app.Processes {
				proc := &app.Processes[processIdx]
				if strings.TrimSpace(proc.ID) == "" {
					proc.ID = runtime.NewID("proc")
				}
				proc.Name = strings.TrimSpace(proc.Name)
				if proc.Name == "" {
					return fmt.Errorf("systems[%d].applications[%d].processes[%d].name is required", systemIdx, appIdx, processIdx)
				}
				for componentIdx := range proc.Components {
					component := &proc.Components[componentIdx]
					if strings.TrimSpace(component.ID) == "" {
						component.ID = runtime.NewID("cmp")
					}
					component.Name = strings.TrimSpace(component.Name)
					if component.Name == "" {
						return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].name is required", systemIdx, appIdx, processIdx, componentIdx)
					}
					for interfaceIdx := range component.Interfaces {
						iface := &component.Interfaces[interfaceIdx]
						if strings.TrimSpace(iface.ID) == "" {
							iface.ID = runtime.NewID("ifc")
						}
						iface.Name = strings.TrimSpace(iface.Name)
						if iface.Name == "" {
							return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].name is required", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx)
						}
						for messageIdx := range iface.Messages {
							msg := &iface.Messages[messageIdx]
							if strings.TrimSpace(msg.ID) == "" {
								msg.ID = runtime.NewID("msg")
							}
							msg.Name = strings.TrimSpace(msg.Name)
							msg.Kind = messageKind(strings.ToLower(strings.TrimSpace(string(msg.Kind))))
							if msg.Name == "" {
								return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].name is required", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx)
							}
							if !isValidMessageKind(msg.Kind) {
								return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].kind must be event or state", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx)
							}
							if !msg.Imported && !msg.Exported {
								return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d] must be imported or exported", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx)
							}
							for structureIdx := range msg.DataStructures {
								structure := &msg.DataStructures[structureIdx]
								if strings.TrimSpace(structure.ID) == "" {
									structure.ID = runtime.NewID("ds")
								}
								structure.Name = strings.TrimSpace(structure.Name)
								if structure.Name == "" {
									return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].data_structures[%d].name is required", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx, structureIdx)
								}
								for dataIdx := range structure.Data {
									field := &structure.Data[dataIdx]
									field.Name = strings.TrimSpace(field.Name)
									field.Type = dataType(strings.ToLower(strings.TrimSpace(string(field.Type))))
									if field.Name == "" {
										return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].data_structures[%d].data[%d].name is required", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx, structureIdx, dataIdx)
									}
									if !isValidDataType(field.Type) {
										return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].data_structures[%d].data[%d].type is invalid", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx, structureIdx, dataIdx)
									}
									if field.Type == dataTypeCustom && strings.TrimSpace(field.CustomType) == "" {
										return fmt.Errorf("systems[%d].applications[%d].processes[%d].components[%d].interfaces[%d].messages[%d].data_structures[%d].data[%d].custom_type is required for custom type", systemIdx, appIdx, processIdx, componentIdx, interfaceIdx, messageIdx, structureIdx, dataIdx)
									}
								}
							}
						}
					}
				}
			}
		}
	}

	return nil
}

func isValidSolutionType(t solutionType) bool {
	switch t {
	case solutionTypeApplication, solutionTypeSystem, solutionTypeGood, solutionTypeProduct, solutionTypeService, solutionTypePlatform:
		return true
	default:
		return false
	}
}

func isValidMessageKind(k messageKind) bool {
	switch k {
	case messageKindEvent, messageKindState:
		return true
	default:
		return false
	}
}

func isValidDataType(t dataType) bool {
	switch t {
	case dataTypeBool, dataTypeString, dataTypeInt, dataTypeCustom, dataTypeVarchar, dataTypeFloat, dataTypeDouble, dataTypeNull, dataTypeArray, dataTypeChar, dataTypeTuple, dataTypeSet, dataTypeMap, dataTypeObject, dataTypePointer, dataTypeDate:
		return true
	default:
		return false
	}
}

func appendUnique(items []string, value string) []string {
	for _, item := range items {
		if item == value {
			return items
		}
	}
	return append(items, value)
}

func uniqueStrings(items []string) []string {
	if len(items) == 0 {
		return []string{}
	}
	seen := map[string]struct{}{}
	out := make([]string, 0, len(items))
	for _, item := range items {
		item = strings.TrimSpace(item)
		if item == "" {
			continue
		}
		if _, ok := seen[item]; ok {
			continue
		}
		seen[item] = struct{}{}
		out = append(out, item)
	}
	return out
}

func removeString(items []string, target string) []string {
	if len(items) == 0 {
		return []string{}
	}
	filtered := make([]string, 0, len(items))
	for _, item := range items {
		if item != target {
			filtered = append(filtered, item)
		}
	}
	return filtered
}

func copyStringSlice(values []string) []string {
	if len(values) == 0 {
		return []string{}
	}
	out := make([]string, 0, len(values))
	for _, value := range values {
		trimmed := strings.TrimSpace(value)
		if trimmed == "" {
			continue
		}
		out = append(out, trimmed)
	}
	return out
}

func copyStringMap(values map[string]string) map[string]string {
	if values == nil {
		return map[string]string{}
	}
	out := make(map[string]string, len(values))
	for key, value := range values {
		out[key] = value
	}
	return out
}

func copyAnyMap(values map[string]interface{}) map[string]interface{} {
	if values == nil {
		return map[string]interface{}{}
	}
	out := make(map[string]interface{}, len(values))
	for key, value := range values {
		out[key] = value
	}
	return out
}
