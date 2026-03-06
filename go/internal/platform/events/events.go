package events

import "time"

const (
	TopicUserEvents         = "USER_EVENTS"
	TopicSDEEvents          = "SDE_EVENTS"
	TopicCMSEvents          = "CMS_EVENTS"
	TopicBuildEvents        = "BUILD_EVENTS"
	TopicArtifactEvents     = "ARTIFACT_EVENTS"
	TopicDataEvents         = "DATA_EVENTS"
	TopicSecurityEvents     = "SECURITY_EVENTS"
	TopicNotifications      = "NOTIFICATIONS"
	TopicAIRecommendations  = "AI_RECOMMENDATIONS"
)

type Envelope struct {
	EventID       string                 `json:"event_id"`
	EventType     string                 `json:"event_type"`
	Topic         string                 `json:"topic"`
	SourceService string                 `json:"source_service"`
	OccurredAt    time.Time              `json:"occurred_at"`
	Payload       map[string]interface{} `json:"payload"`
}
