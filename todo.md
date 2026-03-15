

Service discovery is the automated process of detecting devices and services on a network, crucial for microservices to dynamically find and communicate with each other in changing environments. By using a service registry (e.g., Consul, Eureka, or Kubernetes), services register their IP addresses and ports, eliminating the need for manual configuration and enabling high availability, scaling, and automatic health monitoring. 


Key Service Discovery Techniques & Patterns
Service Registry: A database that stores network locations of service instances.
Self-Registration (Client-Side Discovery): A service instance registers itself with the registry upon startup, making it responsible for its own lifecycle management.
Third-Party Registration (Server-Side Discovery): A separate service registrar handles registration, often used in orchestrated environments like Kubernetes.
Client-Side Discovery: The client queries the registry, selects an available instance, and makes the request.
Server-Side Discovery: The client sends a request to a load balancer/router (e.g., HAProxy, Nginx), which queries the registry and routes to an instance.
Sidecar Proxy: A local proxy handles discovery for the service, allowing it to remain language-agnostic while using a central key/value store


