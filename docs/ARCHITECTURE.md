# 🏗️ DeAI Platform Architecture

## Table of Contents
- [System Overview](#system-overview)
- [Component Architecture](#component-architecture)
- [Data Flow](#data-flow)
- [Smart Contract Architecture](#smart-contract-architecture)
- [API Design](#api-design)
- [Security Architecture](#security-architecture)
- [Scalability Design](#scalability-design)
- [Deployment Architecture](#deployment-architecture)

## System Overview

DeAI is built as a modern, distributed system leveraging microservices architecture with blockchain integration. The platform is designed for high availability, scalability, and security.

### Key Architectural Principles

- **Microservices Architecture**: Modular, loosely coupled services
- **Event-Driven Design**: Asynchronous communication via message queues
- **Horizontal Scalability**: Auto-scaling based on demand
- **Fault Tolerance**: Graceful degradation and self-healing
- **Zero-Trust Security**: Every request verified and authenticated

## Component Architecture

### 🌐 Frontend Layer

```mermaid
graph TB
    subgraph "Frontend Applications"
        WD[Web Dashboard<br/>React + TypeScript]
        MA[Mobile App<br/>React Native]
        CLI[CLI Tool<br/>Rust]
    end
    
    subgraph "Developer Tools"
        SDK_PY[Python SDK]
        SDK_JS[JavaScript SDK]
        SDK_RS[Rust SDK]
        API_DOC[API Documentation]
    end
    
    WD --> API_GW
    MA --> API_GW
    CLI --> API_GW
    SDK_PY --> API_GW
    SDK_JS --> API_GW
    SDK_RS --> API_GW
```

### ⚙️ Backend Services

```mermaid
graph TB
    subgraph "API Layer"
        API_GW[API Gateway<br/>Rust Actix-Web]
        WS[WebSocket Server<br/>Real-time Updates]
        AUTH[Auth Service<br/>JWT + OAuth]
    end
    
    subgraph "Core Services"
        TM[Task Manager<br/>Task Orchestration]
        NM[Node Manager<br/>Node Coordination]
        RM[Reward Manager<br/>Token Distribution]
        MM[Monitoring<br/>Metrics Collection]
    end
    
    subgraph "Data Services"
        DB[PostgreSQL<br/>Primary Database]
        CACHE[Redis<br/>Caching Layer]
        QUEUE[Message Queue<br/>Task Distribution]
        STORAGE[Object Storage<br/>File Management]
    end
    
    API_GW --> TM
    API_GW --> NM
    API_GW --> RM
    TM --> DB
    TM --> CACHE
    TM --> QUEUE
    NM --> DB
    MM --> STORAGE
```

### 🔗 Blockchain Layer

```mermaid
graph TB
    subgraph "NEAR Protocol"
        SC[DeAI Smart Contract<br/>Task Coordination]
        TOKEN[Token Contract<br/>NEP-141 Standard]
        RF[Ref Finance<br/>DEX Integration]
    end
    
    subgraph "Off-Chain Services"
        BRIDGE[Blockchain Bridge<br/>State Synchronization]
        ORACLE[Price Oracle<br/>Token Price Feed]
    end
    
    SC --> TOKEN
    SC --> RF
    SC --> BRIDGE
    RF --> ORACLE
```

## Data Flow

### Task Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant TaskMgr
    participant Contract
    participant Node
    participant Rewards
    
    User->>API: Submit Task
    API->>TaskMgr: Process Task
    TaskMgr->>Contract: Register Task
    Contract->>Node: Assign Task
    Node->>Node: Execute AI Task
    Node->>Contract: Submit Result
    Contract->>Rewards: Distribute Tokens
    Rewards->>Node: Send Rewards
    API->>User: Return Result
```

### Node Registration Flow

```mermaid
sequenceDiagram
    participant Node
    participant NodeMgr
    participant Contract
    participant Rewards
    
    Node->>NodeMgr: Register Request
    NodeMgr->>Contract: Stake Tokens
    Contract->>Contract: Validate Stake
    Contract->>NodeMgr: Confirm Registration
    NodeMgr->>Node: Registration Success
    
    loop Heartbeat
        Node->>NodeMgr: Send Heartbeat
        NodeMgr->>Contract: Update Status
    end
    
    loop Task Assignment
        Contract->>Node: Assign Task
        Node->>Contract: Submit Result
        Contract->>Rewards: Calculate Rewards
    end
```

## Smart Contract Architecture

### Contract Structure

```rust
pub struct DeAICompute {
    // Node management
    pub nodes: HashMap<AccountId, NodeInfo>,
    
    // Task management  
    pub tasks: VecDeque<Task>,
    pub completed_tasks: HashMap<u64, Task>,
    pub task_counter: u64,
    
    // Token economics
    pub token: FungibleToken,
    pub total_rewards_distributed: Balance,
    
    // Configuration
    pub min_stake: NearToken,
    pub owner_id: AccountId,
}
```

### Key Functions

| Function | Purpose | Gas Limit |
|----------|---------|-----------|
| `register_node` | Node registration with stake | 50 TGas |
| `submit_task` | Task submission by users | 30 TGas |
| `submit_result` | Result submission by nodes | 100 TGas |
| `heartbeat` | Node status update | 10 TGas |
| `ft_transfer` | Token transfers | 20 TGas |

### State Management

```mermaid
stateDiagram-v2
    [*] --> Pending: Task Submitted
    Pending --> Assigned: Node Available
    Assigned --> InProgress: Node Starts
    InProgress --> Completed: Result Submitted
    InProgress --> Failed: Timeout/Error
    Completed --> [*]: Rewards Distributed
    Failed --> Pending: Reassign Available
```

## API Design

### RESTful API Structure

```
/api/v1/
├── auth/
│   ├── login
│   ├── register
│   └── refresh
├── tasks/
│   ├── {id}
│   ├── submit
│   └── cancel
├── nodes/
│   ├── register
│   ├── {id}/status
│   └── stats
├── user/
│   ├── profile
│   ├── api-keys
│   └── billing
└── network/
    ├── stats
    └── health
```

### WebSocket Events

```json
{
  "task_update": {
    "task_id": "string",
    "status": "pending|assigned|completed|failed",
    "progress": "number",
    "estimated_completion": "timestamp"
  },
  "node_status": {
    "node_id": "string",
    "status": "online|offline|maintenance",
    "performance_metrics": "object"
  },
  "network_stats": {
    "active_nodes": "number",
    "pending_tasks": "number",
    "throughput": "number"
  }
}
```

## Security Architecture

### Defense in Depth

```mermaid
graph TB
    subgraph "Perimeter Security"
        WAF[Web Application Firewall]
        DDoS[DDoS Protection]
        CDN[Content Delivery Network]
    end
    
    subgraph "Application Security"
        AuthZ[Authorization<br/>RBAC + ABAC]
        AuthN[Authentication<br/>JWT + MFA]
        Rate[Rate Limiting]
        Input[Input Validation]
    end
    
    subgraph "Data Security"
        Encrypt[Encryption at Rest<br/>AES-256]
        TLS[TLS 1.3 in Transit]
        Audit[Audit Logging]
        Backup[Encrypted Backups]
    end
    
    subgraph "Infrastructure Security"
        K8S[Kubernetes Security<br/>Pod Security Standards]
        Secrets[Secret Management<br/>Vault/K8s Secrets]
        Network[Network Policies]
        Monitor[Security Monitoring]
    end
    
    WAF --> AuthZ
    DDoS --> AuthN
    AuthZ --> Encrypt
    AuthN --> TLS
    Rate --> K8S
    Input --> Secrets
```

### Authentication Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Auth
    participant DB
    
    Client->>API: Login Request
    API->>Auth: Validate Credentials
    Auth->>DB: Check User
    DB->>Auth: User Data
    Auth->>Auth: Generate JWT
    Auth->>API: JWT Token
    API->>Client: Access Token
    
    loop API Requests
        Client->>API: Request + JWT
        API->>Auth: Validate JWT
        Auth->>API: Token Valid
        API->>Client: Response
    end
```

## Scalability Design

### Horizontal Scaling

```mermaid
graph TB
    subgraph "Load Balancers"
        LB1[Application LB<br/>HAProxy/Nginx]
        LB2[Database LB<br/>PgBouncer]
    end
    
    subgraph "Application Tier"
        API1[API Gateway 1]
        API2[API Gateway 2]
        API3[API Gateway 3]
        APIx[API Gateway N]
    end
    
    subgraph "Data Tier"
        DB_PRIMARY[PostgreSQL Primary]
        DB_REPLICA1[PostgreSQL Replica 1]
        DB_REPLICA2[PostgreSQL Replica 2]
        REDIS_CLUSTER[Redis Cluster]
    end
    
    LB1 --> API1
    LB1 --> API2
    LB1 --> API3
    LB1 --> APIx
    
    API1 --> LB2
    API2 --> LB2
    API3 --> LB2
    APIx --> LB2
    
    LB2 --> DB_PRIMARY
    LB2 --> DB_REPLICA1
    LB2 --> DB_REPLICA2
    
    API1 --> REDIS_CLUSTER
    API2 --> REDIS_CLUSTER
    API3 --> REDIS_CLUSTER
    APIx --> REDIS_CLUSTER
```

### Auto-Scaling Policies

| Metric | Scale Up Threshold | Scale Down Threshold | Min/Max Instances |
|--------|-------------------|---------------------|-------------------|
| CPU Utilization | >70% for 5 min | <30% for 10 min | 2/20 |
| Memory Usage | >80% for 3 min | <40% for 10 min | 2/20 |
| Request Rate | >1000 RPS | <200 RPS | 2/50 |
| Queue Length | >100 tasks | <10 tasks | 1/10 |

### Performance Targets

| Component | Metric | Target | SLA |
|-----------|--------|--------|-----|
| API Gateway | Response Time | <200ms P95 | 99.9% |
| Task Processing | Throughput | >4000 TPS | 99.5% |
| Node Assignment | Latency | <5 seconds | 99% |
| Database | Query Time | <50ms P95 | 99.9% |
| WebSocket | Connection Time | <1 second | 99% |

## Deployment Architecture

### Kubernetes Architecture

```yaml
# Production deployment overview
apiVersion: v1
kind: Namespace
metadata:
  name: deai-production

---
# API Gateway Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: api-gateway
        image: deai/api-gateway:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Multi-Region Deployment

```mermaid
graph TB
    subgraph "US East (Primary)"
        US_LB[Load Balancer]
        US_K8S[Kubernetes Cluster]
        US_DB[PostgreSQL Primary]
        US_CACHE[Redis Cluster]
    end
    
    subgraph "EU West (Secondary)"
        EU_LB[Load Balancer]
        EU_K8S[Kubernetes Cluster]
        EU_DB[PostgreSQL Replica]
        EU_CACHE[Redis Cluster]
    end
    
    subgraph "Asia Pacific (Secondary)"
        AP_LB[Load Balancer]
        AP_K8S[Kubernetes Cluster]
        AP_DB[PostgreSQL Replica]
        AP_CACHE[Redis Cluster]
    end
    
    subgraph "Global Services"
        DNS[Global DNS<br/>Route 53]
        CDN[CloudFront CDN]
        WAF[AWS WAF]
    end
    
    DNS --> CDN
    CDN --> WAF
    WAF --> US_LB
    WAF --> EU_LB
    WAF --> AP_LB
    
    US_DB --> EU_DB
    US_DB --> AP_DB
```

### Disaster Recovery

| RTO (Recovery Time Objective) | RPO (Recovery Point Objective) | Strategy |
|--------------------------------|--------------------------------|----------|
| **Critical Services**: 15 minutes | **Critical Data**: 1 minute | Active-Active with auto-failover |
| **Standard Services**: 1 hour | **Standard Data**: 15 minutes | Active-Passive with monitoring |
| **Non-Critical**: 4 hours | **Analytics Data**: 1 hour | Backup restore with manual intervention |

## Performance Optimization

### Caching Strategy

```mermaid
graph TB
    subgraph "Caching Layers"
        CDN[CDN Cache<br/>Static Assets]
        NGINX[Nginx Cache<br/>API Responses]
        REDIS[Redis Cache<br/>Application Data]
        DB_CACHE[Database Cache<br/>Query Results]
    end
    
    subgraph "Cache Policies"
        STATIC[Static: 1 year TTL]
        API[API: 5 minutes TTL]
        SESSION[Session: 24 hours TTL]
        QUERY[Query: 1 hour TTL]
    end
    
    CDN --> STATIC
    NGINX --> API
    REDIS --> SESSION
    DB_CACHE --> QUERY
```

### Database Optimization

- **Read Replicas**: 3 replicas for read scaling
- **Connection Pooling**: PgBouncer with 100 connections per instance
- **Query Optimization**: Automatic query analysis and index recommendations
- **Partitioning**: Time-based partitioning for large tables
- **Archival**: Automated data archival for historical data

### Monitoring and Observability

```mermaid
graph TB
    subgraph "Metrics Collection"
        PROM[Prometheus<br/>Metrics Storage]
        GRAF[Grafana<br/>Visualization]
        ALERT[AlertManager<br/>Notifications]
    end
    
    subgraph "Logging"
        FLUENTD[Fluentd<br/>Log Collection]
        ELASTIC[Elasticsearch<br/>Log Storage]
        KIBANA[Kibana<br/>Log Analysis]
    end
    
    subgraph "Tracing"
        JAEGER[Jaeger<br/>Distributed Tracing]
        TEMPO[Tempo<br/>Trace Storage]
    end
    
    subgraph "Application Monitoring"
        APM[Application Performance Monitoring]
        UPTIME[Uptime Monitoring]
        SYNTHETIC[Synthetic Testing]
    end
    
    PROM --> GRAF
    PROM --> ALERT
    FLUENTD --> ELASTIC
    ELASTIC --> KIBANA
    JAEGER --> TEMPO
```

This architecture is designed to support:

- **500+ concurrent nodes**
- **4000+ TPS coordination**
- **99.9% uptime SLA**
- **Global scalability**
- **Enterprise security**

The modular design allows for independent scaling and updating of components while maintaining high availability and performance.