// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "./GovernanceToken.sol";

/**
 * @title SimpleVoting
 * @dev Secure lightweight voting contract with proper token integration
 * @notice Simplified voting sessions with security hardening
 * 
 * Security Features:
 * - Real token-based vote weighting
 * - Reentrancy protection
 * - Access control with roles
 * - Emergency pause mechanism
 * - Anti-spam measures with deposits
 * - Gas-efficient batch operations
 */
contract SimpleVoting is ReentrancyGuard, Pausable, AccessControl {
    using Counters for Counters.Counter;
    
    // ============ Roles ============
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant MODERATOR_ROLE = keccak256("MODERATOR_ROLE");
    
    // ============ State Variables ============
    
    /// @dev Session counter using OpenZeppelin's Counters
    Counters.Counter private _sessionIdCounter;
    
    /// @dev Governance token contract
    GovernanceToken public immutable governanceToken;
    
    /// @dev Minimum voting duration (10 minutes)
    uint256 public constant MIN_VOTING_DURATION = 600;
    
    /// @dev Maximum voting duration (7 days)
    uint256 public constant MAX_VOTING_DURATION = 604800;
    
    /// @dev Session creation deposit (anti-spam)
    uint256 public sessionDeposit = 100 * 10**18; // 100 tokens
    
    /// @dev Vote session struct
    struct VoteSession {
        uint256 id;
        address creator;
        string question;
        string ipfsHash;
        uint256 startTime;
        uint256 endTime;
        uint256 snapshotBlock;
        bool isActive;
        uint256 yesVotes;
        uint256 noVotes;
        uint256 totalParticipants;
        uint256 deposit;
        bool depositRefunded;
        uint256 minimumQuorum;      // Minimum votes required for validity
    }
    
    /// @dev Individual vote in a session
    struct SimpleVote {
        address voter;
        bool choice;           // true = yes, false = no
        uint256 timestamp;
        uint256 weight;        // Token-based vote weight
    }
    
    // ============ Storage ============
    
    /// @dev Mapping from session ID to vote session
    mapping(uint256 => VoteSession) public voteSessions;
    
    /// @dev Mapping from session ID to voter address to vote
    mapping(uint256 => mapping(address => SimpleVote)) public sessionVotes;
    
    /// @dev Mapping to track if an address has voted in a session
    mapping(uint256 => mapping(address => bool)) public hasVotedInSession;
    
    /// @dev Minimum participants required for a valid vote
    uint256 public defaultMinimumQuorum = 1000 * 10**18; // 1000 tokens worth of votes
    
    /// @dev Minimum token balance to create sessions
    uint256 public sessionCreationThreshold = 500 * 10**18; // 500 tokens
    
    // ============ Events ============
    
    event VoteSessionCreated(
        uint256 indexed sessionId,
        address indexed creator,
        string question,
        string ipfsHash,
        uint256 startTime,
        uint256 endTime,
        uint256 snapshotBlock,
        uint256 minimumQuorum
    );
    
    event SimpleVoteCast(
        uint256 indexed sessionId,
        address indexed voter,
        bool choice,
        uint256 weight,
        uint256 timestamp
    );
    
    event VoteSessionEnded(
        uint256 indexed sessionId,
        uint256 yesVotes,
        uint256 noVotes,
        uint256 totalParticipants,
        bool result,
        bool quorumMet
    );
    
    event SessionDepositRefunded(
        uint256 indexed sessionId,
        address indexed creator,
        uint256 amount
    );
    
    event ParameterUpdated(
        string parameter,
        uint256 oldValue,
        uint256 newValue
    );
    
    // ============ Modifiers ============
    
    modifier validSession(uint256 sessionId) {
        require(
            sessionId > 0 && sessionId <= _sessionIdCounter.current(),
            "SimpleVoting: invalid session ID"
        );
        _;
    }
    
    modifier canVoteInSession(uint256 sessionId) {
        VoteSession storage session = voteSessions[sessionId];
        require(session.isActive, "SimpleVoting: session not active");
        require(block.timestamp >= session.startTime, "SimpleVoting: voting not started");
        require(block.timestamp <= session.endTime, "SimpleVoting: voting ended");
        require(!hasVotedInSession[sessionId][msg.sender], "SimpleVoting: already voted");
        _;
    }
    
    modifier onlySessionCreator(uint256 sessionId) {
        require(
            voteSessions[sessionId].creator == msg.sender,
            "SimpleVoting: not session creator"
        );
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        address _governanceToken,
        address _admin
    ) {
        require(_governanceToken != address(0), "SimpleVoting: zero governance token");
        require(_admin != address(0), "SimpleVoting: zero admin");
        
        governanceToken = GovernanceToken(_governanceToken);
        
        // Set up roles
        _grantRole(DEFAULT_ADMIN_ROLE, _admin);
        _grantRole(ADMIN_ROLE, _admin);
        
        // Start session counter at 1
        _sessionIdCounter.increment();
    }
    
    // ============ Core Functions ============
    
    /**
     * @dev Create a new voting session with deposit requirement
     * @param question The question to vote on
     * @param duration Duration of voting period in seconds
     * @param ipfsHash Optional IPFS hash for additional details
     * @param customQuorum Custom minimum quorum (0 to use default)
     * @return sessionId The ID of the created session
     */
    function createVoteSession(
        string calldata question,
        uint256 duration,
        string calldata ipfsHash,
        uint256 customQuorum
    ) external nonReentrant whenNotPaused returns (uint256 sessionId) {
        require(bytes(question).length > 0, "SimpleVoting: empty question");
        require(bytes(question).length <= 500, "SimpleVoting: question too long");
        require(duration >= MIN_VOTING_DURATION, "SimpleVoting: duration too short");
        require(duration <= MAX_VOTING_DURATION, "SimpleVoting: duration too long");
        
        // Check token balance requirement
        uint256 creatorBalance = governanceToken.getVotes(msg.sender);
        require(
            creatorBalance >= sessionCreationThreshold,
            "SimpleVoting: insufficient tokens to create session"
        );
        
        // Require session deposit (anti-spam)
        require(
            governanceToken.transferFrom(msg.sender, address(this), sessionDeposit),
            "SimpleVoting: deposit transfer failed"
        );
        
        // Get current session ID and increment
        sessionId = _sessionIdCounter.current();
        _sessionIdCounter.increment();
        
        // Calculate timing and quorum
        uint256 startTime = block.timestamp;
        uint256 endTime = startTime + duration;
        uint256 snapshotBlock = block.number - 1; // Use previous block
        uint256 minimumQuorum = customQuorum > 0 ? customQuorum : defaultMinimumQuorum;
        
        // Create vote session
        voteSessions[sessionId] = VoteSession({
            id: sessionId,
            creator: msg.sender,
            question: question,
            ipfsHash: ipfsHash,
            startTime: startTime,
            endTime: endTime,
            snapshotBlock: snapshotBlock,
            isActive: true,
            yesVotes: 0,
            noVotes: 0,
            totalParticipants: 0,
            deposit: sessionDeposit,
            depositRefunded: false,
            minimumQuorum: minimumQuorum
        });
        
        emit VoteSessionCreated(
            sessionId,
            msg.sender,
            question,
            ipfsHash,
            startTime,
            endTime,
            snapshotBlock,
            minimumQuorum
        );
    }
    
    /**
     * @dev Cast a vote in a session with token-based weighting
     * @param sessionId ID of the session to vote in
     * @param choice Vote choice (true=yes, false=no)
     */
    function voteInSession(
        uint256 sessionId,
        bool choice
    ) external nonReentrant validSession(sessionId) canVoteInSession(sessionId) whenNotPaused {
        VoteSession storage session = voteSessions[sessionId];
        
        // Get vote weight from snapshot block
        uint256 voteWeight = governanceToken.getPastVotes(msg.sender, session.snapshotBlock);
        require(voteWeight > 0, "SimpleVoting: no voting power at snapshot");
        
        // Record the vote
        sessionVotes[sessionId][msg.sender] = SimpleVote({
            voter: msg.sender,
            choice: choice,
            timestamp: block.timestamp,
            weight: voteWeight
        });
        
        // Mark as voted and update tallies
        hasVotedInSession[sessionId][msg.sender] = true;
        session.totalParticipants++;
        
        if (choice) {
            session.yesVotes += voteWeight;
        } else {
            session.noVotes += voteWeight;
        }
        
        emit SimpleVoteCast(
            sessionId,
            msg.sender,
            choice,
            voteWeight,
            block.timestamp
        );
    }
    
    /**
     * @dev End a voting session and determine results
     * @param sessionId ID of the session to end
     */
    function endVoteSession(uint256 sessionId) 
        external 
        nonReentrant
        validSession(sessionId) 
        whenNotPaused
    {
        VoteSession storage session = voteSessions[sessionId];
        require(session.isActive, "SimpleVoting: session already ended");
        require(block.timestamp > session.endTime, "SimpleVoting: voting still active");
        
        session.isActive = false;
        
        // Check if quorum was met and determine result
        uint256 totalVotes = session.yesVotes + session.noVotes;
        bool quorumMet = totalVotes >= session.minimumQuorum;
        bool result = quorumMet && (session.yesVotes > session.noVotes);
        
        // Refund deposit if quorum was met (incentive for valid sessions)
        if (quorumMet && !session.depositRefunded) {
            session.depositRefunded = true;
            require(
                governanceToken.transfer(session.creator, session.deposit),
                "SimpleVoting: deposit refund failed"
            );
            emit SessionDepositRefunded(sessionId, session.creator, session.deposit);
        }
        // If quorum not met, deposit stays in contract (anti-spam measure)
        
        emit VoteSessionEnded(
            sessionId,
            session.yesVotes,
            session.noVotes,
            session.totalParticipants,
            result,
            quorumMet
        );
    }
    
    /**
     * @dev Cancel a voting session (only by creator or admin)
     * @param sessionId ID of the session to cancel
     */
    function cancelVoteSession(uint256 sessionId) 
        external 
        nonReentrant
        validSession(sessionId)
        whenNotPaused
    {
        VoteSession storage session = voteSessions[sessionId];
        require(
            msg.sender == session.creator || hasRole(ADMIN_ROLE, msg.sender),
            "SimpleVoting: not authorized to cancel"
        );
        require(session.isActive, "SimpleVoting: session already ended");
        
        session.isActive = false;
        
        // Refund deposit for canceled sessions
        if (!session.depositRefunded) {
            session.depositRefunded = true;
            require(
                governanceToken.transfer(session.creator, session.deposit),
                "SimpleVoting: deposit refund failed"
            );
            emit SessionDepositRefunded(sessionId, session.creator, session.deposit);
        }
        
        emit VoteSessionEnded(
            sessionId,
            session.yesVotes,
            session.noVotes,
            session.totalParticipants,
            false, // Canceled = false result
            false  // No quorum check for canceled
        );
    }
    
    /**
     * @dev Emergency stop a voting session (moderator/admin only)
     * @param sessionId ID of the session to stop
     */
    function emergencyStopSession(uint256 sessionId) 
        external 
        nonReentrant
        validSession(sessionId)
        onlyRole(MODERATOR_ROLE)
    {
        VoteSession storage session = voteSessions[sessionId];
        require(session.isActive, "SimpleVoting: session already ended");
        
        session.isActive = false;
        
        // Refund deposit for emergency stopped sessions
        if (!session.depositRefunded) {
            session.depositRefunded = true;
            require(
                governanceToken.transfer(session.creator, session.deposit),
                "SimpleVoting: deposit refund failed"
            );
            emit SessionDepositRefunded(sessionId, session.creator, session.deposit);
        }
        
        emit VoteSessionEnded(
            sessionId,
            session.yesVotes,
            session.noVotes,
            session.totalParticipants,
            false, // Emergency stop = false result
            false  // No quorum check
        );
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Get total number of vote sessions created
     */
    function getSessionCount() external view returns (uint256) {
        return _sessionIdCounter.current() - 1;
    }
    
    /**
     * @dev Get vote session basic details
     */
    function getVoteSession(uint256 sessionId)
        external
        view
        validSession(sessionId)
        returns (
            uint256 id,
            address creator,
            string memory question,
            string memory ipfsHash,
            uint256 startTime,
            uint256 endTime,
            uint256 snapshotBlock,
            bool isActive
        )
    {
        VoteSession storage session = voteSessions[sessionId];
        return (
            session.id,
            session.creator,
            session.question,
            session.ipfsHash,
            session.startTime,
            session.endTime,
            session.snapshotBlock,
            session.isActive
        );
    }
    
    /**
     * @dev Get vote session results and stats
     */
    function getVoteSessionResults(uint256 sessionId)
        external
        view
        validSession(sessionId)
        returns (
            uint256 yesVotes,
            uint256 noVotes,
            uint256 totalParticipants,
            uint256 minimumQuorum
        )
    {
        VoteSession storage session = voteSessions[sessionId];
        return (
            session.yesVotes,
            session.noVotes,
            session.totalParticipants,
            session.minimumQuorum
        );
    }
    
    /**
     * @dev Get current session results with validity check
     */
    function getSessionResults(uint256 sessionId)
        external
        view
        validSession(sessionId)
        returns (
            uint256 yesVotes,
            uint256 noVotes,
            uint256 totalParticipants,
            bool isValid,
            bool yesWinning,
            bool quorumMet
        )
    {
        VoteSession storage session = voteSessions[sessionId];
        uint256 totalVotes = session.yesVotes + session.noVotes;
        
        return (
            session.yesVotes,
            session.noVotes,
            session.totalParticipants,
            session.isActive && block.timestamp <= session.endTime,
            session.yesVotes > session.noVotes,
            totalVotes >= session.minimumQuorum
        );
    }
    
    /**
     * @dev Check if address can create sessions
     */
    function canCreateSession(address account) external view returns (bool) {
        return governanceToken.getVotes(account) >= sessionCreationThreshold;
    }
    
    /**
     * @dev Batch get session results (gas efficient)
     */
    function batchGetSessions(uint256[] calldata sessionIds)
        external
        view
        returns (
            uint256[] memory yesVotes,
            uint256[] memory noVotes,
            uint256[] memory participants,
            bool[] memory isActive,
            bool[] memory quorumMet
        )
    {
        uint256 length = sessionIds.length;
        yesVotes = new uint256[](length);
        noVotes = new uint256[](length);
        participants = new uint256[](length);
        isActive = new bool[](length);
        quorumMet = new bool[](length);
        
        for (uint256 i = 0; i < length; i++) {
            uint256 sessionId = sessionIds[i];
            if (sessionId > 0 && sessionId <= _sessionIdCounter.current()) {
                VoteSession storage session = voteSessions[sessionId];
                yesVotes[i] = session.yesVotes;
                noVotes[i] = session.noVotes;
                participants[i] = session.totalParticipants;
                isActive[i] = session.isActive && 
                             block.timestamp >= session.startTime && 
                             block.timestamp <= session.endTime;
                quorumMet[i] = (session.yesVotes + session.noVotes) >= session.minimumQuorum;
            }
        }
    }
    
    // ============ Admin Functions ============
    
    /**
     * @dev Update session creation threshold
     */
    function updateSessionCreationThreshold(uint256 newThreshold) external onlyRole(ADMIN_ROLE) {
        uint256 oldThreshold = sessionCreationThreshold;
        sessionCreationThreshold = newThreshold;
        emit ParameterUpdated("sessionCreationThreshold", oldThreshold, newThreshold);
    }
    
    /**
     * @dev Update default minimum quorum
     */
    function updateDefaultMinimumQuorum(uint256 newQuorum) external onlyRole(ADMIN_ROLE) {
        require(newQuorum > 0, "SimpleVoting: quorum must be positive");
        uint256 oldQuorum = defaultMinimumQuorum;
        defaultMinimumQuorum = newQuorum;
        emit ParameterUpdated("defaultMinimumQuorum", oldQuorum, newQuorum);
    }
    
    /**
     * @dev Update session deposit
     */
    function updateSessionDeposit(uint256 newDeposit) external onlyRole(ADMIN_ROLE) {
        uint256 oldDeposit = sessionDeposit;
        sessionDeposit = newDeposit;
        emit ParameterUpdated("sessionDeposit", oldDeposit, newDeposit);
    }
    
    /**
     * @dev Emergency pause
     */
    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }
    
    /**
     * @dev Unpause
     */
    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }
    
    /**
     * @dev Withdraw unclaimed deposits (admin only)
     */
    function withdrawUnclaimedDeposits(address to, uint256 amount) external onlyRole(ADMIN_ROLE) {
        require(to != address(0), "SimpleVoting: zero address");
        require(
            governanceToken.transfer(to, amount),
            "SimpleVoting: transfer failed"
        );
    }
}