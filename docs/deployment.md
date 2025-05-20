# Connectify Deployment and Operations Guide

This guide provides instructions for deploying the Connectify application to different environments, monitoring it in production, and performing common operational tasks.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Deployment Environments](#deployment-environments)
- [Deployment Process](#deployment-process)
- [Configuration Management](#configuration-management)
- [Monitoring and Logging](#monitoring-and-logging)
- [Scaling](#scaling)
- [Backup and Recovery](#backup-and-recovery)
- [Security Considerations](#security-considerations)
- [Common Operational Tasks](#common-operational-tasks)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying the Connectify application, ensure you have the following:

- Access to the target deployment environment
- Docker installed (for containerized deployment)
- Appropriate credentials for external services (Google Calendar, Stripe, Twilio, etc.)
- SSL certificates for HTTPS (for production environments)

## Deployment Environments

Connectify supports the following deployment environments:

### Development

The development environment is used for local development and testing. It typically runs on a developer's machine and uses local or development instances of external services.

### Staging

The staging environment is a pre-production environment that mirrors the production environment as closely as possible. It's used for testing changes before they're deployed to production.

### Production

The production environment is the live environment that serves real users. It requires the highest level of reliability, security, and performance.

## Deployment Process

### Docker Deployment

Connectify can be deployed using Docker containers. Here's how to build and run the Docker containers:

1. Build the Docker image:
   ```bash
   docker build -t connectify-backend:latest .
   ```

2. Run the Docker container:
   ```bash
   docker run -d \
     --name connectify-backend \
     -p 8086:8086 \
     -e RUN_ENV=production \
     -e CONNECTIFY__SERVER__HOST=0.0.0.0 \
     -e CONNECTIFY__SERVER__PORT=8086 \
     -v /path/to/config:/app/config \
     connectify-backend:latest
   ```

### Kubernetes Deployment

For production environments, we recommend using Kubernetes for orchestration. Here's a sample Kubernetes deployment configuration:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: connectify-backend
  labels:
    app: connectify-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: connectify-backend
  template:
    metadata:
      labels:
        app: connectify-backend
    spec:
      containers:
      - name: connectify-backend
        image: connectify-backend:latest
        ports:
        - containerPort: 8086
        env:
        - name: RUN_ENV
          value: "production"
        - name: CONNECTIFY__SERVER__HOST
          value: "0.0.0.0"
        - name: CONNECTIFY__SERVER__PORT
          value: "8086"
        volumeMounts:
        - name: config-volume
          mountPath: /app/config
        - name: secrets-volume
          mountPath: /app/secrets
      volumes:
      - name: config-volume
        configMap:
          name: connectify-config
      - name: secrets-volume
        secret:
          secretName: connectify-secrets
```

### CI/CD Pipeline

We use a CI/CD pipeline to automate the deployment process. The pipeline includes the following steps:

1. **Build**: Build the application and run tests.
2. **Package**: Package the application as a Docker image.
3. **Deploy to Staging**: Deploy the Docker image to the staging environment.
4. **Test in Staging**: Run integration tests in the staging environment.
5. **Deploy to Production**: Deploy the Docker image to the production environment.

## Configuration Management

Connectify uses a layered approach to configuration management:

1. Default configuration values are defined in the code.
2. Configuration files (`config/default.yml`, `config/{RUN_ENV}.yml`) override the default values.
3. Environment variables override the values from configuration files.

For production deployments, we recommend using environment variables for sensitive information and configuration files for non-sensitive information.

### Environment Variables

Environment variables should be set in the deployment environment. For Docker deployments, use the `-e` flag to set environment variables. For Kubernetes deployments, use the `env` field in the deployment configuration.

See the [Environment Variables documentation](ENVIRONMENT_VARIABLES.md) for a list of available environment variables.

### Configuration Files

Configuration files should be mounted into the container at `/app/config`. For Docker deployments, use the `-v` flag to mount a volume. For Kubernetes deployments, use a ConfigMap to store the configuration files.

## Monitoring and Logging

### Logging

Connectify uses structured logging with the `tracing` crate. Logs are output to stdout/stderr in JSON format, which can be collected by a log aggregation service like ELK Stack or Datadog.

To enable debug logging, set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug
```

### Metrics

Connectify exposes metrics via a Prometheus endpoint at `/metrics`. These metrics can be collected by Prometheus and visualized in Grafana.

Key metrics to monitor:

- **HTTP request rate**: The number of HTTP requests per second.
- **HTTP request latency**: The time taken to process HTTP requests.
- **Error rate**: The number of errors per second.
- **External service call rate**: The number of calls to external services per second.
- **External service call latency**: The time taken for external service calls.

### Health Checks

Connectify provides health check endpoints that can be used to monitor the health of the application:

- `/health`: Returns 200 OK if the application is healthy.
- `/health/ready`: Returns 200 OK if the application is ready to serve requests.
- `/health/live`: Returns 200 OK if the application is alive.

### Alerting

We recommend setting up alerts for the following conditions:

- **High error rate**: Alert if the error rate exceeds a threshold.
- **High latency**: Alert if the request latency exceeds a threshold.
- **Low availability**: Alert if the application is not responding to health checks.
- **High resource usage**: Alert if CPU or memory usage exceeds a threshold.

## Scaling

Connectify is designed to be horizontally scalable. You can run multiple instances of the application behind a load balancer to handle increased traffic.

### Horizontal Scaling

To scale horizontally, increase the number of replicas in the Kubernetes deployment:

```bash
kubectl scale deployment connectify-backend --replicas=5
```

### Vertical Scaling

To scale vertically, increase the CPU and memory resources allocated to the containers:

```yaml
resources:
  requests:
    cpu: "500m"
    memory: "512Mi"
  limits:
    cpu: "1000m"
    memory: "1Gi"
```

### Auto-Scaling

For production environments, we recommend using Kubernetes Horizontal Pod Autoscaler (HPA) to automatically scale the application based on CPU usage or custom metrics:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: connectify-backend
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: connectify-backend
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 80
```

## Backup and Recovery

### Database Backup

If you're using a database, ensure it's backed up regularly. For example, if you're using PostgreSQL:

```bash
pg_dump -U username -d database_name > backup.sql
```

### Configuration Backup

Backup your configuration files and environment variables:

```bash
cp -r /path/to/config /path/to/backup
```

### Recovery

To recover from a backup:

1. Restore the database:
   ```bash
   psql -U username -d database_name < backup.sql
   ```

2. Restore the configuration:
   ```bash
   cp -r /path/to/backup /path/to/config
   ```

3. Restart the application:
   ```bash
   kubectl rollout restart deployment connectify-backend
   ```

## Security Considerations

### Network Security

- Use HTTPS for all external communication.
- Use a firewall to restrict access to the application.
- Use a VPN for administrative access.

### Authentication and Authorization

- Use strong authentication for administrative access.
- Implement proper authorization checks for API endpoints.
- Use OAuth 2.0 for authentication with external services.

### Secrets Management

- Store secrets in a secure secrets management system like Kubernetes Secrets or HashiCorp Vault.
- Rotate secrets regularly.
- Use environment variables for secrets, not configuration files.

### Vulnerability Scanning

- Regularly scan Docker images for vulnerabilities.
- Keep dependencies up to date.
- Follow security best practices for Rust code.

## Common Operational Tasks

### Deploying a New Version

To deploy a new version of the application:

1. Build and push the new Docker image:
   ```bash
   docker build -t connectify-backend:new-version .
   docker push connectify-backend:new-version
   ```

2. Update the Kubernetes deployment:
   ```bash
   kubectl set image deployment/connectify-backend connectify-backend=connectify-backend:new-version
   ```

### Rolling Back a Deployment

To roll back to a previous version:

```bash
kubectl rollout undo deployment/connectify-backend
```

### Viewing Logs

To view logs from the application:

```bash
kubectl logs -f deployment/connectify-backend
```

### Checking Application Status

To check the status of the application:

```bash
kubectl get pods -l app=connectify-backend
```

### Restarting the Application

To restart the application:

```bash
kubectl rollout restart deployment/connectify-backend
```

## Troubleshooting

### Common Issues

#### Application Won't Start

If the application won't start, check the logs for errors:

```bash
kubectl logs -f deployment/connectify-backend
```

Common causes:
- Missing environment variables
- Invalid configuration
- Database connection issues
- External service connection issues

#### High Latency

If the application is experiencing high latency, check:

- Resource usage (CPU, memory)
- Database query performance
- External service call performance
- Network latency

#### High Error Rate

If the application is experiencing a high error rate, check:

- Application logs for error messages
- External service availability
- Database availability
- Network connectivity

### Debugging in Production

To debug issues in production:

1. Enable debug logging:
   ```bash
   kubectl set env deployment/connectify-backend RUST_LOG=debug
   ```

2. Check the logs:
   ```bash
   kubectl logs -f deployment/connectify-backend
   ```

3. Use the health check endpoints to check the health of the application:
   ```bash
   curl https://your-app-url/health
   curl https://your-app-url/health/ready
   curl https://your-app-url/health/live
   ```

4. Check the metrics endpoint for performance issues:
   ```bash
   curl https://your-app-url/metrics
   ```

5. If necessary, create a temporary debug pod with the same image:
   ```bash
   kubectl run debug-pod --image=connectify-backend:latest --command -- sleep infinity
   kubectl exec -it debug-pod -- /bin/bash
   ```