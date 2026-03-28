# Requirements Document

## Introduction

This feature adds an AI-assisted Transaction Explanation service to the chenpilot backend. Before a user signs a Stellar transaction, the service decodes the XDR-encoded transaction envelope using the Stellar SDK, then passes a structured representation to an LLM (Anthropic Claude) to generate a plain-English summary of what the transaction will do. The explanation is returned to the client so the user can make an informed decision before signing.

## Glossary

- **XDR_Envelope**: A base64-encoded XDR (External Data Representation) string representing a Stellar `TransactionEnvelope`, as defined by the Stellar protocol.
- **Transaction_Explanation_Service**: The backend service responsible for decoding an XDR_Envelope and producing a human-readable explanation via an LLM.
- **LLM**: Large Language Model — specifically Anthropic Claude, already integrated in the project via `AgentLLM`.
- **Decoded_Transaction**: The structured JavaScript object produced by the Stellar SDK after parsing an XDR_Envelope.
- **Explanation**: A plain-English string summarising the intent, operations, fees, and risks of a Decoded_Transaction.
- **Caller**: An authenticated API client (user or frontend application) invoking the explanation endpoint.
- **Operation**: A single action within a Stellar transaction (e.g., payment, path payment, manage offer, invoke contract).

---

## Requirements

### Requirement 1: XDR Decoding

**User Story:** As a Caller, I want the service to decode an XDR_Envelope into a structured representation, so that the LLM receives accurate, structured data rather than raw binary.

#### Acceptance Criteria

1. WHEN a valid base64-encoded XDR_Envelope is provided, THE Transaction_Explanation_Service SHALL decode it into a Decoded_Transaction using the Stellar SDK without making any network calls.
2. IF the provided XDR_Envelope is not valid base64 or cannot be parsed as a Stellar TransactionEnvelope, THEN THE Transaction_Explanation_Service SHALL return an error response with HTTP status 400 and a descriptive message identifying the parse failure.
3. THE Transaction_Explanation_Service SHALL extract and include in the Decoded_Transaction: the source account, sequence number, base fee, list of Operations with their types and parameters, memo type and value, time bounds (if present), and the number of existing signatures.
4. THE Transaction_Explanation_Service SHALL support both `Transaction` and `FeeBumpTransaction` envelope types.
5. FOR ALL valid XDR_Envelopes, decoding then re-encoding the Decoded_Transaction SHALL produce an XDR string that parses to an equivalent transaction (round-trip property).

---

### Requirement 2: LLM-Generated Explanation

**User Story:** As a Caller, I want the service to produce a plain-English explanation of the decoded transaction, so that a non-technical user can understand what they are about to sign.

#### Acceptance Criteria

1. WHEN a Decoded_Transaction is available, THE Transaction_Explanation_Service SHALL send a structured prompt to the LLM containing the transaction details and request a human-readable explanation.
2. THE Transaction_Explanation_Service SHALL include in the Explanation: a one-sentence summary of the overall intent, a description of each Operation in plain English, the total fee in XLM, any time bounds expressed as human-readable dates, and a risk indicator (low / medium / high) based on operation types and amounts.
3. WHEN the LLM returns a response within 30 000 ms, THE Transaction_Explanation_Service SHALL parse and return the Explanation to the Caller.
4. IF the LLM call exceeds 30 000 ms, THEN THE Transaction_Explanation_Service SHALL return an error response with HTTP status 504 and the message "Explanation timed out".
5. IF the LLM returns a malformed or unparseable response, THEN THE Transaction_Explanation_Service SHALL return an error response with HTTP status 502 and a descriptive message.
6. THE Transaction_Explanation_Service SHALL NOT include the user's private key or any secret material in the LLM prompt.

---

### Requirement 3: REST API Endpoint

**User Story:** As a Caller, I want a dedicated REST endpoint to request a transaction explanation, so that the frontend can integrate it into the signing flow.

#### Acceptance Criteria

1. THE Transaction_Explanation_Service SHALL expose a `POST /api/transactions/explain` endpoint that accepts a JSON body containing a `xdrEnvelope` string field and an optional `network` field (`"testnet"` or `"mainnet"`, defaulting to `"testnet"`).
2. WHEN a request is received with a valid `xdrEnvelope`, THE Transaction_Explanation_Service SHALL respond with HTTP status 200 and a JSON body containing `success: true`, an `explanation` object with the fields defined in Requirement 2 AC 2, and a `transactionHash` string.
3. IF the request body is missing the `xdrEnvelope` field, THEN THE Transaction_Explanation_Service SHALL return HTTP status 400 with `success: false` and a descriptive error message.
4. WHILE a Caller is not authenticated, THE Transaction_Explanation_Service SHALL return HTTP status 401 and reject the request.
5. THE Transaction_Explanation_Service SHALL apply a rate limit of 20 requests per minute per authenticated user to prevent LLM cost abuse.
6. IF the `network` field is provided but is not `"testnet"` or `"mainnet"`, THEN THE Transaction_Explanation_Service SHALL return HTTP status 400 with a descriptive error message.

---

### Requirement 4: Prompt Construction and Safety

**User Story:** As a system operator, I want the LLM prompt to be structured and injection-safe, so that malicious transaction memos or data fields cannot alter the model's behaviour.

#### Acceptance Criteria

1. THE Transaction_Explanation_Service SHALL construct the LLM prompt by serialising the Decoded_Transaction fields into a structured format (e.g., JSON) and wrapping user-supplied string values (memo, manage-data values) in XML-style delimiter tags before insertion into the prompt.
2. THE Transaction_Explanation_Service SHALL instruct the LLM via a system prompt to treat delimited content as data only and to produce a JSON-structured response.
3. IF a memo or data field contains characters that could break prompt structure (e.g., `<`, `>`), THE Transaction_Explanation_Service SHALL HTML-encode those characters before including them in the prompt.
4. THE Transaction_Explanation_Service SHALL request the LLM response in a defined JSON schema containing `summary`, `operations` (array), `feeLumens`, `timeBounds`, and `riskLevel` fields.

---

### Requirement 5: Caching

**User Story:** As a system operator, I want identical XDR envelopes to return cached explanations, so that repeated requests for the same transaction do not incur redundant LLM calls.

#### Acceptance Criteria

1. THE Transaction_Explanation_Service SHALL cache Explanation results keyed by the SHA-256 hash of the XDR_Envelope string for a duration of 300 seconds.
2. WHEN a Caller submits an XDR_Envelope whose SHA-256 hash matches a cached entry, THE Transaction_Explanation_Service SHALL return the cached Explanation without invoking the LLM.
3. WHEN a cached entry expires, THE Transaction_Explanation_Service SHALL remove it from the cache and invoke the LLM on the next matching request.
4. THE Transaction_Explanation_Service SHALL use an in-process cache (Map-based, consistent with the existing `TransactionHistoryService` pattern) and SHALL NOT require an external cache dependency for this feature.

---

### Requirement 6: Logging and Audit

**User Story:** As a system operator, I want explanation requests to be logged, so that I can monitor usage, debug failures, and audit LLM interactions.

#### Acceptance Criteria

1. WHEN an explanation request is received, THE Transaction_Explanation_Service SHALL log the authenticated user ID, the `transactionHash`, the `network`, and the request timestamp at INFO level using the existing logger.
2. WHEN an LLM call completes successfully, THE Transaction_Explanation_Service SHALL log the latency in milliseconds and whether the result was served from cache at DEBUG level.
3. IF an error occurs during XDR decoding or LLM invocation, THE Transaction_Explanation_Service SHALL log the error details at ERROR level including the user ID and a sanitised description of the failure (excluding raw XDR data).
4. THE Transaction_Explanation_Service SHALL record an audit log entry via `auditLogService` for each explanation request, using an appropriate `AuditAction` and `AuditSeverity.INFO`.
