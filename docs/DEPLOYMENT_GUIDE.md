# 🚀 DeAI Platform Deployment Guide

## Table of Contents

- [Overview](#overview)
- [Pre-deployment Checklist](#pre-deployment-checklist)
- [Environment Setup](#environment-setup)
- [Mainnet Deployment](#mainnet-deployment)
- [Post-deployment Verification](#post-deployment-verification)
- [Monitoring Setup](#monitoring-setup)
- [Troubleshooting](#troubleshooting)
- [Rollback Procedures](#rollback-procedures)

## Overview

This guide provides step-by-step instructions for deploying the DeAI platform to production (mainnet). The deployment process is fully automated and includes comprehensive testing, security validation, and monitoring setup.

### Deployment Architecture

```mermaid
graph TB
    subgraph "CI/CD Pipeline"
        GH[GitHub Repository]
        GHA[GitHub Actions]
        Tests[Automated Testing]
        Build[Build & Package]
    end
    
    subgraph "Staging Environment"
        STG_K8S[Staging Kubernetes]
        STG_DB[Staging Database]
        STG_Tests[Integration Tests]
    end
    
    subgraph "Production Environment"
        PROD_K8S[Production Kubernetes]
        PROD_DB[Production Database]
        PROD_SC[Smart Contract]
        PROD_MON[Monitoring Stack]
    end
    
    GH --> GHA
    GHA --> Tests
    Tests --> Build
    Build --> STG_K8S
    STG_K8S --> STG_Tests
    STG_Tests --> PROD_K8S
    PROD_K8S --> PROD_MON
```

## Pre-deployment Checklist

### 🔒 Security Requirements

- [ ] **Security audit completed** with no critical issues
- [ ] **SSL certificates** obtained and configured
- [ ] **API keys and secrets** securely stored in vault
- [ ] **Access controls** configured (RBAC, firewall rules)
- [ ] **Backup procedures** tested and verified

### 🏗️ Infrastructure Requirements

- [ ] **Kubernetes cluster** provisioned and configured
- [ ] **Database** setup with replication and backups
- [ ] **Load balancers** configured with health checks
- [ ] **Monitoring stack** deployed and configured
- [ ] **DNS records** configured for all services

### 💰 NEAR Protocol Requirements

- [ ] **NEAR account** created with sufficient balance (100+ NEAR)
- [ ] **Smart contract** compiled and tested
- [ ] **NEAR CLI** installed and authenticated
- [ ] **Contract deployment** tested on testnet

### 🧪 Testing Requirements

- [ ] **Unit tests** passing (>95% coverage)
- [ ] **Integration tests** passing (>90% coverage)
- [ ] **Load tests** completed successfully
- [ ] **Security tests** passed
- [ ] **End-to-end tests** validated

## Environment Setup

### Required Environment Variables

Create a `.env.production` file with the following variables:

```bash
# NEAR Configuration
DEAI_MAINNET_ACCOUNT=deai-compute.near
DEAI_DEPLOYER_KEY=your-deployer-private-key
NEAR_RPC_URL=https://rpc.mainnet.near.org

# Infrastructure
KUBERNETES_CLUSTER=deai-production
DOCKER_REGISTRY=gcr.io/deai-platform
DOCKER_REGISTRY_USERNAME=_json_key
DOCKER_REGISTRY_PASSWORD=your-service-account-key

# Database
DATABASE_URL=postgresql://user:pass@prod-db:5432/deai
REDIS_URL=redis://prod-redis:6379

# Security
JWT_SECRET=your-jwt-secret
API_ENCRYPTION_KEY=your-encryption-key

# Monitoring
PROMETHEUS_URL=https://prometheus.deai.network
GRAFANA_URL=https://grafana.deai.network
SLACK_WEBHOOK_URL=your-slack-webhook-url
DISCORD_WEBHOOK_URL=your-discord-webhook-url

# Domain Configuration
API_DOMAIN=api.deai.network
WEB_DOMAIN=app.deai.network
DASHBOARD_DOMAIN=monitor.deai.network
```

### Required Tools Installation

```bash
# Install required tools
curl -fsSL https://get.docker.com | sh
curl -s https://raw.githubusercontent.com/kubernetes/helm/master/scripts/get-helm-3 | bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
npm install -g @near-cli/cli

# Verify installations
docker --version
kubectl version --client
helm version
near --version
cargo --version
```

## Mainnet Deployment

### Step 1: Clone and Prepare Repository

```bash
# Clone the repository
git clone https://github.com/deai-platform/deai.git
cd deai

# Checkout the release branch
git checkout release/v1.0.0

# Load environment variables
source .env.production
```

### Step 2: Pre-deployment Validation

```bash
# Run comprehensive tests
./scripts/run-all-tests.sh

# Validate configuration
./scripts/validate-config.sh

# Check infrastructure readiness
./scripts/check-infrastructure.sh
```

### Step 3: Execute Deployment

```bash
# Make deployment script executable
chmod +x deployment/mainnet-deploy.sh

# Run deployment (this takes 30-60 minutes)
./deployment/mainnet-deploy.sh
```

### Step 4: Monitor Deployment Progress

The deployment script provides real-time progress updates:

```bash
🚀 Starting DeAI Platform Mainnet Deployment
==========================================
✅ Prerequisites check passed
✅ Pre-deployment validation completed
🔨 Building all components...
✅ Smart contract built successfully
✅ API gateway built successfully
✅ Node client built successfully
✅ Web dashboard built successfully
🐳 Building Docker images...
✅ All Docker images built successfully
📤 Pushing Docker images to registry...
✅ All images pushed successfully
🌐 Deploying smart contract to mainnet...
✅ Smart contract deployed: deai-compute.near
☸️ Deploying to Kubernetes...
✅ Kubernetes deployment completed
📊 Setting up monitoring...
✅ Monitoring and alerting configured
🔍 Running post-deployment verification...
✅ All verification checks passed
🎉 DeAI Platform mainnet deployment completed successfully!
```

## Deployment Components

### Smart Contract Deployment

The smart contract is deployed to NEAR mainnet with the following configuration:

```rust
// Contract initialization
DeAICompute::new(owner_id: "deai-compute.near")

// Initial configuration
min_stake: 1 NEAR
max_nodes: 500
task_timeout: 300 seconds
reward_rate: 10 DEAI per task
```

### Kubernetes Services

| Service | Replicas | Resources | Endpoints |
|---------|----------|-----------|-----------|
| **API Gateway** | 3 | 1 CPU, 2GB RAM | `api.deai.network` |
| **Web Dashboard** | 2 | 0.5 CPU, 1GB RAM | `app.deai.network` |
| **Task Manager** | 2 | 1 CPU, 2GB RAM | Internal |
| **Node Manager** | 2 | 0.5 CPU, 1GB RAM | Internal |
| **PostgreSQL** | 1 (Primary) + 2 (Replicas) | 2 CPU, 8GB RAM | Internal |
| **Redis** | 3 (Cluster) | 1 CPU, 2GB RAM | Internal |
| **Prometheus** | 1 | 1 CPU, 4GB RAM | `monitor.deai.network` |
| **Grafana** | 1 | 0.5 CPU, 1GB RAM | `dashboard.deai.network` |

### Load Balancer Configuration

```yaml
# Application Load Balancer
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: deai-platform-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - api.deai.network
    - app.deai.network
    secretName: deai-platform-tls
  rules:
  - host: api.deai.network
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 8080
  - host: app.deai.network
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: web-dashboard
            port:
              number: 3000
```

## Post-deployment Verification

### Automated Verification Tests

The deployment script automatically runs verification tests:

```bash
# Health checks
✅ API Gateway health check: 200 OK
✅ Web Dashboard health check: 200 OK
✅ Database connectivity: Connected
✅ Redis connectivity: Connected
✅ Smart contract verification: Active

# Performance checks
✅ API response time: 250ms (target: <1s)
✅ Task assignment latency: 2.3s (target: <5s)
✅ Load balancer: 3 endpoints active
✅ SSL certificates: Valid until 2024-12-31

# Security checks
✅ Authentication: Working
✅ Rate limiting: Active
✅ HTTPS redirect: Enabled
✅ Security headers: Configured
```

### Manual Verification Steps

1. **Test API Endpoints**

```bash
# Test API health
curl https://api.deai.network/health

# Test authentication
curl -X POST https://api.deai.network/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"test"}'

# Test task submission
curl -X POST https://api.deai.network/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"test","input":"hello"}'
```

2. **Verify Smart Contract**

```bash
# Check contract state
near state deai-compute.near --networkId mainnet

# Test contract functions
near view deai-compute.near get_task_count --networkId mainnet
near view deai-compute.near get_active_nodes --networkId mainnet
```

3. **Test Web Dashboard**

- Navigate to `https://app.deai.network`
- Verify login functionality
- Test task submission
- Check real-time updates

## Monitoring Setup

### Grafana Dashboards

Access monitoring dashboards at `https://monitor.deai.network`:

| Dashboard | Purpose | Key Metrics |
|-----------|---------|-------------|
| **System Overview** | High-level system status | Uptime, throughput, errors |
| **API Performance** | API gateway metrics | Response times, request rates |
| **Node Network** | Node health and performance | Active nodes, task distribution |
| **Token Economics** | Financial metrics | Token supply, rewards, volume |
| **Infrastructure** | Kubernetes and hardware | CPU, memory, disk usage |

### Alert Configuration

Critical alerts are automatically configured:

| Alert | Threshold | Notification |
|-------|-----------|-------------|
| **API Gateway Down** | >1 minute | Immediate |
| **High Error Rate** | >5% for 5 minutes | 2 minutes |
| **Low Node Count** | <10 active nodes | 5 minutes |
| **Database Issues** | Connection failures | Immediate |
| **High Latency** | >2s for 10 minutes | 10 minutes |

### Log Aggregation

Logs are centralized using the ELK stack:

```bash
# View recent logs
kubectl logs -f deployment/api-gateway -n deai-mainnet

# Search logs in Kibana
# Navigate to logs.deai.network
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Smart Contract Deployment Fails

**Symptom**: Contract deployment returns "insufficient funds" error

**Solution**:
```bash
# Check account balance
near state $DEAI_MAINNET_ACCOUNT --networkId mainnet

# Add more NEAR if needed
near send your-account.near $DEAI_MAINNET_ACCOUNT 10 --networkId mainnet
```

#### 2. Kubernetes Pods Not Starting

**Symptom**: Pods stuck in "Pending" or "CrashLoopBackOff" state

**Diagnosis**:
```bash
# Check pod status
kubectl get pods -n deai-mainnet

# View pod logs
kubectl logs <pod-name> -n deai-mainnet

# Describe pod for events
kubectl describe pod <pod-name> -n deai-mainnet
```

**Common Solutions**:
- **Insufficient resources**: Scale cluster or reduce resource requests
- **Image pull errors**: Check Docker registry credentials
- **Config issues**: Verify ConfigMaps and Secrets

#### 3. Database Connection Issues

**Symptom**: Applications can't connect to database

**Diagnosis**:
```bash
# Test database connectivity
kubectl exec -it deployment/api-gateway -n deai-mainnet -- \
  psql $DATABASE_URL -c "SELECT 1;"
```

**Solutions**:
- Check database credentials in Kubernetes secrets
- Verify network policies allow database access
- Ensure database service is running

#### 4. SSL Certificate Issues

**Symptom**: HTTPS not working or certificate errors

**Diagnosis**:
```bash
# Check certificate status
kubectl get certificates -n deai-mainnet

# View cert-manager logs
kubectl logs -n cert-manager deployment/cert-manager
```

**Solutions**:
- Verify DNS records point to load balancer
- Check Let's Encrypt rate limits
- Ensure cert-manager has proper permissions

### Health Check Commands

```bash
# Comprehensive health check script
#!/bin/bash

echo "🔍 Running health checks..."

# API Gateway
if curl -f https://api.deai.network/health > /dev/null 2>&1; then
    echo "✅ API Gateway: Healthy"
else
    echo "❌ API Gateway: Down"
fi

# Web Dashboard
if curl -f https://app.deai.network > /dev/null 2>&1; then
    echo "✅ Web Dashboard: Healthy"
else
    echo "❌ Web Dashboard: Down"
fi

# Database
if kubectl exec deployment/api-gateway -n deai-mainnet -- \
   psql $DATABASE_URL -c "SELECT 1;" > /dev/null 2>&1; then
    echo "✅ Database: Connected"
else
    echo "❌ Database: Connection failed"
fi

# Smart Contract
if near view deai-compute.near get_task_count --networkId mainnet > /dev/null 2>&1; then
    echo "✅ Smart Contract: Active"
else
    echo "❌ Smart Contract: Issues detected"
fi
```

## Rollback Procedures

### Automatic Rollback

The deployment script includes automatic rollback on failure:

```bash
# If deployment fails, automatic rollback is triggered
warning "Rolling back deployment due to failure..."

# Rollback Kubernetes deployment
helm rollback deai-mainnet --namespace deai-mainnet

# Note: Smart contract rollback requires manual intervention
warning "Smart contract deployment cannot be automatically rolled back"
```

### Manual Rollback

If manual rollback is needed:

```bash
# 1. Rollback Kubernetes deployment
helm rollback deai-mainnet --namespace deai-mainnet

# 2. Rollback Docker images
kubectl set image deployment/api-gateway \
  api-gateway=gcr.io/deai-platform/api-gateway:previous-tag \
  -n deai-mainnet

# 3. Verify rollback
kubectl rollout status deployment/api-gateway -n deai-mainnet

# 4. Update DNS if needed (for major rollbacks)
# Point domains back to previous infrastructure
```

### Smart Contract Rollback

⚠️ **Important**: Smart contracts cannot be rolled back once deployed to mainnet.

**Mitigation strategies**:
1. **Pause functionality**: Use admin functions to pause contract operations
2. **Deploy new version**: Deploy a new contract version with fixes
3. **Migrate state**: Implement state migration if necessary

```bash
# Pause contract operations (if implemented)
near call deai-compute.near pause_operations \
  --accountId $DEAI_MAINNET_ACCOUNT \
  --networkId mainnet

# Deploy new version to different account
near deploy --wasmFile new-contract.wasm \
  --accountId deai-compute-v2.near \
  --networkId mainnet
```

## Performance Optimization

### Post-deployment Tuning

After deployment, monitor and optimize:

1. **Database Performance**
```sql
-- Add indexes for common queries
CREATE INDEX CONCURRENTLY idx_tasks_status ON tasks(status);
CREATE INDEX CONCURRENTLY idx_nodes_active ON nodes(is_active);

-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM tasks WHERE status = 'pending';
```

2. **Cache Optimization**
```bash
# Monitor Redis performance
kubectl exec -it redis-0 -n deai-mainnet -- redis-cli info memory

# Adjust cache TTL based on usage patterns
```

3. **Auto-scaling Configuration**
```yaml
# Fine-tune auto-scaling thresholds
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 20
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

## Security Hardening

### Post-deployment Security Tasks

1. **Enable Security Monitoring**
```bash
# Install security monitoring tools
helm install falco falcosecurity/falco --namespace falco-system --create-namespace

# Configure intrusion detection
kubectl apply -f security/network-policies.yaml
```

2. **Rotate Secrets**
```bash
# Schedule regular secret rotation
# This should be done every 90 days
kubectl create secret generic api-secrets \
  --from-literal=jwt-secret=$(openssl rand -base64 32) \
  --dry-run=client -o yaml | kubectl apply -f -
```

3. **Enable Audit Logging**
```yaml
# Configure Kubernetes audit logging
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: RequestResponse
  resources:
  - group: ""
    resources: ["secrets", "configmaps"]
```

## Maintenance Procedures

### Regular Maintenance Tasks

| Task | Frequency | Description |
|------|-----------|-------------|
| **Backup Verification** | Daily | Verify database backups are successful |
| **Security Updates** | Weekly | Apply security patches to all components |
| **Performance Review** | Weekly | Review metrics and optimize if needed |
| **Certificate Renewal** | Monthly | Check SSL certificate expiration |
| **Capacity Planning** | Monthly | Review resource usage and plan scaling |
| **Disaster Recovery Test** | Quarterly | Test backup restoration procedures |

### Maintenance Scripts

```bash
# Daily backup verification
#!/bin/bash
BACKUP_DATE=$(date -d "yesterday" +%Y-%m-%d)
if kubectl exec postgresql-0 -n deai-mainnet -- \
   pg_basebackup --list | grep $BACKUP_DATE; then
    echo "✅ Backup for $BACKUP_DATE verified"
else
    echo "❌ Backup for $BACKUP_DATE missing"
    # Send alert
fi

# Weekly security updates
#!/bin/bash
kubectl patch deployment api-gateway -n deai-mainnet \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"api-gateway","image":"gcr.io/deai-platform/api-gateway:latest"}]}}}}'

kubectl rollout status deployment/api-gateway -n deai-mainnet
```

## Support and Escalation

### Incident Response

| Severity | Response Time | Escalation |
|----------|---------------|------------|
| **P0 - Critical** | 15 minutes | Immediate escalation to all on-call |
| **P1 - High** | 1 hour | Escalate to primary on-call |
| **P2 - Medium** | 4 hours | Standard escalation |
| **P3 - Low** | 24 hours | Normal business hours |

### Contact Information

- **Primary On-call**: +1-XXX-XXX-XXXX
- **Technical Lead**: tech-lead@deai.network
- **DevOps Team**: devops@deai.network
- **Emergency**: emergency@deai.network

### Documentation Updates

Keep this deployment guide updated:

1. **After each deployment**, update version numbers and configurations
2. **When procedures change**, update the relevant sections
3. **After incidents**, update troubleshooting section with lessons learned
4. **Quarterly reviews**, validate all procedures and update as needed

---

This deployment guide ensures a smooth, secure, and monitored deployment of the DeAI platform to production. Following these procedures will result in a highly available, scalable, and maintainable production environment.