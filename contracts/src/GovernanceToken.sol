// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
// Note: Nonces.sol not available in OpenZeppelin v4.9.6

/**
 * @title GovernanceToken
 * @dev ERC20 token with voting capabilities for Somnia Governance Engine
 * @notice This token provides voting power for governance proposals
 * Features: Voting delegation, permit functionality, secure minting
 */
contract GovernanceToken is ERC20, ERC20Permit, ERC20Votes, Ownable, ReentrancyGuard {
    /// @dev Maximum supply cap (100 million tokens)
    uint256 public constant MAX_SUPPLY = 100_000_000 * 10**18;
    
    /// @dev Minimum time between mints to prevent rapid inflation
    uint256 public constant MIN_MINT_INTERVAL = 86400; // 24 hours
    
    /// @dev Maximum mint amount per transaction (1% of total supply)
    uint256 public constant MAX_MINT_AMOUNT = 1_000_000 * 10**18;
    
    /// @dev Last mint timestamp
    uint256 public lastMintTime;
    
    /// @dev Minting enabled/disabled
    bool public mintingEnabled = true;
    
    /// @dev Emergency pause mechanism
    bool public paused = false;
    
    // ============ Events ============
    
    event MintingDisabled();
    event EmergencyPause(bool paused);
    event TokensMinted(address indexed to, uint256 amount);
    
    // ============ Modifiers ============
    
    modifier whenNotPaused() {
        require(!paused, "GovernanceToken: contract is paused");
        _;
    }
    
    modifier onlyMinter() {
        require(mintingEnabled, "GovernanceToken: minting disabled");
        require(
            block.timestamp >= lastMintTime + MIN_MINT_INTERVAL,
            "GovernanceToken: mint too soon"
        );
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        string memory name,
        string memory symbol,
        address initialOwner,
        uint256 initialSupply
    ) ERC20(name, symbol) ERC20Permit(name) {
        _transferOwnership(initialOwner);
        require(initialOwner != address(0), "GovernanceToken: zero address");
        require(initialSupply <= MAX_SUPPLY, "GovernanceToken: initial supply too high");
        
        // Ownership transferred in constructor
        
        if (initialSupply > 0) {
            _mint(initialOwner, initialSupply);
        }
        
        lastMintTime = block.timestamp;
    }
    
    // ============ Minting Functions ============
    
    /**
     * @dev Mint new tokens (with restrictions)
     * @param to Address to receive tokens
     * @param amount Amount to mint
     */
    function mint(address to, uint256 amount) 
        external 
        onlyOwner 
        onlyMinter 
        nonReentrant 
        whenNotPaused 
    {
        require(to != address(0), "GovernanceToken: mint to zero address");
        require(amount > 0, "GovernanceToken: mint amount is zero");
        require(amount <= MAX_MINT_AMOUNT, "GovernanceToken: amount exceeds max mint");
        require(totalSupply() + amount <= MAX_SUPPLY, "GovernanceToken: exceeds max supply");
        
        lastMintTime = block.timestamp;
        _mint(to, amount);
        
        emit TokensMinted(to, amount);
    }
    
    /**
     * @dev Permanently disable minting (cannot be undone)
     */
    function disableMinting() external onlyOwner {
        mintingEnabled = false;
        emit MintingDisabled();
    }
    
    // ============ Emergency Functions ============
    
    /**
     * @dev Emergency pause/unpause (for security incidents)
     * @param _paused True to pause, false to unpause
     */
    function emergencyPause(bool _paused) external onlyOwner {
        paused = _paused;
        emit EmergencyPause(_paused);
    }
    
    // ============ Voting Power Functions ============
    
    /**
     * @dev Get current voting power of an account
     * @param account Address to check
     * @return Current voting power
     */
    function getVotingPower(address account) external view returns (uint256) {
        return getVotes(account);
    }
    
    /**
     * @dev Get voting power at a specific block
     * @param account Address to check
     * @param blockNumber Block number to check
     * @return Historical voting power
     */
    function getPastVotingPower(address account, uint256 blockNumber) 
        external 
        view 
        returns (uint256) 
    {
        return getPastVotes(account, blockNumber);
    }
    
    /**
     * @dev Get total voting power (total supply with delegations)
     * @return Total voting power in circulation
     */
    function getTotalVotingPower() external view returns (uint256) {
        return totalSupply();
    }
    
    // ============ Delegation Helpers ============
    
    /**
     * @dev Delegate voting power to another address
     * @param delegatee Address to delegate to
     */
    function delegateVotingPower(address delegatee) external whenNotPaused {
        delegate(delegatee);
    }
    
    /**
     * @dev Delegate voting power by signature (gasless)
     * @param delegatee Address to delegate to
     * @param nonce Nonce for replay protection
     * @param expiry Signature expiry
     * @param v Signature v parameter
     * @param r Signature r parameter
     * @param s Signature s parameter
     */
    function delegateVotingPowerBySig(
        address delegatee,
        uint256 nonce,
        uint256 expiry,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external whenNotPaused {
        delegateBySig(delegatee, nonce, expiry, v, r, s);
    }
    
    // ============ Override Functions ============
    
    function _mint(address account, uint256 amount) internal virtual override(ERC20, ERC20Votes) {
        super._mint(account, amount);
    }

    function _burn(address account, uint256 amount) internal virtual override(ERC20, ERC20Votes) {
        super._burn(account, amount);
    }

    function _afterTokenTransfer(address from, address to, uint256 amount) internal virtual override(ERC20, ERC20Votes) {
        super._afterTokenTransfer(from, to, amount);
    }
    
    function nonces(address owner) public view virtual override(ERC20Permit) returns (uint256) {
        return super.nonces(owner);
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Check if account has minimum voting power for proposals
     * @param account Address to check
     * @param threshold Minimum threshold required
     * @return True if account has sufficient voting power
     */
    function hasVotingPower(address account, uint256 threshold) external view returns (bool) {
        return getVotes(account) >= threshold;
    }
    
    /**
     * @dev Get delegation info for an account
     * @param account Address to check
     * @return delegate Current delegate
     * @return votingPower Current voting power
     * @return balance Token balance
     */
    function getDelegationInfo(address account) 
        external 
        view 
        returns (
            address delegate,
            uint256 votingPower,
            uint256 balance
        ) 
    {
        return (
            delegates(account),
            getVotes(account),
            balanceOf(account)
        );
    }
    
    /**
     * @dev Get contract status
     * @return mintingEnabled_ Whether minting is enabled
     * @return paused_ Whether contract is paused
     * @return totalSupply_ Current total supply
     * @return maxSupply_ Maximum possible supply
     */
    function getContractStatus() 
        external 
        view 
        returns (
            bool mintingEnabled_,
            bool paused_,
            uint256 totalSupply_,
            uint256 maxSupply_
        ) 
    {
        return (
            mintingEnabled,
            paused,
            totalSupply(),
            MAX_SUPPLY
        );
    }
}