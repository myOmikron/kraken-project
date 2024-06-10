import React from "react";

/**
 * This hook simply returns a function which, when called, will trigger a rerender.
 *
 * @returns function that can be called to trigger rerender
 */
export function useTriggerUpdate() {
    const [_, setDummy] = React.useState({});
    return () => setDummy({});
}

Object.defineProperty(useTriggerUpdate, "name", { value: "useTriggerUpdate" });
