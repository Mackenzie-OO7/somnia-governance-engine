// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IGovernanceHub
 * @dev Interface for GovernanceHub contract
 * @notice This interface defines the external functions for the governance system
 * Used for type-safe integration with the Rust backend
 */
interface IGovernanceHub {
    // ============ Enums ============
    
    enum ProposalType {
        Standard,
        Emergency,
        Constitutional
    }
    
    enum ProposalStatus {
        Pending,
        Active,
        Succeeded,
        Failed,
        Executed
    }
    
    enum VoteChoice {
        Against,
        For,
        Abstain
    }
    
    // ============ Events ============
    
    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        string ipfsHash,
        uint256 startTime,
        uint256 endTime,
        uint8 proposalType
    );
    
    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        uint8 choice,
        uint256 power,
        uint256 timestamp,
        string ipfsHash
    );
    
    event ProposalExecuted(
        uint256 indexed proposalId,
        address indexed executor
    );
    
    // ============ Functions ============
    
    function createProposal(
        string calldata ipfsHash,
        uint256 duration,
        ProposalType proposalType
    ) external returns (uint256 proposalId);
    
    function vote(
        uint256 proposalId,
        VoteChoice choice,
        string calldata ipfsHash
    ) external;
    
    function finalizeProposal(uint256 proposalId) external;
    
    function executeProposal(uint256 proposalId) external;
    
    // ============ View Functions ============
    
    function getProposalCount() external view returns (uint256);
    
    function getProposal(uint256 proposalId)
        external
        view
        returns (
            uint256 id,
            address proposer,
            string memory ipfsHash,
            uint256 startTime,
            uint256 endTime,
            ProposalType proposalType,
            ProposalStatus status,
            uint256 forVotes,
            uint256 againstVotes,
            uint256 totalVotingPower
        );
    
    function getVote(uint256 proposalId, address voter)
        external
        view
        returns (
            address voterAddr,
            VoteChoice choice,
            uint256 power,
            uint256 timestamp,
            string memory ipfsHash
        );
    
    function getVoters(uint256 proposalId) external view returns (address[] memory);
    
    function getVotingPower(address account) external view returns (uint256);
    
    function getTotalVotingPower() external view returns (uint256);
}