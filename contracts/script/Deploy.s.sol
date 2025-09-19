// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/GovernanceHub.sol";
import "../src/SimpleVoting.sol";
import "../src/GovernanceToken.sol";
import "../src/TimelockController.sol";

/**
 * @title Deploy
 * @dev Deployment script for Somnia Governance Engine contracts
 * @notice This script deploys both GovernanceHub and SimpleVoting contracts
 */
contract Deploy is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);

        // Deploy GovernanceToken
        GovernanceToken governanceToken = new GovernanceToken(
            "Somnia Governance Token",
            "SGT",
            deployer,
            1000000 * 10**18 // 1 million tokens initial supply
        );
        console.log("GovernanceToken deployed at:", address(governanceToken));

        // Deploy Timelock
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](0); // Anyone can execute

        SomniaTimelockController timelock = new SomniaTimelockController(
            86400, // 24 hours minimum delay
            proposers,
            executors,
            deployer // admin
        );
        console.log("TimelockController deployed at:", address(timelock));

        // Deploy GovernanceHub contract
        GovernanceHub governanceHub = new GovernanceHub(
            address(governanceToken),
            payable(address(timelock)),
            deployer
        );
        console.log("GovernanceHub deployed at:", address(governanceHub));

        // Deploy SimpleVoting contract
        SimpleVoting simpleVoting = new SimpleVoting(
            address(governanceToken),
            deployer
        );
        console.log("SimpleVoting deployed at:", address(simpleVoting));

        vm.stopBroadcast();
        
        // Log deployment summary
        console.log("\n=== Deployment Summary ===");
        console.log("Network: ", block.chainid);
        console.log("Block Number: ", block.number);
        console.log("GovernanceToken: ", address(governanceToken));
        console.log("TimelockController: ", address(timelock));
        console.log("GovernanceHub: ", address(governanceHub));
        console.log("SimpleVoting: ", address(simpleVoting));
        console.log("Deployer: ", deployer);
        console.log("========================\n");
    }
}

/**
 * @title DeployTestnet
 * @dev Testnet deployment script with initial configuration
 */
contract DeployTestnet is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);

        // Deploy GovernanceToken with smaller initial supply for testing
        GovernanceToken governanceToken = new GovernanceToken(
            "Somnia Test Token",
            "SGTT",
            deployer,
            10000 * 10**18 // 10,000 tokens for testing
        );

        // Deploy Timelock with shorter delay for testnet
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](0); // Anyone can execute

        SomniaTimelockController timelock = new SomniaTimelockController(
            3600, // 1 hour minimum delay for testnet
            proposers,
            executors,
            deployer
        );

        // Deploy contracts
        GovernanceHub governanceHub = new GovernanceHub(
            address(governanceToken),
            payable(address(timelock)),
            deployer
        );

        SimpleVoting simpleVoting = new SimpleVoting(
            address(governanceToken),
            deployer
        );

        // Configure GovernanceHub for testnet
        governanceHub.updateVotingThreshold(100);  // Lower threshold for testing
        governanceHub.updateQuorum(10, 100);        // 10% quorum for testing (10/100)

        // Configure SimpleVoting for testnet
        simpleVoting.updateDefaultMinimumQuorum(1); // Allow single participant for testing

        vm.stopBroadcast();

        console.log("\n=== Testnet Deployment Summary ===");
        console.log("GovernanceToken: ", address(governanceToken));
        console.log("TimelockController: ", address(timelock));
        console.log("- Min Delay: 1 hour");
        console.log("");
        console.log("GovernanceHub: ", address(governanceHub));
        console.log("- Voting Threshold: 100");
        console.log("- Quorum: 10%");
        console.log("");
        console.log("SimpleVoting: ", address(simpleVoting));
        console.log("- Minimum Quorum: 1");
        console.log("==================================\n");
    }
}