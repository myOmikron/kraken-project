import React from "react";

/**
 * Hack for exposing a checkbox in React's dev tools
 *
 * This component doesn't render anything.
 *
 * It contains one `state` variable `logging` whose value is kept in sync
 * with the implementation behind the default export `CONSOLE`.
 *
 * It is implemented as class component, because it allows toggling the `logging`
 * state with an easy checkbox in React's dev tools' Component tab.
 *
 * The `logging` state will also be saved into `localStorage`.
 */
export class LoggingSwitch extends React.PureComponent {
    static displayName = "LoggingSwitch";
    state = { logging: false };

    /** Set `CONSOLE` based on the internal state */
    setConsole() {
        if (this.state.logging && CONSOLE.__proto__ === NOOP_CONSOLE) {
            CONSOLE.__proto__ = console;
            localStorage.setItem("logging", String(this.state.logging));
        } else if (!this.state.logging && CONSOLE.__proto__ === console) {
            CONSOLE.__proto__ = NOOP_CONSOLE;
            localStorage.setItem("logging", String(this.state.logging));
        }
    }

    // eslint-disable-next-line jsdoc/require-jsdoc
    componentDidMount() {
        this.setState({ logging: localStorage.getItem("logging") === "true" });
    }

    // eslint-disable-next-line jsdoc/require-jsdoc
    componentDidUpdate() {
        this.setConsole();
    }

    // eslint-disable-next-line jsdoc/require-jsdoc
    render() {
        return null;
    }
}

/* eslint-disable jsdoc/require-jsdoc, @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any */
/** Dummy `console` whose methods do nothing */
const NOOP_CONSOLE: Console = {
    assert(condition?: boolean, ...data: any[]) {},
    clear() {},
    count(label?: string) {},
    countReset(label?: string) {},
    debug(...data: any[]) {},
    dir(item?: any, options?: any) {},
    dirxml(...data: any[]) {},
    error(...data: any[]) {},
    group(...data: any[]) {},
    groupCollapsed(...data: any[]) {},
    groupEnd() {},
    info(...data: any[]) {},
    log(...data: any[]) {},
    table(tabularData?: any, properties?: string[]) {},
    time(label?: string) {},
    timeEnd(label?: string) {},
    timeLog(label?: string, ...data: any[]) {},
    timeStamp(label?: string) {},
    trace(...data: any[]) {},
    warn(...data: any[]) {},
};
/* eslint-enable jsdoc/require-jsdoc, @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any */

const CONSOLE = { __proto__: NOOP_CONSOLE };
/** [`console`]{@link console} which can be (and defaults to being) disabled */
// @ts-ignore: `CONSOLE` has a prototype of type `Console` making it behave like one
export default CONSOLE as Console;
