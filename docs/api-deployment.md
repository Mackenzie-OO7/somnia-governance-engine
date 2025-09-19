# API Deployment Options

The Somnia Governance Engine REST API is **completely optional**. This guide covers all deployment approaches.

## Do I Need to Deploy an API?

**Short answer: Probably not!**

The API is useful when you need:
- HTTP endpoints for frontend applications
- Microservice architecture with separate governance service
- Integration with non-Rust applications
- Webhook endpoints for external systems

You **don't need** the API if you're:
- Using the Rust library directly in your application
- Building a CLI tool or desktop application
- Integrating governance into existing Rust web services

## Integration Approaches

### Option 1: Direct Library Integration (Recommended)

Use the governance engine directly in your Rust application:

```rust
// In your existing application
use somnia_governance_engine::blockchain::ContractManager;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let contract_manager = ContractManager::new(config).await?;

    // Use directly - no separate API server needed
    let proposal_id = contract_manager.create_proposal(
        "QmProposalHash",
        86400,
        ProposalType::Standard,
    ).await?;

    Ok(())
}
```

**Pros:**
- No additional deployment complexity
- Better performance (no HTTP overhead)
- Type safety with Rust
- Easier error handling

**Cons:**
- Only works with Rust applications
- Harder to integrate with frontend frameworks

### Option 2: Embedded API Routes

Add governance routes to your existing web server:

```rust
// With Axum
use axum::{Router, routing::get};
use somnia_governance_engine::api::routes::governance_routes;

async fn setup_server() -> Result<Router> {
    let governance_api = GovernanceAPI::new(contract_manager, db).await?;

    let app = Router::new()
        // Your existing routes
        .route("/health", get(health_check))
        .route("/users", get(list_users))

        // Add governance routes
        .nest("/governance", governance_routes(governance_api))

        // Your other routes
        .route("/orders", get(list_orders));

    Ok(app)
}
```

**Pros:**
- Single deployment
- Shared authentication/middleware
- Consistent monitoring/logging

**Cons:**
- Couples governance with your main application
- Increases main application complexity

### Option 3: Standalone API Server

Deploy the governance API as a separate service:

```bash
# Clone and build
git clone https://github.com/your-org/somnia-governance-engine.git
cd backend

# Configure environment
cp .env.example .env
# Edit .env with your settings

# Run the server
cargo run --bin governance-api
```

**Environment Configuration:**
```bash
# .env
DATABASE_URL=postgresql://user:pass@localhost/governance
RPC_URL=https://somnia-rpc-endpoint
PRIVATE_KEY=0x...
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Contract addresses (from deployment)
GOVERNANCE_HUB_ADDRESS=0x...
SIMPLE_VOTING_ADDRESS=0x...
GOVERNANCE_TOKEN_ADDRESS=0x...
TIMELOCK_ADDRESS=0x...
```

**Pros:**
- Language-agnostic integration
- Independent scaling
- Clear separation of concerns
- Easy to expose to external partners

**Cons:**
- Additional deployment complexity
- Network latency
- Need for API authentication

## Deployment Environments

### Development

```bash
# Quick local setup
cd backend
cargo run

# Available at http://localhost:3000
curl http://localhost:3000/api/proposals
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/governance-api /usr/local/bin/governance-api

EXPOSE 3000
CMD ["governance-api"]
```

```bash
# Build and run
docker build -t governance-api .
docker run -p 3000:3000 --env-file .env governance-api
```

### Cloud Deployment

#### AWS (ECS/Fargate)

```yaml
# docker-compose.yml for ECS
version: '3.8'
services:
  governance-api:
    image: your-registry/governance-api:latest
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - RPC_URL=${RPC_URL}
      - PRIVATE_KEY=${PRIVATE_KEY}
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: governance
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

#### Railway/Render

```toml
# railway.toml
[build]
builder = "NIXPACKS"

[deploy]
startCommand = "cargo run --release --bin governance-api"
restartPolicyType = "ON_FAILURE"

[env]
DATABASE_URL = "$DATABASE_URL"
RPC_URL = "$RPC_URL"
```

#### Vercel (Serverless Functions)

```rust
// api/governance.rs
use vercel_runtime::{run, Body, Error, Request, Response};
use somnia_governance_engine::api::handlers;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let governance_api = GovernanceAPI::from_env().await?;
    governance_api.handle_vercel_request(req).await
}
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: governance-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: governance-api
  template:
    metadata:
      labels:
        app: governance-api
    spec:
      containers:
      - name: governance-api
        image: your-registry/governance-api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: governance-secrets
              key: database-url
        - name: RPC_URL
          valueFrom:
            configMapKeyRef:
              name: governance-config
              key: rpc-url
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: governance-api-service
spec:
  selector:
    app: governance-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

## API Usage Examples

Once deployed, you can interact with the API:

### JavaScript/TypeScript

```typescript
// Frontend integration
const API_BASE = 'https://your-governance-api.com';

class GovernanceClient {
  async createProposal(proposal: CreateProposalRequest): Promise<Proposal> {
    const response = await fetch(`${API_BASE}/api/proposals`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(proposal),
    });
    return response.json();
  }

  async vote(proposalId: number, vote: VoteRequest): Promise<void> {
    await fetch(`${API_BASE}/api/proposals/${proposalId}/vote`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(vote),
    });
  }

  async getProposal(proposalId: number): Promise<Proposal> {
    const response = await fetch(`${API_BASE}/api/proposals/${proposalId}`);
    return response.json();
  }
}
```

### Python

```python
import requests
from typing import Dict, Any

class GovernanceClient:
    def __init__(self, base_url: str):
        self.base_url = base_url

    def create_proposal(self, proposal: Dict[str, Any]) -> Dict[str, Any]:
        response = requests.post(f"{self.base_url}/api/proposals", json=proposal)
        response.raise_for_status()
        return response.json()

    def vote(self, proposal_id: int, vote: Dict[str, Any]) -> None:
        response = requests.post(
            f"{self.base_url}/api/proposals/{proposal_id}/vote",
            json=vote
        )
        response.raise_for_status()

    def get_proposal(self, proposal_id: int) -> Dict[str, Any]:
        response = requests.get(f"{self.base_url}/api/proposals/{proposal_id}")
        response.raise_for_status()
        return response.json()

# Usage
client = GovernanceClient("https://your-governance-api.com")
proposal = client.create_proposal({
    "ipfs_hash": "QmProposal123",
    "duration": 86400,
    "proposal_type": "standard"
})
```

### curl

```bash
# Create proposal
curl -X POST https://your-governance-api.com/api/proposals \
  -H "Content-Type: application/json" \
  -d '{
    "ipfs_hash": "QmProposal123",
    "duration": 86400,
    "proposal_type": "standard"
  }'

# Vote on proposal
curl -X POST https://your-governance-api.com/api/proposals/1/vote \
  -H "Content-Type: application/json" \
  -d '{
    "choice": "for",
    "reasoning": "I support this proposal",
    "signature": "0x..."
  }'

# Get proposal
curl https://your-governance-api.com/api/proposals/1
```

## Monitoring and Observability

### Health Checks

```rust
// Built-in health endpoints
GET /health        // Basic health check
GET /ready         // Readiness check (DB + RPC connectivity)
GET /metrics       // Prometheus metrics
```

### Logging

```rust
// Configure structured logging
use tracing::{info, warn, error};
use tracing_subscriber;

// In your main function
tracing_subscriber::fmt::init();

// Throughout the application
info!("Proposal created: {}", proposal_id);
warn!("Low token balance for user: {}", user_address);
error!("Failed to connect to RPC: {}", error);
```

### Metrics

```rust
// Prometheus metrics
use prometheus::{Counter, Histogram, Registry};

static PROPOSALS_CREATED: Counter = Counter::new(
    "governance_proposals_created_total",
    "Total number of proposals created"
).unwrap();

static VOTE_LATENCY: Histogram = Histogram::new(
    "governance_vote_duration_seconds",
    "Time taken to process votes"
).unwrap();
```

## Security Considerations

### API Authentication

```rust
// JWT-based authentication
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Serialize, Deserialize)]
struct Claims {
    address: String,
    exp: usize,
}

async fn authenticate_request(headers: HeaderMap) -> Result<String> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or("Missing token")?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    )?;

    Ok(claims.claims.address)
}
```

### Rate Limiting

```rust
// Using tower-governor
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap(),
);

let app = Router::new()
    .route("/api/proposals", post(create_proposal))
    .layer(GovernorLayer { config: governor_conf });
```

### CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
    .allow_origin("https://your-frontend.com".parse::<HeaderValue>().unwrap());

let app = Router::new()
    .route("/api/proposals", get(list_proposals))
    .layer(cors);
```

## Performance Optimization

### Connection Pooling

```rust
// Database connection pooling
use sqlx::postgres::PgPoolOptions;

let db_pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

### Caching

```rust
// Redis caching for proposal data
use redis::aio::ConnectionManager;

async fn get_proposal_cached(
    proposal_id: u64,
    redis: &mut ConnectionManager,
    db: &Database,
) -> Result<Proposal> {
    let cache_key = format!("proposal:{}", proposal_id);

    // Try cache first
    if let Ok(cached) = redis.get::<_, String>(&cache_key).await {
        if let Ok(proposal) = serde_json::from_str::<Proposal>(&cached) {
            return Ok(proposal);
        }
    }

    // Fallback to database
    let proposal = db.get_proposal(proposal_id).await?;

    // Cache for 5 minutes
    let _: () = redis.setex(
        &cache_key,
        300,
        serde_json::to_string(&proposal)?,
    ).await?;

    Ok(proposal)
}
```

## Conclusion

The choice of API deployment depends on your specific use case:

- **Direct Integration**: Best for Rust applications
- **Embedded Routes**: Good for adding governance to existing services
- **Standalone API**: Best for microservices or multi-language environments
- **Serverless**: Good for cost optimization and automatic scaling

Choose the approach that best fits your architecture and team expertise!