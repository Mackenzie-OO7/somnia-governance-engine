# Smart Contracts Guide

This guide provides comprehensive documentation for deploying, configuring, and interacting with the Somnia Governance Engine smart contracts.

## Table of Contents

- [Contract Overview](#contract-overview)
- [Deployment Guide](#deployment-guide)
- [Contract Interaction](#contract-interaction)
- [Security Considerations](#security-considerations)
- [Upgradeability](#upgradeability)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

## Contract Overview

The Somnia Governance Engine consists of four main smart contracts:

### 1. GovernanceToken (`GovernanceToken.sol`)

**Purpose**: ERC20 token with voting delegation capabilities.

**Key Features**:
- Standard ERC20 functionality
- Vote delegation mechanism (ERC20Votes)
- Mintable with access control
- Pausable for emergency situations
- Maximum supply cap (100M tokens)

**Main Functions**:
```solidity
// Delegation
function delegate(address delegatee) external
function delegateBySig(address delegatee, uint256 nonce, uint256 expiry, uint8 v, bytes32 r, bytes32 s) external

// Voting power queries
function getVotes(address account) external view returns (uint256)
function getPastVotes(address account, uint256 blockNumber) external view returns (uint256)

// Admin functions (MINTER_ROLE)
function mint(address to, uint256 amount) external
function pause() external
function unpause() external
```

### 2. GovernanceHub (`GovernanceHub.sol`)

**Purpose**: Main governance contract for proposal creation, voting, and execution.

**Key Features**:
- Proposal lifecycle management
- Token-weighted voting
- Quorum requirements
- Timelock integration
- Anti-spam deposits
- Multiple proposal types (Standard, Emergency, Constitutional)

**Main Functions**:
```solidity
// Proposal management
function createProposal(string calldata ipfsHash, uint256 duration, ProposalType proposalType) external returns (uint256)
function vote(uint256 proposalId, VoteChoice choice, string calldata ipfsHash) external
function executeProposal(uint256 proposalId) external

// View functions
function getProposal(uint256 proposalId) external view returns (ProposalInfo memory)
function getProposalCount() external view returns (uint256)
function canCreateProposal(address account) external view returns (bool)
function hasVoted(uint256 proposalId, address account) external view returns (bool)

// Admin functions
function updateVotingThreshold(uint256 newThreshold) external
function updateQuorum(uint256 newNumerator, uint256 newDenominator) external
function updateProposalDeposit(uint256 newDeposit) external
```

### 3. SimpleVoting (`SimpleVoting.sol`)

**Purpose**: Lightweight voting sessions for quick community decisions.

**Key Features**:
- Simple yes/no voting
- Token-weighted votes
- Session-based voting
- Configurable quorum
- Session creator controls

**Main Functions**:
```solidity
// Session management
function createVoteSession(string calldata question, uint256 duration, string calldata ipfsHash, uint256 customQuorum) external returns (uint256)
function voteInSession(uint256 sessionId, bool choice) external
function endVoteSession(uint256 sessionId) external

// View functions
function getVoteSession(uint256 sessionId) external view returns (SessionInfo memory)
function getVoteSessionResults(uint256 sessionId) external view returns (uint256, uint256, uint256, uint256)
function getSessionCount() external view returns (uint256)

// Admin functions
function updateSessionCreationThreshold(uint256 newThreshold) external
function updateDefaultMinimumQuorum(uint256 newQuorum) external
function updateSessionDeposit(uint256 newDeposit) external
```

### 4. SomniaTimelockController (`TimelockController.sol`)

**Purpose**: Timelock for secure proposal execution with configurable delays.

**Key Features**:
- Configurable execution delays
- Role-based access control
- Batch operation support
- Cancel functionality
- Multiple delay presets

**Main Functions**:
```solidity
// Inherited from OpenZeppelin TimelockController
function schedule(address target, uint256 value, bytes calldata data, bytes32 predecessor, bytes32 salt, uint256 delay) external
function execute(address target, uint256 value, bytes calldata data, bytes32 predecessor, bytes32 salt) external
function cancel(bytes32 id) external

// Custom functions
function getRecommendedDelay(uint8 proposalType) external pure returns (uint256)
```

## Deployment Guide

### Prerequisites

**⚠️ Important: These prerequisites are ONLY needed if you want to deploy/modify our smart contracts.**

**If you're using your own contracts or pure Rust governance, you can skip this entire section.**

1. **Foundry installed** (only for our smart contracts):
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

2. **Node.js v16+** (required by Foundry ecosystem):
   ```bash
   # Check if you have Node.js
   node --version

   # Install if needed
   # Visit https://nodejs.org or use your package manager
   ```

3. **Environment setup**:
   ```bash
   cd contracts
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Install dependencies**:
   ```bash
   forge install
   ```

### Alternative: Use Your Own Contracts

If you have existing governance contracts, you can skip deployment and just configure our Rust engine:

```rust
// Point our engine to your contracts instead
let config = Config {
    governance_hub_address: "0xYourGovernanceContract".to_string(),
    governance_token_address: "0xYourTokenContract".to_string(),
    // ... other addresses point to your contracts
    rpc_url: "https://your-rpc-endpoint".to_string(),
    // No need for our contract deployment
};

let contract_manager = ContractManager::with_custom_contracts(config).await?;
```

### Environment Configuration

Create `contracts/.env`:

```bash
# Deployment Configuration
PRIVATE_KEY=0xYourPrivateKeyHere
RPC_URL=https://somnia-testnet-rpc-url
CHAIN_ID=1234

# Verification (optional)
ETHERSCAN_API_KEY=your_etherscan_api_key

# Gas Configuration
GAS_PRICE=20000000000  # 20 gwei
GAS_LIMIT=8000000

# Token Configuration
TOKEN_NAME="Your Governance Token"
TOKEN_SYMBOL="YGT"
INITIAL_SUPPLY=1000000  # In tokens (will be converted to wei)
```

### Deployment Scripts

The project includes three deployment options:

#### 1. Basic Deployment

For development and testing:

```bash
forge script script/Deploy.s.sol:Deploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify
```

**Output**:
```
GovernanceToken deployed at: 0x1234...
TimelockController deployed at: 0x5678...
GovernanceHub deployed at: 0x9abc...
SimpleVoting deployed at: 0xdef0...
```

#### 2. Testnet Deployment

For testnet with optimized parameters:

```bash
forge script script/Deploy.s.sol:DeployTestnet \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify
```

**Features**:
- Lower token thresholds for testing
- Shorter timelock delays
- Reduced quorum requirements

#### 3. Production Deployment

For mainnet with security-focused configuration:

```bash
forge script script/SecureDeploy.s.sol:SecureDeploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify
```

**Features**:
- High security parameters
- Comprehensive role setup
- Production-ready thresholds
- Full access control configuration

### Post-Deployment Steps

1. **Save contract addresses**:
   ```bash
   # Contract addresses are saved in broadcast/ directory
   cat broadcast/Deploy.s.sol/$CHAIN_ID/run-latest.json | jq '.transactions[].contractAddress'
   ```

2. **Verify contracts** (if not done during deployment):
   ```bash
   forge verify-contract $CONTRACT_ADDRESS src/GovernanceToken.sol:GovernanceToken \
       --chain-id $CHAIN_ID \
       --constructor-args $(cast abi-encode "constructor(string,string,address,uint256)" "$TOKEN_NAME" "$TOKEN_SYMBOL" "$DEPLOYER_ADDRESS" "$INITIAL_SUPPLY")
   ```

3. **Update backend configuration**:
   ```bash
   # Update backend/.env with deployed addresses
   GOVERNANCE_HUB_ADDRESS=0x9abc...
   SIMPLE_VOTING_ADDRESS=0xdef0...
   GOVERNANCE_TOKEN_ADDRESS=0x1234...
   TIMELOCK_ADDRESS=0x5678...
   ```

## Contract Interaction

### Using Foundry Cast

#### Token Operations

```bash
# Check token balance
cast call $GOVERNANCE_TOKEN_ADDRESS "balanceOf(address)(uint256)" $USER_ADDRESS --rpc-url $RPC_URL

# Delegate voting power to self
cast send $GOVERNANCE_TOKEN_ADDRESS "delegate(address)" $USER_ADDRESS \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Check voting power
cast call $GOVERNANCE_TOKEN_ADDRESS "getVotes(address)(uint256)" $USER_ADDRESS --rpc-url $RPC_URL

# Transfer tokens
cast send $GOVERNANCE_TOKEN_ADDRESS "transfer(address,uint256)" $RECIPIENT_ADDRESS 1000000000000000000000 \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL
```

#### Governance Operations

```bash
# Create a proposal
cast send $GOVERNANCE_HUB_ADDRESS "createProposal(string,uint256,uint8)" \
    "QmProposalContentHash" 86400 0 \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Vote on proposal (0=Against, 1=For, 2=Abstain)
cast send $GOVERNANCE_HUB_ADDRESS "vote(uint256,uint8,string)" \
    0 1 "I support this proposal" \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Check proposal details
cast call $GOVERNANCE_HUB_ADDRESS "getProposal(uint256)" 0 --rpc-url $RPC_URL

# Execute proposal (after voting period ends)
cast send $GOVERNANCE_HUB_ADDRESS "executeProposal(uint256)" 0 \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL
```

#### Simple Voting Operations

```bash
# Create voting session
cast send $SIMPLE_VOTING_ADDRESS "createVoteSession(string,uint256,string,uint256)" \
    "Should we implement feature X?" 3600 "QmSessionDetails" 0 \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Vote in session (true=yes, false=no)
cast send $SIMPLE_VOTING_ADDRESS "voteInSession(uint256,bool)" 0 true \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Get session results
cast call $SIMPLE_VOTING_ADDRESS "getVoteSessionResults(uint256)" 0 --rpc-url $RPC_URL

# End session
cast send $SIMPLE_VOTING_ADDRESS "endVoteSession(uint256)" 0 \
    --private-key $PRIVATE_KEY --rpc-url $RPC_URL
```

### Using ethers.js

```javascript
const { ethers } = require('ethers');

// Setup
const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
const wallet = new ethers.Wallet(PRIVATE_KEY, provider);

// Contract instances
const governanceToken = new ethers.Contract(GOVERNANCE_TOKEN_ADDRESS, GOVERNANCE_TOKEN_ABI, wallet);
const governanceHub = new ethers.Contract(GOVERNANCE_HUB_ADDRESS, GOVERNANCE_HUB_ABI, wallet);
const simpleVoting = new ethers.Contract(SIMPLE_VOTING_ADDRESS, SIMPLE_VOTING_ABI, wallet);

// Create proposal
async function createProposal() {
    const tx = await governanceHub.createProposal(
        "QmProposalHash",
        86400, // 24 hours
        0      // Standard proposal
    );
    const receipt = await tx.wait();
    console.log("Proposal created:", receipt.transactionHash);
}

// Vote on proposal
async function voteOnProposal(proposalId, choice) {
    const tx = await governanceHub.vote(
        proposalId,
        choice, // 0=Against, 1=For, 2=Abstain
        "My reasoning for this vote"
    );
    await tx.wait();
    console.log("Vote cast successfully");
}

// Create voting session
async function createVotingSession() {
    const tx = await simpleVoting.createVoteSession(
        "Should we implement this feature?",
        3600, // 1 hour
        "QmSessionDetails",
        0     // Use default quorum
    );
    const receipt = await tx.wait();
    console.log("Session created:", receipt.transactionHash);
}
```

### Using Web3.py

```python
from web3 import Web3
import json

# Setup
w3 = Web3(Web3.HTTPProvider(RPC_URL))
account = w3.eth.account.from_key(PRIVATE_KEY)

# Load contract ABIs
with open('out/GovernanceHub.sol/GovernanceHub.json') as f:
    governance_hub_artifact = json.load(f)

governance_hub = w3.eth.contract(
    address=GOVERNANCE_HUB_ADDRESS,
    abi=governance_hub_artifact['abi']
)

# Create proposal
def create_proposal():
    tx = governance_hub.functions.createProposal(
        "QmProposalHash",
        86400,  # 24 hours
        0       # Standard proposal
    ).build_transaction({
        'from': account.address,
        'gas': 2000000,
        'gasPrice': w3.toWei('20', 'gwei'),
        'nonce': w3.eth.get_transaction_count(account.address)
    })

    signed_tx = w3.eth.account.sign_transaction(tx, PRIVATE_KEY)
    tx_hash = w3.eth.send_raw_transaction(signed_tx.rawTransaction)
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Proposal created: {tx_hash.hex()}")

# Vote on proposal
def vote_on_proposal(proposal_id, choice):
    tx = governance_hub.functions.vote(
        proposal_id,
        choice,  # 0=Against, 1=For, 2=Abstain
        "My voting reasoning"
    ).build_transaction({
        'from': account.address,
        'gas': 1000000,
        'gasPrice': w3.toWei('20', 'gwei'),
        'nonce': w3.eth.get_transaction_count(account.address)
    })

    signed_tx = w3.eth.account.sign_transaction(tx, PRIVATE_KEY)
    tx_hash = w3.eth.send_raw_transaction(signed_tx.rawTransaction)
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Vote cast: {tx_hash.hex()}")
```

## Security Considerations

### Access Control

The contracts use OpenZeppelin's AccessControl for role-based permissions:

```solidity
// Roles in GovernanceHub
bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
bytes32 public constant PROPOSER_ROLE = keccak256("PROPOSER_ROLE");
bytes32 public constant EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE");

// Roles in GovernanceToken
bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

// Roles in SimpleVoting
bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
bytes32 public constant MODERATOR_ROLE = keccak256("MODERATOR_ROLE");
```

### Security Features

1. **Reentrancy Protection**: All state-changing functions use `nonReentrant`
2. **Pause Mechanism**: Emergency pause for all contracts
3. **Timelock Protection**: Critical operations require timelock delay
4. **Input Validation**: Comprehensive input sanitization
5. **Overflow Protection**: Uses Solidity 0.8+ built-in overflow checks
6. **Access Control**: Role-based function access

### Best Practices for Deployment

1. **Use Multisig Wallets**: Deploy with multisig for admin roles
2. **Gradual Decentralization**: Start with admin control, gradually transfer to governance
3. **Parameter Testing**: Test all parameters on testnet first
4. **Emergency Procedures**: Have emergency pause/upgrade procedures ready
5. **Audit**: Get contracts audited before mainnet deployment

### Common Security Pitfalls

❌ **Don't**:
- Deploy with single EOA as admin
- Set very low quorum requirements
- Skip testing on testnet
- Use weak random numbers
- Ignore front-running attacks

✅ **Do**:
- Use multisig for admin operations
- Set reasonable quorum (5-20%)
- Test all edge cases
- Monitor for unusual activity
- Have emergency response plan

## Upgradeability

The current contracts are **not upgradeable** by design for security and trust. However, governance can deploy new versions and migrate:

### Migration Strategy

1. **Deploy new contracts**
2. **Create governance proposal** to migrate
3. **Transfer admin roles** to new contracts
4. **Pause old contracts**
5. **Update frontend/backend** to use new addresses

### Proxy Pattern (Optional)

If upgradeability is needed, consider using OpenZeppelin's proxy pattern:

```solidity
// Example upgradeable version
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

contract GovernanceHubUpgradeable is Initializable, UUPSUpgradeable {
    function initialize() public initializer {
        // Initialize instead of constructor
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(ADMIN_ROLE) {
        // Upgrade authorization logic
    }
}
```

## Testing

### Running Tests

```bash
# Run all tests
forge test

# Run specific test file
forge test --match-path test/GovernanceHub.t.sol

# Run with verbose output
forge test -vv

# Run coverage analysis
forge coverage
```

### Test Structure

```
test/
├── Integration.t.sol              # Basic integration tests
├── Governance.t.sol  # Comprehensive workflow tests
├── GovernanceHub.t.sol            # GovernanceHub unit tests
├── SimpleVoting.t.sol             # SimpleVoting unit tests
├── GovernanceToken.t.sol          # Token unit tests
└── TimelockController.t.sol       # Timelock unit tests
```

### Writing Custom Tests

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/GovernanceHub.sol";
import "../src/GovernanceToken.sol";

contract CustomGovernanceTest is Test {
    GovernanceHub public governance;
    GovernanceToken public token;

    address public user1 = address(0x1);
    address public user2 = address(0x2);

    function setUp() public {
        // Deploy contracts
        token = new GovernanceToken("Test Token", "TEST", address(this), 1000000 * 10**18);
        governance = new GovernanceHub(address(token), payable(address(this)), address(this));

        // Setup test scenario
        token.transfer(user1, 100000 * 10**18);
        token.transfer(user2, 50000 * 10**18);

        vm.prank(user1);
        token.delegate(user1);

        vm.prank(user2);
        token.delegate(user2);
    }

    function testCustomScenario() public {
        // Your custom test logic
        vm.startPrank(user1);

        uint256 proposalId = governance.createProposal(
            "QmTestProposal",
            86400,
            GovernanceHub.ProposalType.Standard
        );

        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        vm.stopPrank();

        // Assert expected outcomes
        assertTrue(governance.hasVoted(proposalId, user1));
    }
}
```

## Troubleshooting

### Common Issues

1. **"Insufficient voting power"**:
   - Check token balance with `getVotes()`
   - Ensure user has delegated tokens
   - Verify proposal threshold is met

2. **"Proposal not found"**:
   - Check proposal ID exists
   - Verify correct contract address
   - Ensure proposal was successfully created

3. **"Voting period ended"**:
   - Check current block timestamp
   - Verify proposal end time
   - Cannot vote after deadline

4. **"Already voted"**:
   - Each address can only vote once per proposal
   - Check `hasVoted()` before voting

5. **Gas estimation failed**:
   - Increase gas limit
   - Check contract state
   - Verify transaction parameters

### Debug Commands

```bash
# Check proposal details
cast call $GOVERNANCE_HUB_ADDRESS "getProposal(uint256)" $PROPOSAL_ID --rpc-url $RPC_URL

# Check voting power
cast call $GOVERNANCE_TOKEN_ADDRESS "getVotes(address)" $USER_ADDRESS --rpc-url $RPC_URL

# Check if user voted
cast call $GOVERNANCE_HUB_ADDRESS "hasVoted(uint256,address)" $PROPOSAL_ID $USER_ADDRESS --rpc-url $RPC_URL

# Check current block timestamp
cast call --block latest --rpc-url $RPC_URL "timestamp()(uint256)"
```

### Event Monitoring

Monitor contract events for debugging:

```bash
# Monitor all events from governance hub
cast logs --address $GOVERNANCE_HUB_ADDRESS --rpc-url $RPC_URL

# Monitor specific event
cast logs --address $GOVERNANCE_HUB_ADDRESS \
    --topics 0x7d84a6263ae0d98d3329bd7b46bb4e8d6f98cd35a7adb45c274c8b7fd5ebd5e0 \
    --rpc-url $RPC_URL
```

This comprehensive smart contracts guide provides everything needed to deploy, configure, and interact with the Somnia Governance Engine contracts securely and effectively.