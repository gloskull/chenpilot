"use strict";
var __awaiter =
  (this && this.__awaiter) ||
  function (thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P
        ? value
        : new P(function (resolve) {
            resolve(value);
          });
    }
    return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done
          ? resolve(result.value)
          : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.SorobanEventSubscription = void 0;
exports.subscribeToEvents = subscribeToEvents;
exports.parseEvent = parseEvent;
// ─── Constants ──────────────────────────────────────────────────────────────
const DEFAULT_RPC_URLS = {
  testnet: "https://soroban-testnet.stellar.org",
  mainnet: "https://soroban-mainnet.stellar.org",
};
const DEFAULT_POLLING_INTERVAL_MS = 5000;
// ─── Event subscription implementation ──────────────────────────────────────
/**
 * High-level API for subscribing to Soroban contract events.
 *
 * Polls the Soroban RPC at regular intervals to fetch new events from
 * specified contracts and invoke handlers for matching events.
 *
 * @example
 * ```typescript
 * const subscription = subscribeToEvents({
 *   network: "testnet",
 *   contractIds: ["CABC1234..."],
 *   topicFilter: ["transfer"],
 * });
 *
 * subscription.on("event", (event) => {
 *   console.log("Transfer event:", event.topics, event.data);
 * });
 *
 * subscription.on("error", (err) => {
 *   console.error("Subscription error:", err);
 * });
 *
 * // Later...
 * await subscription.unsubscribe();
 * ```
 */
class SorobanEventSubscription {
  constructor(config) {
    this.isActive_ = false;
    this.lastLedger_ = null;
    this.pollingHandle_ = null;
    this.eventHandlers = new Set();
    this.errorHandlers = new Set();
    this.processedTransactions = new Set();
    if (!config.contractIds || config.contractIds.length === 0) {
      throw new Error("At least one contractId is required");
    }
    this.config = config;
    this.rpcUrl = config.rpcUrl || DEFAULT_RPC_URLS[config.network];
    if (!this.rpcUrl) {
      throw new Error(`Unknown network: ${config.network}`);
    }
  }
  on(event, handler) {
    if (event === "event") {
      this.eventHandlers.add(handler);
    } else if (event === "error") {
      this.errorHandlers.add(handler);
    }
    return this;
  }
  off(event, handler) {
    if (event === "event") {
      this.eventHandlers.delete(handler);
    } else if (event === "error") {
      this.errorHandlers.delete(handler);
    }
    return this;
  }
  /**
   * Start polling for events.
   */
  subscribe() {
    return __awaiter(this, void 0, void 0, function* () {
      var _a;
      if (this.isActive_) {
        return; // Already subscribed
      }
      this.isActive_ = true;
      const interval =
        (_a = this.config.pollingIntervalMs) !== null && _a !== void 0
          ? _a
          : DEFAULT_POLLING_INTERVAL_MS;
      // Run once immediately
      yield this.poll();
      // Then set up polling
      this.pollingHandle_ = setInterval(() => {
        this.poll().catch((err) => this.emitError(err));
      }, interval);
    });
  }
  /**
   * Stop polling and clean up resources.
   */
  unsubscribe() {
    return __awaiter(this, void 0, void 0, function* () {
      if (!this.isActive_) {
        return;
      }
      this.isActive_ = false;
      if (this.pollingHandle_) {
        clearInterval(this.pollingHandle_);
        this.pollingHandle_ = null;
      }
      this.eventHandlers.clear();
      this.errorHandlers.clear();
    });
  }
  /**
   * Get the current subscription status.
   */
  isActive() {
    return this.isActive_;
  }
  /**
   * Get the last ledger that was checked.
   */
  getLastLedger() {
    return this.lastLedger_;
  }
  // ─── Private helpers ────────────────────────────────────────────────────
  poll() {
    return __awaiter(this, void 0, void 0, function* () {
      try {
        // In a real implementation, this would call an RPC method like
        // getLedgerEvents (if available) or iterate through recent ledgers.
        // For now, we use a simulation approach.
        const events = yield this.fetchRecentEvents();
        for (const event of events) {
          yield this.emitEvent(event);
        }
      } catch (error) {
        this.emitError(
          error instanceof Error ? error : new Error(String(error))
        );
      }
    });
  }
  fetchRecentEvents() {
    return __awaiter(this, void 0, void 0, function* () {
      // This is a placeholder. In production, you would:
      // 1. Query the RPC for recent ledgers
      // 2. Fetch transactions from those ledgers
      // 3. Filter by contract ID and extract events
      // For now, return empty to demonstrate the interface
      return [];
    });
  }
  emitEvent(event) {
    return __awaiter(this, void 0, void 0, function* () {
      // Avoid duplicate processing
      if (this.processedTransactions.has(event.transactionHash)) {
        return;
      }
      this.processedTransactions.add(event.transactionHash);
      // Apply topic filter if configured
      if (this.config.topicFilter && this.config.topicFilter.length > 0) {
        const hasMatchingTopic = event.topics.some((topic) =>
          this.config.topicFilter.some((filter) => topic.includes(filter))
        );
        if (!hasMatchingTopic) {
          return;
        }
      }
      // Invoke all registered handlers
      for (const handler of this.eventHandlers) {
        try {
          yield handler(event);
        } catch (err) {
          this.emitError(err instanceof Error ? err : new Error(String(err)));
        }
      }
    });
  }
  emitError(error) {
    for (const handler of this.errorHandlers) {
      try {
        void handler(error);
      } catch (_a) {
        // Ignore errors in error handlers
      }
    }
  }
}
exports.SorobanEventSubscription = SorobanEventSubscription;
/**
 * Subscribe to Soroban contract events.
 *
 * Creates and starts an event subscription for the specified contracts.
 *
 * @param config - Subscription configuration
 * @returns Active subscription object
 *
 * @example
 * ```typescript
 * const subscription = subscribeToEvents({
 *   network: "testnet",
 *   contractIds: ["CABC1234567890"],
 *   pollingIntervalMs: 3000,
 * });
 *
 * subscription.on("event", (event) => {
 *   console.log("Event received:", event);
 * });
 *
 * await subscription.subscribe(); // Start polling
 * ```
 */
function subscribeToEvents(config) {
  return __awaiter(this, void 0, void 0, function* () {
    const subscription = new SorobanEventSubscription(config);
    yield subscription.subscribe();
    return subscription;
  });
}
/**
 * Parse a raw RPC event into a structured SorobanEvent.
 *
 * @param raw - Raw event from RPC
 * @param contractId - Contract that emitted the event
 * @param transactionHash - Transaction hash
 * @param ledger - Ledger sequence
 * @param createdAt - Ledger close time
 * @returns Parsed event
 */
function parseEvent(raw, contractId, transactionHash, ledger, createdAt) {
  var _a;
  return {
    transactionHash,
    contractId,
    topics: Array.isArray(raw.topic)
      ? raw.topic.map((t) => (typeof t === "string" ? t : JSON.stringify(t)))
      : [],
    data: (_a = raw.value) !== null && _a !== void 0 ? _a : null,
    ledger,
    createdAt,
  };
}
