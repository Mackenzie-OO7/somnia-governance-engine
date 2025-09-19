// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/GovernanceToken.sol";
import "../src/TimelockController.sol";
import "../src/GovernanceHub.sol";
import "../src/SimpleVoting.sol";

contract GovernanceTest is Test {
    GovernanceToken public token;
    SomniaTimelockController public timelock;
    GovernanceHub public governance;
    SimpleVoting public simpleVoting;

    address public deployer = address(0x1);
    address public proposer = address(0x2);
    address public voter1 = address(0x3);
    address public voter2 = address(0x4);
    address public voter3 = address(0x5);

    uint256 constant INITIAL_SUPPLY = 1000000 * 10**18;
    uint256 constant PROPOSAL_THRESHOLD = 1000 * 10**18;  // Minimum tokens to create proposal
    uint256 constant PROPOSAL_DEPOSIT = 1000 * 10**18;

    function setUp() public {
        vm.startPrank(deployer);

        // Deploy token
        token = new GovernanceToken("Test Gov Token", "TGT", deployer, INITIAL_SUPPLY);

        // Deploy timelock
        address[] memory proposers = new address[](1);
        proposers[0] = deployer;
        address[] memory executors = new address[](0);
        timelock = new SomniaTimelockController(3600, proposers, executors, deployer);

        // Deploy governance
        governance = new GovernanceHub(address(token), payable(address(timelock)), deployer);

        // Deploy simple voting
        simpleVoting = new SimpleVoting(address(token), deployer);

        // Configure governance parameters
        governance.updateVotingThreshold(PROPOSAL_THRESHOLD);
        governance.updateQuorum(10, 100); // 10% quorum
        governance.updateProposalDeposit(PROPOSAL_DEPOSIT);

        // Distribute tokens for testing
        token.transfer(proposer, 50000 * 10**18);
        token.transfer(voter1, 100000 * 10**18);
        token.transfer(voter2, 80000 * 10**18);
        token.transfer(voter3, 60000 * 10**18);

        vm.stopPrank();

        // Users delegate to themselves
        vm.prank(proposer);
        token.delegate(proposer);
        vm.prank(voter1);
        token.delegate(voter1);
        vm.prank(voter2);
        token.delegate(voter2);
        vm.prank(voter3);
        token.delegate(voter3);

        // Deployer keeps and delegates remaining tokens
        vm.prank(deployer);
        token.delegate(deployer);
    }

    function testFullProposalLifecycle() public {
        vm.startPrank(proposer);

        // Ensure proposer has enough tokens
        assertGe(token.getVotes(proposer), PROPOSAL_THRESHOLD);

        // Create proposal
        uint256 proposalId = governance.createProposal(
            "QmTest123", // IPFS hash
            86400,       // 24 hours
            GovernanceHub.ProposalType.Standard
        );

        assertEq(proposalId, 0);
        assertEq(governance.getProposalCount(), 1);

        vm.stopPrank();

        // Check proposal deposit was taken
        assertEq(token.balanceOf(proposer), 50000 * 10**18 - PROPOSAL_DEPOSIT);

        // Vote on proposal
        vm.prank(voter1);
        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        vm.prank(voter2);
        governance.vote(proposalId, GovernanceHub.VoteChoice.Against, "");

        vm.prank(voter3);
        governance.vote(proposalId, GovernanceHub.VoteChoice.Abstain, "");

        // Check vote recording
        assertTrue(governance.hasVoted(proposalId, voter1));
        assertTrue(governance.hasVoted(proposalId, voter2));
        assertTrue(governance.hasVoted(proposalId, voter3));

        // Advance time past voting period
        vm.warp(block.timestamp + 86401);

        // Execute proposal (should pass as voter1 has more tokens than voter2)
        governance.executeProposal(proposalId);

        // Check that deposit was refunded (proposal passed)
        assertEq(token.balanceOf(proposer), 50000 * 10**18);
    }

    function testProposalFailsWithoutQuorum() public {
        vm.startPrank(proposer);

        uint256 proposalId = governance.createProposal("QmTest456", 86400, GovernanceHub.ProposalType.Standard);

        vm.stopPrank();

        // Only small vote (not meeting 10% quorum)
        vm.prank(voter3); // 60k tokens = 6% of 1M total
        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        vm.warp(block.timestamp + 86401);

        // Execution should fail due to lack of quorum
        vm.expectRevert("GovernanceHub: proposal failed");
        governance.executeProposal(proposalId);

        // Deposit should not be refunded
        assertEq(token.balanceOf(proposer), 50000 * 10**18 - PROPOSAL_DEPOSIT);
    }

    function testCannotVoteTwice() public {
        vm.prank(proposer);
        uint256 proposalId = governance.createProposal("QmTest789", 86400, GovernanceHub.ProposalType.Standard);

        vm.startPrank(voter1);
        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        // Try to vote again
        vm.expectRevert("GovernanceHub: already voted");
        governance.vote(proposalId, GovernanceHub.VoteChoice.Against, "");
        vm.stopPrank();
    }

    function testCannotCreateProposalWithoutTokens() public {
        address poorUser = address(0x999);

        vm.prank(poorUser);
        vm.expectRevert("GovernanceHub: insufficient voting power");
        governance.createProposal("QmTestFail", 86400, GovernanceHub.ProposalType.Standard);
    }

    function testSimpleVotingSessionLifecycle() public {
        // Setup: Give tokens to users for simple voting
        vm.startPrank(deployer);
        simpleVoting.updateSessionCreationThreshold(1000 * 10**18);
        simpleVoting.updateSessionDeposit(100 * 10**18);
        vm.stopPrank();

        vm.startPrank(voter1);

        // Create voting session
        uint256 sessionId = simpleVoting.createVoteSession(
            "Should we implement feature X?",
            3600, // 1 hour
            "QmSession123",
            0 // Use default quorum
        );

        assertEq(sessionId, 0);
        assertEq(simpleVoting.getSessionCount(), 1);

        vm.stopPrank();

        // Vote in session
        vm.prank(voter1);
        simpleVoting.voteInSession(sessionId, true); // Yes

        vm.prank(voter2);
        simpleVoting.voteInSession(sessionId, false); // No

        vm.prank(voter3);
        simpleVoting.voteInSession(sessionId, true); // Yes

        // Check results before ending
        (uint256 yesVotes, uint256 noVotes, uint256 totalParticipants,) =
            simpleVoting.getVoteSessionResults(sessionId);

        assertEq(totalParticipants, 3);
        assertEq(yesVotes, 100000 * 10**18 + 60000 * 10**18); // voter1 + voter3
        assertEq(noVotes, 80000 * 10**18); // voter2

        // Advance time and end session
        vm.warp(block.timestamp + 3601);

        vm.prank(voter1);
        simpleVoting.endVoteSession(sessionId);

        // Verify session ended
        (,,,,,,, bool isActive) = simpleVoting.getVoteSession(sessionId);
        assertFalse(isActive);
    }

    function testCannotVoteInExpiredSession() public {
        vm.prank(voter1);
        uint256 sessionId = simpleVoting.createVoteSession("Test", 3600, "", 0);

        // Advance time past session end
        vm.warp(block.timestamp + 3601);

        vm.prank(voter2);
        vm.expectRevert("SimpleVoting: voting period ended");
        simpleVoting.voteInSession(sessionId, true);
    }

    function testTimelockDelay() public {
        // Grant timelock proposer role to governance
        vm.prank(deployer);
        timelock.grantRole(timelock.PROPOSER_ROLE(), address(governance));

        vm.prank(proposer);
        uint256 proposalId = governance.createProposal("QmTimelock", 86400, GovernanceHub.ProposalType.Standard);

        // Vote to pass proposal
        vm.prank(voter1);
        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        vm.warp(block.timestamp + 86401);

        // Execute proposal (this should queue it in timelock)
        governance.executeProposal(proposalId);

        // Proposal should be queued but not executable yet
        // (Would need to check timelock state - simplified for this test)
    }

    function testQuorumCalculation() public {
        // With 1M total supply and 10% quorum, need 100k votes minimum
        uint256 currentQuorum = governance.getCurrentQuorum();
        assertEq(currentQuorum, 100000 * 10**18);

        // Update quorum to 5%
        vm.prank(deployer);
        governance.updateQuorum(5, 100);

        currentQuorum = governance.getCurrentQuorum();
        assertEq(currentQuorum, 50000 * 10**18);
    }

    function testAccessControlInAllContracts() public {
        address unauthorized = address(0x666);

        // Test GovernanceHub admin functions
        vm.startPrank(unauthorized);
        vm.expectRevert();
        governance.updateVotingThreshold(1000);

        vm.expectRevert();
        governance.updateQuorum(1, 10);

        vm.expectRevert();
        governance.pause();

        // Test SimpleVoting admin functions
        vm.expectRevert();
        simpleVoting.updateSessionCreationThreshold(1000);

        vm.expectRevert();
        simpleVoting.pause();

        vm.stopPrank();
    }

    function testPauseUnpauseFunctionality() public {
        vm.startPrank(deployer);

        // Pause governance
        governance.pause();

        vm.stopPrank();

        // Try to create proposal while paused
        vm.prank(proposer);
        vm.expectRevert();
        governance.createProposal("QmPaused", 86400, GovernanceHub.ProposalType.Standard);

        // Unpause
        vm.prank(deployer);
        governance.unpause();

        // Should work again
        vm.prank(proposer);
        uint256 proposalId = governance.createProposal("QmUnpaused", 86400, GovernanceHub.ProposalType.Standard);
        assertEq(proposalId, 0);
    }

    function testComplexVotingScenarios() public {
        vm.prank(proposer);
        uint256 proposalId = governance.createProposal("QmComplex", 86400, GovernanceHub.ProposalType.Standard);

        // Scenario: Majority votes FOR but with different voting powers
        vm.prank(deployer); // Has ~710k tokens (remaining after distribution)
        governance.vote(proposalId, GovernanceHub.VoteChoice.For, "");

        vm.prank(voter1); // 100k tokens
        governance.vote(proposalId, GovernanceHub.VoteChoice.Against, "");

        vm.prank(voter2); // 80k tokens
        governance.vote(proposalId, GovernanceHub.VoteChoice.Against, "");

        // Total FOR: ~710k, Total AGAINST: 180k
        // Quorum: 100k needed, we have ~890k total votes - passes quorum
        // Proposal should pass (FOR > AGAINST)

        vm.warp(block.timestamp + 86401);
        governance.executeProposal(proposalId);

        // Deposit should be refunded
        assertEq(token.balanceOf(proposer), 50000 * 10**18);
    }
}