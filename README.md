# Somnia Governance Engine

A comprehensive, production-ready governance toolkit for blockchain ecosystems, designed specifically for the Somnia network. This toolkit provides both smart contracts and a Rust backend library for implementing secure, token-based governance systems.

## Overview

The Somnia Governance Engine is a complete governance solution that enables projects to:

- **Create and manage governance proposals** with timelock execution
- **Implement token-based voting** with proper delegation and snapshot mechanics
- **Run simple voting sessions** for community decisions
- **Integrate governance functionality** into existing Rust applications
- **Deploy secure contracts** with built-in anti-spam and security measures

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Rust Backend  │    │ Smart Contracts │
│   (Optional)    │◄───┤   Library       │◄───┤   (Solidity)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                               ▼
                       ┌─────────────────┐
                       │   Blockchain    │
                       │   (Somnia)      │
                       └─────────────────┘
```

### Core Components

1. **Smart Contracts** (`/contracts`):
   - `GovernanceHub`: Main governance contract with proposal lifecycle management
   - `SimpleVoting`: Lightweight voting sessions for quick decisions
   - `GovernanceToken`: ERC20 token with voting delegation capabilities
   - `TimelockController`: Secure delayed execution of proposals

2. **Backend (Rust)** (`/backend`):
   - REST API for governance operations
   - Blockchain integration and event monitoring
   - Signature verification and authentication
   - Database persistence layer

## Installation & Setup

### Prerequisites

**Core Requirements:**
- **Rust** (latest stable version) - Always required

**Optional Requirements (depending on your use case):**

**Node.js v16+ and npm:**
- **Required if:** Deploying/modifying smart contracts, using ethers.js frontend integration, running contract build system
- **NOT needed if:** Only using the Rust library for governance logic

**Foundry:**
- **Required if:** Working with smart contracts (deployment, testing, modifications)
- **NOT needed if:** Using your own contracts or pure off-chain governance

**PostgreSQL:**
- **Required if:** You want persistent database storage with our database layer
- **NOT needed if:** Using in-memory storage, your own database, or file-based storage

### Quick Start Options

Choose your integration approach:

#### Option A: Rust Library Only (Minimal Setup)
```bash
# Just add to your Cargo.toml - no other dependencies needed
cargo add somnia-governance-engine

# Start using immediately
use somnia_governance_engine::core::VotingEngine;
```

#### Option B: With Our Smart Contracts
```bash
# 1. Clone repository
git clone https://github.com/your-org/somnia-governance-engine.git
cd somnia-governance-engine

# 2. Set up smart contracts (requires Node.js + Foundry)
cd contracts
forge install
forge build

# 3. Deploy contracts
forge script script/Deploy.s.sol:Deploy --rpc-url $RPC_URL --broadcast

# 4. Use Rust backend
cd ../backend
cargo build
```

#### Option C: With Your Own Contracts
```bash
# 1. Add to your project
cargo add somnia-governance-engine

# 2. Configure for your contracts
# No need to deploy our contracts - point to yours
```

#### Option D: Full Stack with Database
```bash
# Requires PostgreSQL setup
createdb governance_db

# Then follow Option B steps above
# Configure DATABASE_URL in .env
```

## Usage

### As a Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
somnia-governance-engine = { path = "path/to/backend" }
tokio = { version = "1.0", features = ["full"] }
```

#### Basic Example

```rust
use somnia_governance_engine::{
    blockchain::ContractManager,
    models::{Proposal, Vote},
    config::Config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the governance engine
    let config = Config::from_env()?;
    let contract_manager = ContractManager::new(config.clone()).await?;

    // Create a new proposal
    let proposal = contract_manager.create_proposal(
        "QmProposalHash123",  // IPFS hash with proposal details
        86400,               // 24 hours voting duration
        ProposalType::Standard,
    ).await?;

    println!("Created proposal with ID: {}", proposal.id);

    // Cast a vote
    contract_manager.vote(
        proposal.id,
        VoteChoice::For,
        Some("Supporting this proposal because...".to_string()),
    ).await?;

    Ok(())
}
```

#### Advanced Integration

```rust
use somnia_governance_engine::{
    api::governance::{GovernanceAPI, CreateProposalRequest},
    blockchain::EventMonitor,
    database::Database,
};

// Set up governance API
let db = Database::connect(&config.database_url).await?;
let governance_api = GovernanceAPI::new(contract_manager, db);

// Create proposal via API
let proposal_request = CreateProposalRequest {
    ipfs_hash: "QmDetailedProposal456".to_string(),
    duration: 172800, // 48 hours
    proposal_type: "emergency".to_string(),
};

let proposal = governance_api.create_proposal(proposal_request).await?;

// Monitor blockchain events
let mut event_monitor = EventMonitor::new(contract_manager.clone());
event_monitor.start_monitoring().await?;

// Listen for proposal events
while let Some(event) = event_monitor.next_event().await {
    match event {
        GovernanceEvent::ProposalCreated { id, creator, .. } => {
            println!("New proposal {} created by {}", id, creator);
        }
        GovernanceEvent::VoteCast { proposal_id, voter, choice, .. } => {
            println!("Vote cast on proposal {}: {} voted {:?}",
                     proposal_id, voter, choice);
        }
        _ => {}
    }
}
```

### REST API Usage

Start the backend server:

```bash
cd backend
cargo run
```

#### API Endpoints

**Create Proposal**:
```bash
curl -X POST http://localhost:3000/api/proposals \\
  -H "Content-Type: application/json" \\
  -d '{
    "ipfs_hash": "QmProposalContent123",
    "duration": 86400,
    "proposal_type": "standard"
  }'
```

**Vote on Proposal**:
```bash
curl -X POST http://localhost:3000/api/proposals/1/vote \\
  -H "Content-Type: application/json" \\
  -d '{
    "choice": "for",
    "reasoning": "I support this proposal",
    "signature": "0x..."
  }'
```

**Get Proposal Details**:
```bash
curl http://localhost:3000/api/proposals/1
```

**List All Proposals**:
```bash
curl http://localhost:3000/api/proposals
```

## 📋 Contract Deployment

### Environment Setup

Create a `.env` file in the `contracts` directory:

```bash
# contracts/.env
PRIVATE_KEY=0xYourPrivateKeyHere
RPC_URL=https://somnia-testnet-rpc-url
ETHERSCAN_API_KEY=your_etherscan_key  # Optional for verification
```

### Deployment Scripts

The project includes three deployment scripts:

1. **Basic Deployment** (`Deploy.s.sol`):
   ```bash
   forge script script/Deploy.s.sol:Deploy --rpc-url $RPC_URL --broadcast
   ```

2. **Testnet Deployment** (`Deploy.s.sol:DeployTestnet`):
   ```bash
   forge script script/Deploy.s.sol:DeployTestnet --rpc-url $RPC_URL --broadcast
   ```

3. **Production Deployment** (`SecureDeploy.s.sol`):
   ```bash
   forge script script/SecureDeploy.s.sol:SecureDeploy --rpc-url $RPC_URL --broadcast
   ```

### Contract Addresses

After deployment, you'll get contract addresses like:

```
GovernanceToken: 0x1234...
TimelockController: 0x5678...
GovernanceHub: 0x9abc...
SimpleVoting: 0xdef0...
```

Update your backend configuration with these addresses.

## Configuration

### Backend Configuration

Create `backend/.env`:

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost/governance_db

# Blockchain
RPC_URL=https://somnia-testnet-rpc
CHAIN_ID=1234
PRIVATE_KEY=0xYourPrivateKey

# Contract Addresses (from deployment)
GOVERNANCE_HUB_ADDRESS=0x9abc...
SIMPLE_VOTING_ADDRESS=0xdef0...
GOVERNANCE_TOKEN_ADDRESS=0x1234...
TIMELOCK_ADDRESS=0x5678...

# API
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
JWT_SECRET=your-secret-key
CORS_ORIGINS=http://localhost:3000,https://yourdomain.com
```

### Frontend Integration

If you're building a frontend, here's how to interact with the contracts:

```typescript
import { ethers } from 'ethers';

// Contract ABIs (copy from contracts/out/)
const GOVERNANCE_HUB_ABI = [...];

// Connect to contract
const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
const governanceHub = new ethers.Contract(
  GOVERNANCE_HUB_ADDRESS,
  GOVERNANCE_HUB_ABI,
  provider
);

// Create proposal
const tx = await governanceHub.createProposal(
  "QmProposalHash",
  86400, // duration
  0      // ProposalType.Standard
);

await tx.wait();
```

## 🧪 Testing

### Smart Contract Tests

```bash
cd contracts
forge test -vv
```

### Backend Tests

```bash
cd backend
cargo test
```

### Integration Tests

```bash
# Run comprehensive governance tests
cd contracts
forge test --match-path test/Governance.t.sol -vv
```

## 📚 Governance Workflows

### 1. Standard Proposal Flow

```mermaid
graph TD
    A[Create Proposal] --> B[Community Discussion]
    B --> C[Voting Period]
    C --> D{Quorum Met?}
    D -->|Yes| E{Majority Vote?}
    D -->|No| F[Proposal Failed]
    E -->|Yes| G[Timelock Queue]
    E -->|No| F
    G --> H[Execute After Delay]
    H --> I[Proposal Executed]
```

### 2. Simple Voting Session

```mermaid
graph TD
    A[Create Session] --> B[Open Voting]
    B --> C[Cast Votes]
    C --> D[Session Expires]
    D --> E[End Session]
    E --> F[Results Available]
```

## 🔒 Security Features

- **Timelock Protection**: All governance proposals go through a timelock delay
- **Token-Based Voting**: Only token holders can participate
- **Snapshot Voting**: Prevents manipulation during voting period
- **Deposit Requirements**: Anti-spam measures for proposal creation
- **Access Control**: Role-based permissions for admin functions
- **Reentrancy Protection**: Guards against reentrancy attacks
- **Pause Mechanism**: Emergency pause for critical situations

## Development

### Project Structure

```
somnia-governance-engine/
├── contracts/                 # Smart contracts (Solidity)
│   ├── src/                  # Contract source files
│   ├── script/               # Deployment scripts
│   ├── test/                 # Contract tests
│   └── foundry.toml          # Foundry configuration
├── backend/                  # Rust backend library
│   ├── src/                  # Rust source code
│   ├── migrations/           # Database migrations
│   └── Cargo.toml           # Rust dependencies
└── docs/                    # Additional documentation
```

### Adding New Features

1. **Smart Contract Changes**:
   - Modify contracts in `contracts/src/`
   - Add tests in `contracts/test/`
   - Update deployment scripts if needed

2. **Backend Changes**:
   - Add new modules in `backend/src/`
   - Update API endpoints in `backend/src/api/`
   - Add database migrations if needed

### Code Quality

The project maintains high code quality standards:

- **Rust**: Uses `clippy` and `rustfmt`
- **Solidity**: Follows OpenZeppelin patterns
- **Testing**: Comprehensive test coverage
- **Documentation**: Inline code documentation

## API Reference

### Governance API

#### `POST /api/proposals`
Create a new governance proposal.

**Request**:
```json
{
  "ipfs_hash": "QmProposalContent",
  "duration": 86400,
  "proposal_type": "standard"
}
```

**Response**:
```json
{
  "id": 1,
  "ipfs_hash": "QmProposalContent",
  "creator": "0x123...",
  "status": "active",
  "created_at": "2025-01-15T10:00:00Z"
}
```

#### `GET /api/proposals/{id}`
Get proposal details.

**Response**:
```json
{
  "id": 1,
  "ipfs_hash": "QmProposalContent",
  "creator": "0x123...",
  "status": "active",
  "for_votes": "150000000000000000000000",
  "against_votes": "50000000000000000000000",
  "quorum": "100000000000000000000000",
  "end_time": "2025-01-16T10:00:00Z"
}
```

#### `POST /api/proposals/{id}/vote`
Cast a vote on a proposal.

**Request**:
```json
{
  "choice": "for",
  "reasoning": "I support this proposal",
  "signature": "0x..."
}
```

### Simple Voting API

#### `POST /api/sessions`
Create a voting session.

**Request**:
```json
{
  "question": "Should we implement feature X?",
  "duration": 3600,
  "ipfs_hash": "QmSessionDetails"
}
```

#### `POST /api/sessions/{id}/vote`
Vote in a session.

**Request**:
```json
{
  "choice": true,
  "signature": "0x..."
}
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: Check the `/docs` folder for detailed guides
- **Issues**: Report bugs or request features on GitHub Issues
- **Community**: Join our Discord/Telegram for discussions

## Roadmap

- [ ] Multi-signature wallet integration
- [ ] Advanced voting mechanisms (quadratic voting, conviction voting)
- [ ] Cross-chain governance support
- [ ] Enhanced frontend dashboard
- [ ] Governance analytics and insights
- [ ] Mobile app support

---

**Built for the Somnia Ecosystem**

This governance engine is specifically designed to work seamlessly with Somnia's high-performance blockchain infrastructure, providing the tools needed for decentralized decision-making at scale.