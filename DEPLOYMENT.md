# Deployment Guide

This guide covers deploying the Raito Proving Service in production environments.

## Prerequisites

- Docker 20.10+
- Kubernetes 1.20+ (for K8s deployment)
- Load balancer with SSL termination
- Monitoring stack (Prometheus + Grafana)

## Environment Variables

| Variable   | Description              | Default       | Required |
| ---------- | ------------------------ | ------------- | -------- |
| `PORT`     | Server port              | `8080`        | No       |
| `RUST_LOG` | Log level                | `info`        | No       |
| `WORKERS`  | Number of worker threads | Auto-detected | No       |

## Docker Deployment

### Build Image

```bash
docker build -t raito-proving-service:latest .
```

### Run Container

```bash
docker run -d \
  --name raito-proving-service \
  -p 8080:8080 \
  -e RUST_LOG=info \
  --restart unless-stopped \
  raito-proving-service:latest
```

### Docker Compose

```yaml
version: '3.8'

services:
  raito-proving-service:
    image: ghcr.io/raito/proving-service:latest
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info
      PORT: 8080
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped
    networks:
      - raito-network

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    networks:
      - raito-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    networks:
      - raito-network

networks:
  raito-network:
    driver: bridge
```

## Kubernetes Deployment

### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: raito
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: raito-proving-service
  namespace: raito
  labels:
    app: raito-proving-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: raito-proving-service
  template:
    metadata:
      labels:
        app: raito-proving-service
    spec:
      containers:
      - name: service
        image: ghcr.io/raito/proving-service:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: PORT
          value: "8080"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /healthz
            port: http
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /healthz
            port: http
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          allowPrivilegeEscalation: false
          runAsNonRoot: true
          runAsUser: 1001
          capabilities:
            drop:
            - ALL
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: raito-proving-service
  namespace: raito
spec:
  selector:
    app: raito-proving-service
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  type: ClusterIP
```

### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: raito-proving-service
  namespace: raito
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.raito.wtf
    secretName: raito-tls
  rules:
  - host: api.raito.wtf
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: raito-proving-service
            port:
              number: 80
```

### HorizontalPodAutoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: raito-proving-service-hpa
  namespace: raito
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: raito-proving-service
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'raito-proving-service'
    static_configs:
      - targets: ['raito-proving-service:8080']
    metrics_path: /metrics
    scrape_interval: 10s

rule_files:
  - "raito_alerts.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Grafana Dashboard

Key metrics to monitor:

1. **HTTP Metrics**
   - Request rate (requests/second)
   - Response time (P50, P95, P99)
   - Error rate (4xx, 5xx responses)

2. **System Metrics**
   - CPU usage
   - Memory usage
   - Disk I/O

3. **Application Metrics**
   - Active connections
   - Thread pool utilization
   - Heap memory usage

### Alerts

```yaml
# raito_alerts.yml
groups:
- name: raito-proving-service
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: High error rate on Raito Proving Service

  - alert: HighResponseTime
    expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High response time on Raito Proving Service

  - alert: ServiceDown
    expr: up{job="raito-proving-service"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: Raito Proving Service is down
```

## Security Considerations

### Network Security

1. Use TLS termination at load balancer
2. Restrict ingress to necessary ports only
3. Use network policies in Kubernetes
4. Enable firewall rules

### Container Security

1. Run as non-root user (UID 1001)
2. Use distroless/minimal base images
3. Scan images for vulnerabilities
4. Keep dependencies updated

### Secrets Management

1. Use Kubernetes secrets for sensitive data
2. Rotate credentials regularly
3. Use service mesh for inter-service communication
4. Enable audit logging

## Performance Tuning

### Application Level

1. **Thread Pool Size**: Adjust based on CPU cores
2. **Connection Limits**: Set appropriate limits
3. **Timeouts**: Configure reasonable timeouts
4. **Caching**: Enable response caching where appropriate

### Infrastructure Level

1. **Resource Limits**: Set appropriate CPU/memory limits
2. **Load Balancing**: Use sticky sessions if needed
3. **CDN**: Use CDN for static content and proof files
4. **Database**: Optimize database connections and queries

### Scaling Strategy

1. **Horizontal Scaling**: Use HPA based on CPU/memory
2. **Vertical Scaling**: Increase resources for single instances
3. **Regional Deployment**: Deploy in multiple regions
4. **Cache Layer**: Add Redis/Memcached for caching

## Backup and Recovery

### Data Backup

1. **Mock Data**: Version control in Git
2. **Proof Files**: Backup to S3/GCS
3. **Configuration**: Store in ConfigMaps/Secrets
4. **Logs**: Centralized logging with retention

### Disaster Recovery

1. **Multi-region deployment**
2. **Automated failover**
3. **Regular DR testing**
4. **Recovery time objectives (RTO) < 15 minutes**

## Maintenance

### Rolling Updates

```bash
kubectl set image deployment/raito-proving-service \
  service=ghcr.io/raito/proving-service:v1.1.0 \
  -n raito
```

### Health Checks

- Liveness probe: `/healthz`
- Readiness probe: `/healthz`
- Startup probe: `/healthz` (for slow starting containers)

### Log Management

- Structured JSON logging
- Log aggregation with ELK stack or similar
- Log retention policies
- Alert on error patterns 