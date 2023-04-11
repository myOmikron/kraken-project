import React from "react";

/** Configuration for defining {@link Route routes} */
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

/** Regex for a bind parameter in {@link RouteConfig.url `url`} */
const BIND_REGEX = /^\{(.*)}$/;

class Route<Params extends {}> {
    /** The route's configuration */
    readonly config: RouteConfig<Params>;

    /** Pre-split and "parsed" version of {@link RouteConfig.url `config.url`} */
    readonly pattern: Array<string | { bind: keyof Params }>;

    /** List of errors the constructor found in the config */
    readonly errors: Array<string>;

    /** Router this route is registered in */
    readonly router: Router;

    constructor(router: Router, config: RouteConfig<Params>) {
        this.router = router;
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
     * @return the constructed url
     */
    build(params: { [Param in keyof Params]: any }): string {
        return this.pattern
            .map((pattern) => {
                if (typeof pattern === "string") return pattern;
                else return String(params[pattern.bind]);
            })
            .join("/");
    }

    /**
     * Open this route in the current tab
     *
     * @param params parameters to {@link build `build`} the url with
     */
    visit(params: { [Param in keyof Params]: any }) {
        const url = this.build(params);
        window.location.hash = url;
    }

    /**
     * Open this route in a new tab
     *
     * @param params parameters to {@link build `build`} the url with
     */
    open(params: { [Param in keyof Params]: any }) {
        const url = this.build(params);
        window.open(`/#${url}`);
    }
}

export class Router {
    routes: Array<Route<{}>> = [];

    /**
     * Create a new route and add it to this router
     *
     * @param config the route's config
     * @return the new route
     */
    add<Params extends {}>(config: RouteConfig<Params>): Route<Params> {
        const route = new Route(this, config);
        this.routes.push(route as unknown as Route<{}>);
        return route;
    }

    /**
     * Finalize all routes and log any potential errors
     *
     * TODO this method could post process the list of all route and produce some kind of tree to speed up the url matching process
     */
    finish() {
        for (const route of this.routes) {
            if (route.errors.length > 0) {
                console.error(`Errors in route "${route.config.url}":`, ...route.errors);
            }
        }
    }

    /**
     * Match a given pre-split url
     *
     * @param url url already split at "/"
     * @return the matched route and its parameters, if any
     */
    match(url: Array<string>): [{}, Route<{}>] | undefined {
        // TODO this naive iter and check step by step could be improved by processing the list in `finish()`
        for (const route of this.routes) {
            const params = route.match(url);
            if (params !== undefined) return [params, route];
        }
        return undefined;
    }

    /**
     * Match a given pre-split url and render the routes element
     *
     * @param url url already split at "/"
     * @return the matched route's {@link RouteConfig.render `render`} result, if any
     */
    matchAndRender(url: Array<string>): React.ReactNode | undefined {
        const match = this.match(url);
        if (match === undefined) return undefined;
        const [params, route] = match;
        return route.config.render(params);
    }
}
