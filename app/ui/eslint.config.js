// @ts-check
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import solid from "eslint-plugin-solid/configs/typescript";
import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig(
  { ignores: ["dist/", "node_modules/"] },
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  { files: ["src/**/*.{ts,tsx}"], ...solid },
  { files: ["**/*.js"], extends: [tseslint.configs.disableTypeChecked] },
  prettier,
);
