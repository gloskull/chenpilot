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
exports.TelegramAdapter = void 0;
const telegraf_1 = require("telegraf");
class TelegramAdapter {
  constructor(token) {
    this.userChatIds = new Map(); // userId -> chatId
    this.token = token;
  }
  init() {
    return __awaiter(this, void 0, void 0, function* () {
      if (!this.token) {
        console.warn(
          "⚠️ Telegram: No token provided, skipping initialization."
        );
        return;
      }
      this.bot = new telegraf_1.Telegraf(this.token);
      this.bot.start((ctx) =>
        ctx.reply(
          "Welcome to Chen Pilot! I am your AI-powered Stellar DeFi assistant."
        )
      );
      this.bot.help((ctx) => ctx.reply("Commands: /start, /balance, /swap"));
      this.bot.launch();
      console.log("✅ Telegram bot initialized.");
    });
  }
  /**
   * Register a user to receive notifications
   */
  registerUser(userId, chatId) {
    return __awaiter(this, void 0, void 0, function* () {
      this.userChatIds.set(userId, chatId);
      return true;
    });
  }
  /**
   * Send a transaction confirmation notification
   */
  sendTransactionNotification(userId, data) {
    return __awaiter(this, void 0, void 0, function* () {
      if (!this.bot) {
        console.warn("⚠️ Telegram bot not initialized");
        return false;
      }
      const chatId = this.userChatIds.get(userId);
      if (!chatId) {
        console.warn(`⚠️ No chat ID found for user ${userId}`);
        return false;
      }
      const message = this.formatTransactionMessage(data);
      try {
        yield this.bot.telegram.sendMessage(chatId, message, {
          parse_mode: "HTML",
        });
        return true;
      } catch (error) {
        console.error("Error sending Telegram notification:", error);
        return false;
      }
    });
  }
  /**
   * Format transaction notification message
   */
  formatTransactionMessage(data) {
    const statusEmoji = data.successful ? "✅" : "❌";
    const timestamp = new Date(data.timestamp).toLocaleString();
    let message = `<b>Transaction ${data.successful ? "Confirmed" : "Failed"}</b> ${statusEmoji}\n\n`;
    message += `📋 <b>Hash:</b> <code>${data.hash.slice(0, 8)}...${data.hash.slice(-8)}</code>\n`;
    message += `💰 <b>Amount:</b> ${data.amount} ${data.asset}\n`;
    message += `📤 <b>From:</b> <code>${data.from.slice(0, 4)}...${data.from.slice(-4)}</code>\n`;
    message += `📥 <b>To:</b> <code>${data.to.slice(0, 4)}...${data.to.slice(-4)}</code>\n`;
    message += `⏱️ <b>Time:</b> ${timestamp}\n`;
    if (data.fee) {
      message += `💵 <b>Fee:</b> ${data.fee} XLM\n`;
    }
    if (data.memo) {
      message += `📝 <b>Memo:</b> ${data.memo}\n`;
    }
    return message;
  }
  /**
   * Send a general notification to a user
   */
  sendNotification(userId, message) {
    return __awaiter(this, void 0, void 0, function* () {
      if (!this.bot) {
        console.warn("⚠️ Telegram bot not initialized");
        return false;
      }
      const chatId = this.userChatIds.get(userId);
      if (!chatId) {
        return false;
      }
      try {
        yield this.bot.telegram.sendMessage(chatId, message, {
          parse_mode: "HTML",
        });
        return true;
      } catch (error) {
        console.error("Error sending Telegram notification:", error);
        return false;
      }
    });
  }
}
exports.TelegramAdapter = TelegramAdapter;
