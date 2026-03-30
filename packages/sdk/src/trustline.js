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
exports.hasValidStellarTrustline = hasValidStellarTrustline;
// @ts-ignore: dependency is provided at the workspace root
const stellar_sdk_1 = require("stellar-sdk");
/**
 * Checks whether an account has a valid, non-frozen trustline for an asset.
 * - For native XLM the function returns exists=true, authorized=true.
 * - For other assets it fetches the account from Horizon and inspects balances.
 *
 * @param horizonUrl Horizon server URL (defaults to public Horizon)
 * @param accountId Stellar account id to check
 * @param assetCode Asset code (string; "XLM" for native)
 * @param assetIssuer Asset issuer public key (optional for native/XLM)
 */
function hasValidStellarTrustline(
  horizonUrl,
  accountId,
  assetCode,
  assetIssuer
) {
  return __awaiter(this, void 0, void 0, function* () {
    var _a, _b, _c;
    const server = new stellar_sdk_1.Server(
      horizonUrl || "https://horizon.stellar.org"
    );
    // Native XLM does not require a trustline
    if (!assetCode || assetCode.toUpperCase() === "XLM") {
      return { exists: true, authorized: true };
    }
    let account;
    try {
      account = yield server.accounts().accountId(accountId).call();
    } catch (err) {
      return {
        exists: false,
        authorized: false,
        details: { error: String(err) },
      };
    }
    const balances = account.balances || [];
    const match = balances.find((b) => {
      return (
        b.asset_code === assetCode &&
        (assetIssuer ? b.asset_issuer === assetIssuer : true)
      );
    });
    if (!match) {
      return { exists: false, authorized: false };
    }
    // Horizon may include authorization properties on the trustline/balance object
    const authorized =
      // common property name
      (_c =
        (_b =
          (_a = match.is_authorized) !== null && _a !== void 0
            ? _a
            : match.authorized) !== null && _b !== void 0
          ? _b
          : // if issuer uses 'authorized_to_maintain_liabilities' treat as authorized
            match.authorized_to_maintain_liabilities) !== null && _c !== void 0
        ? _c
        : true;
    return { exists: true, authorized, details: { balance: match } };
  });
}
exports.default = hasValidStellarTrustline;
