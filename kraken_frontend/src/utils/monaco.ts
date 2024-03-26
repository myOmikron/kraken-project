import { loader, Monaco } from "@monaco-editor/react";
import React from "react";

export const MONACO_PROMISE = loader.init().then(configureMonaco);

export let MONACO: Monaco | null = null;
MONACO_PROMISE.then((monaco) => {
    MONACO = monaco;
});

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

export function useMonaco(): Monaco | null {
    const [monaco, setMonaco] = React.useState(MONACO);
    React.useEffect(() => {
        if (monaco === null)
            MONACO_PROMISE.then(setMonaco).catch((error) => console.error("Monaco initialization: ", error));
    });
    return monaco;
}
