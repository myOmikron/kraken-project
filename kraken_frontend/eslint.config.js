// @ts-check

import eslint from "@eslint/js";
import parser from "@typescript-eslint/parser";
import jsdoc from "eslint-plugin-jsdoc";
import tsEslint from "typescript-eslint";

const config = tsEslint.config(
    eslint.configs.recommended,
    ...tsEslint.configs.recommended,
    jsdoc.configs["flat/recommended-typescript"],
    {
        ignores: ["src/api/generated/**"],
    },
    {
        languageOptions: {
            parser,
            parserOptions: { project: ["./tsconfig.json"] },
        },
        rules: {
            "no-case-declarations": "off", // potential errors are already caught by typescript

            "@typescript-eslint/switch-exhaustiveness-check": "error",

            "@typescript-eslint/ban-ts-comment": [
                "error",
                { "ts-ignore": "allow-with-description" }, //
            ],

            "@typescript-eslint/ban-types": [
                "error",
                { extendDefaults: true, types: { "{}": false } }, // its just syntactically nicer and consistent to use `type ...Props = {};` and extend it later
            ],

            "@typescript-eslint/no-unused-vars": [
                "error",
                { varsIgnorePattern: "^_", argsIgnorePattern: "^_|props" }, // mimic rust behaviour and ignore the props argument of functional components
            ],

            "@typescript-eslint/no-namespace": "off", // TODO: needs second thought / discussion with team
        },
    },
);
// disableAllBut("prefer-const"); // Hack to disable all rules when using `--fix`
export default config;

function disableAllBut(rule, options) {
    for (const entry of config) {
        if ("rules" in entry) entry.rules = {};
    }
    config.push({
        rules: {
            [rule]: options !== undefined ? ["error", ...options] : "error",
        },
    });
}
