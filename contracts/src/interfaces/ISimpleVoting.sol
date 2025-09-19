// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ISimpleVoting
 * @dev Interface for SimpleVoting contract
 * @notice This interface defines the external functions for simple voting sessions
 */
interface ISimpleVoting {
    // ============ Events ============
    
    event VoteSessionCreated(
        uint256 indexed sessionId,
        address indexed creator,
        string question,
        string ipfsHash,
        uint256 startTime,
        uint256 endTime
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
        bool result
    );
    
    // ============ Functions ============
    
    function createVoteSession(
        string calldata question,
        uint256 duration,
        string calldata ipfsHash
    ) external returns (uint256 sessionId);
    
    function voteInSession(uint256 sessionId, bool choice) external;
    
    function endVoteSession(uint256 sessionId) external;
    
    // ============ View Functions ============
    
    function getSessionCount() external view returns (uint256);
    
    function getVoteSession(uint256 sessionId)
        external
        view
        returns (
            uint256 id,
            address creator,
            string memory question,
            string memory ipfsHash,
            uint256 startTime,
            uint256 endTime,
            bool isActive,
            uint256 yesVotes,
            uint256 noVotes,
            uint256 totalParticipants
        );
    
    function getSessionResults(uint256 sessionId)
        external
        view
        returns (
            uint256 yesVotes,
            uint256 noVotes,
            uint256 totalParticipants,
            bool isValid,
            bool yesWinning
        );
    
    function isSessionActive(uint256 sessionId) external view returns (bool);
    
    function getTimeRemaining(uint256 sessionId) external view returns (uint256);
}