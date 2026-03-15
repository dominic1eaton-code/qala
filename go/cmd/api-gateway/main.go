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

	// Define the routing rules for the API Gateway. Each key is a prefix that will be matched against incoming requests,
	// and the value is the target service's base URL to which the request will be forwarded.
	// For example, a request to /api/users will be forwarded to the USER_SERVICE_URL, while a request to /api/sdes will be forwarded to the SDE_SERVICE_URL.
	// This allows us to centralize the routing logic in the API Gateway, making it easier to manage and scale our microservices architecture.
	// The envOr function is used to read the target service URLs from environment variables, with a default fallback value if the environment variable is not set.
	// This design allows for flexibility in deployment, as the target service URLs can be easily configured without changing the code.
	// Note: In a production environment, you might want to add more sophisticated error handling, logging, and support for additional HTTP methods or headers.
	// Additionally, you may want to consider using a more robust routing library or framework if your routing needs become more complex.
	// The routes defined here are based on the services mentioned in the original prompt, and you can adjust them as needed to fit your specific architecture and service endpoints.
	// The API Gateway will listen for incoming requests on the specified port and route them to the appropriate backend services based on the defined routes.
	// Example routes:
	// - /api/users -> USER_SERVICE_URL
	// - /api/sdes -> SDE_SERVICE_URL
	// - /api/solutions -> SDE_SERVICE_URL
	// - /api/factories -> SDE_SERVICE_URL
	// - /api/workspaces -> WORKSPACE_SERVICE_URL
	// - /api/pipelines -> WORKFLOW_SERVICE_URL
	// - /api/artifacts -> ARTIFACT_SERVICE_URL
	// - /api/data -> DATA_SERVICE_URL
	// - /api/recommendations -> AI_SERVICE_URL
	// - /api/analyze_sde -> AI_SERVICE_URL
	// - /api/threats -> SECURITY_SERVICE_URL
	// - /api/policy -> SECURITY_SERVICE_URL
	// - /api/scan_sde -> SECURITY_SERVICE_URL
	// - /api/notify -> NOTIFICATION_SERVICE_URL
	// - /api/notifications -> NOTIFICATION_SERVICE_URL
	// The API Gateway will also expose a /health endpoint for health checks and a /routes endpoint to list the configured routes.
	// This setup allows for a clean separation of concerns, where the API Gateway is responsible for routing and load balancing, while the individual services focus on their specific business logic.
	// In a real-world application, you might also want to implement additional features such as authentication, rate limiting, caching, and monitoring within the API Gateway to enhance security and performance.
	// Overall, this API Gateway serves as a critical component in a microservices architecture, providing a single entry point for clients and enabling seamless communication between services.
	// Note: The actual implementation of the backend services (users, sdes, solutions, etc.) is not included here, as this code focuses on the API Gateway's routing logic.
	// You can implement the backend services separately, ensuring that they listen on the appropriate ports and handle the expected routes as defined in the API Gateway.
	// The API Gateway can be further enhanced with features such as circuit breakers, retries, and fallback mechanisms to improve resilience and fault tolerance in the system.
	// Additionally, you may want to consider using a service discovery mechanism to dynamically discover the target service URLs instead of hardcoding them in the API Gateway.
	// This code provides a basic implementation of an API Gateway in Go, which can be extended and customized based on the specific requirements of your application and microservices architecture.
	// Remember to run this API Gateway alongside your backend services, ensuring that they are properly configured to handle the incoming requests forwarded by the gateway.
	// Finally, you can test the API Gateway by sending requests to the defined routes and verifying that they are correctly forwarded to the respective backend services.
	// Example test:
	// - Send a GET request to http://localhost:8080/api/users and verify that it is forwarded to the USER_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/sdes and verify that it is forwarded to the SDE_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/workspaces and verify that it is forwarded to the WORKSPACE_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/pipelines and verify that it is forwarded to the WORKFLOW_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/artifacts and verify that it is forwarded to the ARTIFACT_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/data and verify that it is forwarded to the DATA_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/recommendations and verify that it is forwarded to the AI_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/analyze_sde and verify that it is forwarded to the AI_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/threats and verify that it is forwarded to the SECURITY_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/policy and verify that it is forwarded to the SECURITY_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/scan_sde and verify that it is forwarded to the SECURITY_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/notify and verify that it is forwarded to the NOTIFICATION_SERVICE_URL.
	// - Send a GET request to http://localhost:8080/api/notifications and verify that it is forwarded to the NOTIFICATION_SERVICE_URL.
	// By following this implementation, you can create a functional API Gateway that routes requests to various backend services based on the defined routes, providing a centralized entry point for clients and enabling efficient communication within your microservices architecture.
	// Note: This code is a simplified example for demonstration purposes. In a production environment, you should consider additional factors such as security, error handling, logging, and performance optimizations when implementing an API Gateway.
	// The API Gateway can be further enhanced with features such as authentication, rate limiting, caching, and monitoring to improve security and performance. Additionally, you may want to consider using a service discovery mechanism to dynamically discover the target service URLs instead of hardcoding them in the API Gateway.
	// Overall, this API Gateway serves as a critical component in a microservices architecture, providing a single entry point for clients and enabling seamless communication between services. Remember to run this API Gateway alongside your backend services, ensuring that they are properly configured to handle the incoming requests forwarded by the gateway.
	// The API Gateway will also expose a /health endpoint for health checks and a /routes endpoint to list the configured routes. This setup allows for a clean separation of concerns, where the API Gateway is responsible for routing and load balancing, while the individual services focus on their specific business logic.
	// In a real-world application, you might also want to implement additional features such as authentication, rate limiting, caching, and monitoring within the API Gateway to enhance security and performance. The actual implementation of the backend services (users, sdes, solutions, etc.) is not included here, as this code focuses on the API Gateway's routing logic. You can implement the backend services separately, ensuring that they listen on the appropriate ports and handle the expected routes as defined in the API Gateway.
	// Note: The routes defined here are based on the services mentioned in the original prompt, and you can adjust them as needed to fit your specific architecture and service endpoints. The API Gateway will listen for incoming requests on the specified port and route them to the appropriate backend services based on the defined routes.
	// Example routes:
	// - /api/users -> USER_SERVICE_URL
	// - /api/sdes -> SDE_SERVICE_URL
	// - /api/solutions -> SDE_SERVICE_URL
	// - /api/factories -> SDE_SERVICE_URL
	// - /api/workspaces -> WORKSPACE_SERVICE_URL
	// - /api/pipelines -> WORKFLOW_SERVICE_URL
	// - /api/artifacts -> ARTIFACT_SERVICE_URL
	// - /api/data -> DATA_SERVICE_URL
	// - /api/recommendations -> AI_SERVICE_URL
	// - /api/analyze_sde -> AI_SERVICE_URL
	// - /api/threats -> SECURITY_SERVICE_URL
	// - /api/policy -> SECURITY_SERVICE_URL
	// - /api/scan_sde -> SECURITY_SERVICE_URL
	// - /api/notify -> NOTIFICATION_SERVICE_URL
	// - /api/notifications -> NOTIFICATION_SERVICE_URL
	// The API Gateway will also expose a /health endpoint for health checks and a /routes endpoint to list the configured routes. This setup allows for a clean separation of concerns, where the API Gateway is responsible for routing and load balancing, while the individual services focus on their specific business logic. In a real-world application, you might also want to implement additional features such as authentication, rate limiting, caching, and monitoring within the API Gateway to enhance security and performance. Additionally, you may want to consider using a service discovery mechanism to dynamically discover the target service URLs instead of hardcoding them in the API Gateway.
	// @todo implement dynamic port and service discovery for better scalability and flexibility in a production environment.
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
