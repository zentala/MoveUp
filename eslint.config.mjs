import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";

/**
 * Flat ESLint config for the React + TypeScript frontend (`src/`).
 * Rust (`src-tauri/`) and build output are out of scope.
 */
export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "src/generated/**",
      "coverage/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  reactHooks.configs.flat["recommended-latest"],
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      globals: { ...globals.browser, ...globals.es2020 },
    },
    plugins: { "react-refresh": reactRefresh },
    rules: {
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    // Baseline downgrades — each of these rules flags pre-existing code in
    // files this task may not touch. They stay enabled as warnings so the
    // violations are visible; tightening them back to "error" is tracked in
    // .plan/BACKLOG.md ("ESLint baseline downgrades", E018-T04, 2026-09-06).
    files: ["src/**/*.{ts,tsx}"],
    rules: {
      "react-hooks/rules-of-hooks": "warn",
      "react-hooks/set-state-in-effect": "warn",
      "react-hooks/immutability": "warn",
      "react-hooks/purity": "error",
      "no-useless-assignment": "error",
      "prefer-const": "error",
    },
  },
  {
    // Tests may reach for `any` when stubbing Tauri IPC payloads.
    files: ["src/**/*.test.{ts,tsx}", "src/test/**/*.{ts,tsx}"],
    languageOptions: {
      globals: { ...globals.browser, ...globals.node, ...globals.vitest },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
);
