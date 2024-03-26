// @ts-check

import eslint from "@eslint/js";
import tsEslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

const config = tsEslint.config(
    eslint.configs.recommended,
    ...tsEslint.configs.recommended,
    // @ts-ignore: plugin hasn't updated to new API, but it still works
    // { plugins: { "react-hooks": reactHooks }, rules: reactHooks.configs.recommended.rules },
    {
        ignores: ["src/api/generated/**"],
    },
    {
        rules: {
            "no-case-declarations": "off", // potential errors already caught by typescript

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

/*
module.exports = {
    ignorePatterns: ["src/api/generated"],
    env: { browser: true, es2020: true },
    extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended", "plugin:react-hooks/recommended"],
    parser: "@typescript-eslint/parser",
    parserOptions: { ecmaVersion: "latest", sourceType: "module" },
    plugins: ["react-refresh"],
    rules: {
        "react-refresh/only-export-components": "warn",

        "no-unused-vars": "off",
    },
};
*/

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
