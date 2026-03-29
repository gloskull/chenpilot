// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./access/EmergencyControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title CoreEngine
 * @dev Main contract for AI-driven swap and liquidity operations with emergency evacuation.
 */
contract CoreEngine is EmergencyControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // Track user principal deposits (excluding yield/gains)
    mapping(address => mapping(address => uint256)) public userPrincipal;

    event Deposited(address indexed user, address indexed token, uint256 amount);
    event Swapped(address indexed user, address indexed fromToken, address indexed toToken, uint256 amount);
    event Rebalanced(address indexed user, address[] tokens, uint256[] amounts);
    event EmergencyWithdrawn(address indexed user, address indexed token, uint256 amount);

    /**
     * @dev Deposit principal into the contract.
     * @param token Address of the token to deposit.
     * @param amount Amount of tokens to deposit.
     */
    function deposit(address token, uint256 amount) external whenNotPaused nonReentrant {
        require(amount > 0, "Amount must be greater than zero");
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        userPrincipal[msg.sender][token] += amount;
        emit Deposited(msg.sender, token, amount);
    }

    /**
     * @dev Execute an AI-driven swap. Gated by whenNotPaused.
     * @param fromToken Token to swap from.
     * @param toToken Token to swap to.
     * @param amount Amount to swap.
     */
    function swap(address fromToken, address toToken, uint256 amount) external whenNotPaused nonReentrant {
        require(amount > 0, "Amount must be greater than zero");
        // AI-driven swap logic would go here
        // For this task, we assume the logic exists and ensure it is gated.
        emit Swapped(msg.sender, fromToken, toToken, amount);
    }

    /**
     * @dev Rebalance the portfolio. Gated by whenNotPaused.
     * @param tokens Array of tokens to rebalance.
     * @param amounts Array of target amounts.
     */
    function rebalance(address[] calldata tokens, uint256[] calldata amounts) external whenNotPaused nonReentrant {
        require(tokens.length == amounts.length, "Invalid input lengths");
        // Rebalancing logic would go here
        emit Rebalanced(msg.sender, tokens, amounts);
    }

    /**
     * @dev Emergency evacuation of principal funds.
     * Only available when the contract is paused.
     * Returns only the original principal deposited by the user.
     * @param token Address of the token to withdraw.
     */
    function emergencyWithdraw(address token) external whenPaused nonReentrant {
        uint256 principal = userPrincipal[msg.sender][token];
        require(principal > 0, "No principal to withdraw");

        // Reset principal before transfer to prevent re-entrancy
        userPrincipal[msg.sender][token] = 0;

        // Perform the transfer
        IERC20(token).safeTransfer(msg.sender, principal);

        emit EmergencyWithdrawn(msg.sender, token, principal);
    }
}
