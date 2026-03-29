// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title EmergencyControl
 * @dev Implements a multi-sig/governance controlled circuit breaker.
 * Allows roles to pause and unpause the contract in case of an exploit or emergency.
 */
abstract contract EmergencyControl is AccessControl, Pausable {
    bytes32 public constant EMERGENCY_ROLE = keccak256("EMERGENCY_ROLE");

    event EmergencyPaused(address indexed account);
    event EmergencyUnpaused(address indexed account);

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(EMERGENCY_ROLE, msg.sender);
    }

    /**
     * @dev Pauses the contract. Only callable by accounts with EMERGENCY_ROLE.
     */
    function pause() external onlyRole(EMERGENCY_ROLE) {
        _pause();
        emit EmergencyPaused(msg.sender);
    }

    /**
     * @dev Unpauses the contract. Only callable by accounts with EMERGENCY_ROLE.
     */
    function unpause() external onlyRole(EMERGENCY_ROLE) {
        _unpause();
        emit EmergencyUnpaused(msg.sender);
    }

    /**
     * @dev Internal helper to check if the contract is currently paused.
     */
    function isEmergencyPaused() public view returns (bool) {
        return paused();
    }
}
