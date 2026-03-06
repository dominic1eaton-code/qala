package runtime

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
	"sync/atomic"
	"time"

	"qala/go-services/internal/platform/events"
	"qala/go-services/internal/platform/httpx"
)

var counter uint64

func NewID(prefix string) string {
	next := atomic.AddUint64(&counter, 1)
	return fmt.Sprintf("%s-%d-%d", prefix, time.Now().UnixNano(), next)
}

func EnvInt(name string, fallback int) int {
	raw := os.Getenv(name)
	if raw == "" {
		return fallback
	}
	port, err := strconv.Atoi(raw)
	if err != nil {
		return fallback
	}
	return port
}

func HealthHandler(service string) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		httpx.WriteJSON(w, http.StatusOK, map[string]string{
			"service": service,
			"status":  "ok",
		})
	}
}

func EmitEvent(event events.Envelope) {
	log.Printf("[%s] topic=%s type=%s payload=%v", event.SourceService, event.Topic, event.EventType, event.Payload)
	forwardToKernel(event)
}

func forwardToKernel(event events.Envelope) {
	url := os.Getenv("KERNEL_EVENT_INGEST_URL")
	if url == "" {
		url = "http://localhost:7000/v1/kernel/events"
	}
	body, err := json.Marshal(event)
	if err != nil {
		log.Printf("[runtime] event marshal failed: %v", err)
		return
	}
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		log.Printf("[runtime] event request build failed: %v", err)
		return
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[runtime] kernel forward failed: %v", err)
		return
	}
	_ = resp.Body.Close()
}
