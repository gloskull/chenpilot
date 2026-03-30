"use strict";
import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
export default [
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts}"],
    languageOptions: {
      globals: Object.assign(Object.assign({}, globals.node), globals.es2021),
    },
    rules: {
      "@typescript-eslint/no-namespace": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": "warn",
      "@typescript-eslint/ban-ts-comment": "warn",
      "no-case-declarations": "warn",
    },
  },
  {
    files: ["**/*.test.{js,ts}", "**/tests/**/*.{js,ts}"],
    languageOptions: {
      globals: globals.jest,
    },
  },
  {
    ignores: [
      "node_modules/**",
      "dist/**",
      "build/**",
      "tests/**/*.js",
      "packages/**/*.js",
      "scripts/**/*.js",
    ],
  },
];
