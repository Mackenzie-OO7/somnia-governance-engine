// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/GovernanceToken.sol";
import "../src/TimelockController.sol";
import "../src/GovernanceHub.sol";
import "../src/SimpleVoting.sol";

contract IntegrationTest is Test {
    GovernanceToken public governanceToken;
    SomniaTimelockController public timelock;
    GovernanceHub public governanceHub;
    SimpleVoting public simpleVoting;

    address public deployer = address(0x1);
    address public user1 = address(0x2);
    address public user2 = address(0x3);

    function setUp() public {
        vm.startPrank(deployer);

        // Deploy GovernanceToken
        governanceToken = new GovernanceToken(
            "Test Token",
            "TEST",
            deployer,
            1000000 * 10**18
        );

        // Deploy Timelock
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](0);

        timelock = new SomniaTimelockController(
            3600, // 1 hour
            proposers,
            executors,
            deployer
        );

        // Deploy GovernanceHub
        governanceHub = new GovernanceHub(
            address(governanceToken),
            payable(address(timelock)),
            deployer
        );

        // Deploy SimpleVoting
        simpleVoting = new SimpleVoting(
            address(governanceToken),
            deployer
        );

        vm.stopPrank();
    }

    function testDeployment() public {
        // Check contracts are deployed
        assertTrue(address(governanceToken) != address(0));
        assertTrue(address(timelock) != address(0));
        assertTrue(address(governanceHub) != address(0));
        assertTrue(address(simpleVoting) != address(0));
    }

    function testTokenSupply() public {
        assertEq(governanceToken.totalSupply(), 1000000 * 10**18);
        assertEq(governanceToken.balanceOf(deployer), 1000000 * 10**18);
    }

    function testGovernanceConfiguration() public {
        vm.startPrank(deployer);

        // Test updating voting threshold
        governanceHub.updateVotingThreshold(5000 * 10**18);
        assertEq(governanceHub.votingThreshold(), 5000 * 10**18);

        // Test updating quorum
        governanceHub.updateQuorum(10, 100);
        assertEq(governanceHub.quorumNumerator(), 10);
        assertEq(governanceHub.quorumDenominator(), 100);

        vm.stopPrank();
    }

    function testSimpleVotingConfiguration() public {
        vm.startPrank(deployer);

        // Test updating session creation threshold
        simpleVoting.updateSessionCreationThreshold(1000 * 10**18);
        assertEq(simpleVoting.sessionCreationThreshold(), 1000 * 10**18);

        // Test updating session deposit
        simpleVoting.updateSessionDeposit(100 * 10**18);
        assertEq(simpleVoting.sessionDeposit(), 100 * 10**18);

        vm.stopPrank();
    }

    function testAccessControl() public {
        // Test that non-admin cannot update configuration
        vm.startPrank(user1);
        vm.expectRevert();
        governanceHub.updateVotingThreshold(1000);

        vm.expectRevert();
        simpleVoting.updateSessionCreationThreshold(1000);
        vm.stopPrank();
    }

    function testTokenTransfer() public {
        vm.startPrank(deployer);

        // Transfer tokens to user1
        governanceToken.transfer(user1, 10000 * 10**18);
        assertEq(governanceToken.balanceOf(user1), 10000 * 10**18);

        // User1 delegates to themselves
        vm.stopPrank();
        vm.startPrank(user1);
        governanceToken.delegate(user1);

        // Check voting power
        uint256 votingPower = governanceToken.getVotes(user1);
        assertEq(votingPower, 10000 * 10**18);

        vm.stopPrank();
    }
}