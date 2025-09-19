// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/GovernanceToken.sol";
import "../src/TimelockController.sol";
import "../src/GovernanceHub.sol";
import "../src/SimpleVoting.sol";

/**
 * @title SecureDeploy
 * @dev Secure deployment script for production Somnia Governance Engine
 * @notice Deploys all contracts with proper security configurations
 */
contract SecureDeploy is Script {
    // ============ Configuration ============
    
    // Token parameters
    string constant TOKEN_NAME = "Somnia Governance Token";
    string constant TOKEN_SYMBOL = "SGT";
    uint256 constant INITIAL_SUPPLY = 10_000_000 * 10**18; // 10M tokens
    
    // Timelock parameters  
    uint256 constant MIN_DELAY = 86400; // 24 hours for production
    
    // Governance parameters
    uint256 constant VOTING_THRESHOLD = 100_000 * 10**18;  // 100k tokens to propose
    uint256 constant PROPOSAL_THRESHOLD = 50_000 * 10**18; // 50k tokens minimum
    uint256 constant PROPOSAL_DEPOSIT = 10_000 * 10**18;   // 10k tokens deposit
    uint256 constant QUORUM_NUMERATOR = 4;                 // 4% quorum
    uint256 constant QUORUM_DENOMINATOR = 100;
    
    // SimpleVoting parameters
    uint256 constant SESSION_CREATION_THRESHOLD = 1_000 * 10**18; // 1k tokens
    uint256 constant SESSION_DEPOSIT = 100 * 10**18;              // 100 tokens
    uint256 constant DEFAULT_QUORUM = 5_000 * 10**18;             // 5k tokens
    
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);
        
        console.log("=== Secure Somnia Governance Deployment ===");
        console.log("Deployer:", deployer);
        console.log("Chain ID:", block.chainid);
        console.log("Block Number:", block.number);
        console.log("");
        
        // ============ Deploy Governance Token ============
        console.log("1. Deploying Governance Token...");
        GovernanceToken governanceToken = new GovernanceToken(
            TOKEN_NAME,
            TOKEN_SYMBOL,
            deployer,        // Initial owner
            INITIAL_SUPPLY   // Initial supply to deployer
        );
        console.log("   GovernanceToken:", address(governanceToken));
        console.log("   Initial Supply:", INITIAL_SUPPLY / 10**18, "SGT");
        console.log("");
        
        // ============ Deploy Timelock Controller ============
        console.log("2. Deploying Timelock Controller...");
        
        address[] memory proposers = new address[](1);
        proposers[0] = deployer; // Initially deployer can propose, will transfer to governance
        
        address[] memory executors = new address[](0); // Empty = anyone can execute after delay

        SomniaTimelockController timelock = new SomniaTimelockController(
            MIN_DELAY,
            proposers,
            executors,
            deployer // admin
        );
        console.log("   TimelockController:", address(timelock));
        console.log("   Minimum Delay:", MIN_DELAY / 3600, "hours");
        console.log("");
        
        // ============ Deploy Secure Governance Hub ============
        console.log("3. Deploying Secure Governance Hub...");
        GovernanceHub governance = new GovernanceHub(
            address(governanceToken),
            payable(address(timelock)),
            deployer // Initial admin
        );
        console.log("   GovernanceHub:", address(governance));
        console.log("");
        
        // ============ Deploy Secure Simple Voting ============
        console.log("4. Deploying Secure Simple Voting...");
        SimpleVoting simpleVoting = new SimpleVoting(
            address(governanceToken),
            deployer // Initial admin
        );
        console.log("   SimpleVoting:", address(simpleVoting));
        console.log("");
        
        // ============ Configure Governance Parameters ============
        console.log("5. Configuring Governance Parameters...");
        
        // Configure GovernanceHub
        governance.updateVotingThreshold(VOTING_THRESHOLD);
        governance.updateQuorum(QUORUM_NUMERATOR, QUORUM_DENOMINATOR);
        governance.updateProposalDeposit(PROPOSAL_DEPOSIT);
        console.log("   Voting Threshold:", VOTING_THRESHOLD / 10**18, "SGT");
        console.log("   Quorum:", QUORUM_NUMERATOR, "%");
        console.log("   Proposal Deposit:", PROPOSAL_DEPOSIT / 10**18, "SGT");
        
        // Configure SimpleVoting
        simpleVoting.updateSessionCreationThreshold(SESSION_CREATION_THRESHOLD);
        simpleVoting.updateSessionDeposit(SESSION_DEPOSIT);
        simpleVoting.updateDefaultMinimumQuorum(DEFAULT_QUORUM);
        console.log("   Session Creation Threshold:", SESSION_CREATION_THRESHOLD / 10**18, "SGT");
        console.log("   Session Deposit:", SESSION_DEPOSIT / 10**18, "SGT");
        console.log("   Default Session Quorum:", DEFAULT_QUORUM / 10**18, "SGT");
        console.log("");
        
        // ============ Set Up Roles ============
        console.log("6. Setting Up Access Control...");
        
        // Grant timelock PROPOSER role to governance contract
        bytes32 PROPOSER_ROLE = timelock.PROPOSER_ROLE();
        timelock.grantRole(PROPOSER_ROLE, address(governance));
        
        // Grant governance EXECUTOR role (can execute proposals)
        governance.grantRole(governance.EXECUTOR_ROLE(), deployer);
        
        // Grant moderator role for SimpleVoting
        simpleVoting.grantRole(simpleVoting.MODERATOR_ROLE(), deployer);
        
        console.log("   Timelock PROPOSER role granted to governance");
        console.log("   Governance EXECUTOR role granted to deployer");
        console.log("   SimpleVoting MODERATOR role granted to deployer");
        console.log("");
        
        // ============ Delegate Initial Voting Power ============
        console.log("7. Setting Up Initial Voting Delegation...");
        
        // Self-delegate to enable voting (deployer has initial tokens)
        governanceToken.delegate(deployer);
        console.log("   Delegated", INITIAL_SUPPLY / 10**18, "SGT voting power to deployer");
        console.log("");
        
        vm.stopBroadcast();
        
        // ============ Deployment Summary ============
        console.log("=== Deployment Complete ===");
        console.log("");
        console.log("Contract Addresses:");
        console.log("- GovernanceToken:", address(governanceToken));
        console.log("- TimelockController:", address(timelock));
        console.log("- GovernanceHub:", address(governance));
        console.log("- SimpleVoting:", address(simpleVoting));
        console.log("");
        
        console.log("Configuration:");
        console.log("- Total Token Supply:", INITIAL_SUPPLY / 10**18, "SGT");
        console.log("- Timelock Delay:", MIN_DELAY / 3600, "hours");
        console.log("- Proposal Threshold:", VOTING_THRESHOLD / 10**18, "SGT");
        console.log("- Governance Quorum:", QUORUM_NUMERATOR, "%");
        console.log("");
        
        console.log("Next Steps:");
        console.log("1. Distribute tokens to community members");
        console.log("2. Have token holders delegate voting power");
        console.log("3. Test governance with a simple proposal");
        console.log("4. Consider transferring admin roles to multisig");
        console.log("");
        
        console.log("Security Notes:");
        console.log("- All contracts use OpenZeppelin security modules");
        console.log("- Proposal execution requires timelock delay");
        console.log("- Token-based voting prevents fake votes");
        console.log("- Deposits prevent spam attacks");
        console.log("- Access control protects admin functions");
        console.log("=======================================");
    }
}

/**
 * @title TestnetDeploy
 * @dev Testnet deployment with relaxed parameters for testing
 */
contract TestnetDeploy is Script {
    // Relaxed parameters for testing
    string constant TOKEN_NAME = "Test Somnia Governance Token";
    string constant TOKEN_SYMBOL = "tSGT";
    uint256 constant INITIAL_SUPPLY = 1_000_000 * 10**18; // 1M tokens for testing
    uint256 constant MIN_DELAY = 300;                     // 5 minutes for testing
    
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);
        
        console.log("=== Testnet Deployment ===");
        
        // Deploy with test parameters
        GovernanceToken governanceToken = new GovernanceToken(
            TOKEN_NAME,
            TOKEN_SYMBOL,
            deployer,
            INITIAL_SUPPLY
        );
        
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](0);

        SomniaTimelockController timelock = new SomniaTimelockController(
            MIN_DELAY,
            proposers,
            executors,
            deployer // admin
        );

        GovernanceHub governance = new GovernanceHub(
            address(governanceToken),
            payable(address(timelock)),
            deployer
        );
        
        SimpleVoting simpleVoting = new SimpleVoting(
            address(governanceToken),
            deployer
        );
        
        // Configure with test-friendly parameters
        governance.updateVotingThreshold(1000 * 10**18);      // 1k tokens
        governance.updateQuorum(1, 100);                      // 1% quorum  
        governance.updateProposalDeposit(100 * 10**18);       // 100 tokens
        
        simpleVoting.updateSessionCreationThreshold(100 * 10**18); // 100 tokens
        simpleVoting.updateSessionDeposit(10 * 10**18);            // 10 tokens
        simpleVoting.updateDefaultMinimumQuorum(500 * 10**18);     // 500 tokens
        
        // Set up roles
        timelock.grantRole(timelock.PROPOSER_ROLE(), address(governance));
        governance.grantRole(governance.EXECUTOR_ROLE(), deployer);
        simpleVoting.grantRole(simpleVoting.MODERATOR_ROLE(), deployer);
        
        // Delegate voting power
        governanceToken.delegate(deployer);
        
        vm.stopBroadcast();
        
        console.log("=== Testnet Contracts ===");
        console.log("GovernanceToken:", address(governanceToken));
        console.log("TimelockController:", address(timelock));
        console.log("GovernanceHub:", address(governance));
        console.log("SimpleVoting:", address(simpleVoting));
        console.log("=========================");
    }
}