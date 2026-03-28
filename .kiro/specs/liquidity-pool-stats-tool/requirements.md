# Requirements Document

## Introduction

This feature adds a new agent tool — `LiquidityPoolStatsTool` — to the chenpilot project. Unlike the existing `stellarLiquidityTool` (which queries order book data for XLM pairs via the DEX), this tool targets Stellar AMM (Automated Market Maker) liquidity pools directly. It fetches pool-level statistics including total liquidity (reserves), 24-hour trading volume, and estimated APR, then returns them in a structured format consumable by the AI agent layer.

The tool follows the existing `BaseTool` / `ToolDefinition` pattern and is auto-discovered via `ToolAutoDiscovery`.

## Glossary

- **Tool**: A class extending `BaseTool` that implements `ToolDefinition` and is registered in the tool registry.
- **LiquidityPoolStatsTool**: The new tool being specified in this document.
- **Liquidity_Pool**: A Stellar AMM pool identified by a unique pool ID on the Stellar network.
- **Pool_ID**: A 64-character hex string uniquely identifying a Stellar AMM liquidity pool.
- **Horizon_API**: The Stellar Horizon REST API, accessed via the configured `stellar.horizonUrl`.
- **Reserve**: The amount of a specific asset held in a liquidity pool at a given time.
- **Volume**: The total value of trades routed through a liquidity pool over a rolling 24-hour window, as reported by Horizon.
- **APR**: Annual Percentage Rate — an annualized estimate of fee yield based on 24-hour volume and total pool reserves.
- **Fee_Percentage**: The fixed swap fee for a Stellar AMM pool (typically 0.30%).
- **ToolResult**: The standard return type defined in `ToolMetadata.ts` with `action`, `status`, and `data` fields.
- **ToolPayload**: The input record type passed to `execute`.

---

## Requirements

### Requirement 1: Fetch Liquidity Pool Statistics by Pool ID

**User Story:** As an AI agent, I want to retrieve statistics for a specific Stellar AMM liquidity pool by its pool ID, so that I can answer user questions about pool performance and health.

#### Acceptance Criteria

1. WHEN a valid `poolId` is provided, THE LiquidityPoolStatsTool SHALL query the Horizon_API `/liquidity_pools/{poolId}` endpoint and return pool statistics.
2. THE LiquidityPoolStatsTool SHALL include the following fields in the response data: `poolId`, `assetA`, `assetB`, `reserveA`, `reserveB`, `totalShares`, `totalTrustlines`, `fee`, `volume24h`, `apr`, and `timestamp`.
3. WHEN the Horizon_API returns a successful response, THE LiquidityPoolStatsTool SHALL parse the reserve amounts as floating-point numbers rounded to 7 decimal places.
4. THE LiquidityPoolStatsTool SHALL return results as a `ToolResult` with `status: "success"` and all statistics nested under the `data` field.

---

### Requirement 2: Input Validation

**User Story:** As a developer integrating the tool, I want invalid inputs to be rejected with clear error messages, so that the agent receives actionable feedback rather than cryptic API errors.

#### Acceptance Criteria

1. THE LiquidityPoolStatsTool SHALL define `poolId` as a required string parameter in its `ToolMetadata`.
2. WHEN the `poolId` parameter is absent from the payload, THE LiquidityPoolStatsTool SHALL return a validation error listing the missing parameter.
3. WHEN the `poolId` value is not a 64-character hexadecimal string, THE LiquidityPoolStatsTool SHALL return a validation error with the message `"poolId must be a 64-character hexadecimal string"`.
4. THE LiquidityPoolStatsTool SHALL perform validation before making any network request to the Horizon_API.

---

### Requirement 3: Error Handling

**User Story:** As an AI agent, I want meaningful error messages when a pool lookup fails, so that I can relay useful information to the user instead of a raw exception.

#### Acceptance Criteria

1. IF the Horizon_API returns a 404 response for the given `poolId`, THEN THE LiquidityPoolStatsTool SHALL return a `ToolResult` with `status: "error"` and `error: "Liquidity pool not found. Verify the pool ID exists on the configured Stellar network."`.
2. IF the Horizon_API returns any non-404 HTTP error, THEN THE LiquidityPoolStatsTool SHALL return a `ToolResult` with `status: "error"` and include the HTTP status code in the error message.
3. IF a network timeout or connection error occurs, THEN THE LiquidityPoolStatsTool SHALL return a `ToolResult` with `status: "error"` and `error: "Failed to reach Horizon API. Check network connectivity."`.
4. WHEN an error occurs during execution, THE LiquidityPoolStatsTool SHALL log the full error details using the Winston logger before returning the error result.

---

### Requirement 4: APR Calculation

**User Story:** As a user, I want to see an estimated APR for a liquidity pool, so that I can evaluate whether providing liquidity is worthwhile.

#### Acceptance Criteria

1. THE LiquidityPoolStatsTool SHALL calculate APR using the formula: `APR = (volume24h × fee_percentage × 365) / total_liquidity_usd_equivalent × 100`, where `total_liquidity_usd_equivalent` is the sum of both reserves expressed in a common unit.
2. WHEN `volume24h` is zero or unavailable from the Horizon_API, THE LiquidityPoolStatsTool SHALL return `apr: 0` rather than an error.
3. WHEN `total_liquidity_usd_equivalent` is zero, THE LiquidityPoolStatsTool SHALL return `apr: 0` to avoid division by zero.
4. THE LiquidityPoolStatsTool SHALL express the returned `apr` value as a percentage rounded to 2 decimal places (e.g., `4.72`).

---

### Requirement 5: Tool Registration and Discovery

**User Story:** As a developer, I want the new tool to be automatically discovered and registered, so that no manual wiring is needed beyond placing the file in the tools directory.

#### Acceptance Criteria

1. THE LiquidityPoolStatsTool SHALL extend `BaseTool` and export a class that `ToolAutoDiscovery` can instantiate without constructor arguments.
2. THE LiquidityPoolStatsTool SHALL declare a `metadata` object conforming to the `ToolMetadata` interface with `name: "get_liquidity_pool_stats"`, a descriptive `description`, `category: "stellar"`, and `version: "1.0.0"`.
3. THE LiquidityPoolStatsTool SHALL be placed in `src/Agents/tools/` so that `ToolAutoDiscovery` includes it in the registry scan.
4. WHEN the tool registry is initialized, THE ToolAutoDiscovery SHALL register the LiquidityPoolStatsTool alongside existing tools without requiring changes to registry configuration files.
