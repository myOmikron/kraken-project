import React from "react";
import Home from "../views/home";

/** Configuration for defining {@link Route routes} */
/**
 * @member url
 * @member parser
 * @member render
 */
export interface RouteConfig<Params extends {}> {
    /**
     * The route's url as string
     *
     * Use `{<param-name>}` to bind a part of the url to a parameter.
     * For example `user/{username}` binds to the parameter names "username".
     *
     * **Note:**
     * Binding parts of the url is only supported for a whole "directory" in the path.
     * For example `user-{username}` is not supported
     */
    url: string;

    /**
     * Set of functions to parse parameters
     *
     * When a parse function receives invalid input,
     * it should throw an error instead of returning `null` or `undefined`.
     */
    parser: { [Param in keyof Params]: (param: string) => Params[Param] };

    /**
     * Take a set of bound parameters and return the corresponding react element to render
     * @param params values parse from an url
     */
    render: (params: Params) => React.ReactNode;
}

/** Regex for a bind parameter in {@link RouteConfig.url} */
const BIND_REGEX = /^\{(.*)}$/;

class Route<Params extends {}> {
    /** The route's configuration */
    readonly config: RouteConfig<Params>;

    /** Pre-split and "parsed" version of {@link RouteConfig.url `config.url`} */
    readonly pattern: Array<string | { bind: keyof Params }>;

    /** List of errors the constructor found in the config */
    readonly errors: Array<string>;

    constructor(config: RouteConfig<Params>) {
        this.config = config;
        if (config.url.length === 0) this.pattern = [];
        else
            this.pattern = config.url.split("/").map((fragment) => {
                let match = fragment.match(BIND_REGEX);
                return match === null ? fragment : { bind: match[1] as keyof Params };
            });
        this.errors = [];

        const occurrence: Set<keyof Params> = new Set();
        for (const pattern of this.pattern) {
            if (typeof pattern === "string") continue;

            if (occurrence.has(pattern.bind)) {
                this.errors.push(`The parameter '${String(pattern.bind)}' appears multiple times in the url pattern`);
            } else {
                occurrence.add(pattern.bind);
            }

            if (this.config.parser[pattern.bind] === undefined) {
                this.errors.push(`The parameter '${String(pattern.bind)}' doesn't have a parser`);
            }
        }

        for (const param of Object.keys(config.parser)) {
            if (!occurrence.has(param as keyof Params)) {
                this.errors.push(`The parameter '${String(param)}' does not appear in the url`);
            }
        }
    }

    /**
     * Try to match an url to this route and parse its parameters
     *
     * @param url an url string which has been split at `/`
     */
    match(url: Array<string>): { [Param in keyof Params]: Params[Param] } | undefined {
        if (url.length !== this.pattern.length) return;

        const params: { [Param in keyof Params]?: Params[Param] } = {};
        for (const i in url) {
            const input = url[i];
            const pattern = this.pattern[i];

            if (typeof pattern !== "string") {
                const parser = this.config.parser[pattern.bind];
                try {
                    params[pattern.bind] = parser(input);
                } catch {
                    return;
                }
            } else if (pattern !== input) {
                return;
            }
        }

        // @ts-ignore
        return params;
    }

    /**
     * Build an url to this route using concrete parameters
     *
     * @param params parameters to use
     */
    build(params: { [Param in keyof Params]: any }): string {
        return this.pattern
            .map((pattern) => {
                if (typeof pattern === "string") return pattern;
                else return String(params[pattern.bind]);
            })
            .join("/");
    }
}

/** Set of all routes in kraken frontend */
export const ROUTES = {
    HOME: new Route({ url: "", parser: {}, render: () => <Home /> }),
};

// Log any errors from the route creation process
for (const route of Object.values(ROUTES)) {
    if (route.errors.length > 0) {
        console.error(`Errors in route "${route.config.url}":`, ...route.errors);
    }
}
