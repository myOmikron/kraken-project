import { loader, Monaco } from "@monaco-editor/react";
import React from "react";

/** Global promise resolving to monaco */
export const MONACO_PROMISE = loader.init().then(configureMonaco);

/** The monaco instance {@link MONACO_PROMISE} resolved to */
export let MONACO: Monaco | null = null;
MONACO_PROMISE.then((monaco) => {
    MONACO = monaco;
});

/**
 * Configures monaco
 *
 * Currently, this just adds the `"kraken"` theme.
 *
 * This method should be an implementation detail of the [`MONACO_PROMISE`]{@link MONACO_PROMISE}.
 *
 * But it is exposed for usage in  `<Editor />`
 * until it is completely replaced with `<ModelEditor />`
 *
 * @param monaco the monaco instance to configure
 * @returns the configured monaco instance
 */
export function configureMonaco(monaco: Monaco): Monaco {
    monaco.editor.defineTheme("kraken", {
        base: "vs-dark",
        inherit: true,
        rules: [],
        colors: {
            "editor.foreground": "#C5C8C6",
            "editor.background": "#00ccff10",
            "editor.selectionBackground": "#33ccff40",
            "editor.editorLineNumber": "#ffffff",
            "editorCursor.foreground": "#AEAFAD",
        },
    });
    return monaco;
}

/**
 * Hook waiting for {@link MONACO_PROMISE} to resolve
 *
 * @returns the monaco instance once its loaded
 */
export function useMonaco(): Monaco | null {
    const [monaco, setMonaco] = React.useState(MONACO);
    React.useEffect(() => {
        if (monaco === null)
            MONACO_PROMISE.then(setMonaco).catch((error) => console.error("Monaco initialization: ", error));
    });
    return monaco;
}
