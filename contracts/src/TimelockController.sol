// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/governance/TimelockController.sol";

/**
 * @title SomniaTimelockController
 * @dev Timelock controller for secure governance proposal execution
 * @notice Provides time-delayed execution of governance proposals
 * Prevents immediate execution of critical changes
 */
contract SomniaTimelockController is TimelockController {
    /// @dev Minimum delay for standard proposals (24 hours)
    uint256 public constant STANDARD_DELAY = 86400;
    
    /// @dev Minimum delay for emergency proposals (1 hour)
    uint256 public constant EMERGENCY_DELAY = 3600;
    
    /// @dev Minimum delay for constitutional proposals (7 days)
    uint256 public constant CONSTITUTIONAL_DELAY = 604800;
    
    /**
     * @dev Constructor
     * @param minDelay Minimum delay for all operations
     * @param proposers Array of addresses that can propose
     * @param executors Array of addresses that can execute (empty for anyone)
     * @param admin Admin address (optional, can be zero address)
     */
    constructor(
        uint256 minDelay,
        address[] memory proposers,
        address[] memory executors,
        address admin
    ) TimelockController(minDelay, proposers, executors, admin) {
        // TimelockController handles the initialization
    }
    
    /**
     * @dev Get recommended delay for proposal type
     * @param proposalType Type of proposal (0=Standard, 1=Emergency, 2=Constitutional)
     * @return Recommended delay in seconds
     */
    function getRecommendedDelay(uint8 proposalType) external pure returns (uint256) {
        if (proposalType == 1) { // Emergency
            return EMERGENCY_DELAY;
        } else if (proposalType == 2) { // Constitutional
            return CONSTITUTIONAL_DELAY;
        } else { // Standard
            return STANDARD_DELAY;
        }
    }
}