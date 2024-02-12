import React, { useState } from "react";
import "../styling/knowledge-base.css";
import { ROUTES } from "../routes";
import { Editor, Monaco } from "@monaco-editor/react";

type KnowledgeBaseProps = {};

export default function KnowledgeBase(props: KnowledgeBaseProps) {
    const [userInput, setUserInput] = useState("# heading");

    return (
        <div className={"knowledge-base-container"}>
            <div className={"pane"}>
                <h2 className={"sub-heading"}>Knowledge Base</h2>
            </div>
            <div className={"pane knowledge-base-content"}>
                <div className={"knowledge-base-mask"}></div>
                <div className={"knowledge-base-eyes"}></div>
                <div>
                    <Editor
                        theme={"custom"}
                        value={userInput}
                        language={"markdown"}
                        onChange={(value) => {
                            setUserInput(value !== undefined ? value : "");
                        }}
                        className={"knowledge-base-editor"}
                        beforeMount={setupMonaco}
                    />
                </div>
                <div>
                    <button className={"button"} {...ROUTES.FINDING_DEFINITION_LIST.clickHandler({})}>
                        Finding Definitions
                    </button>
                </div>
            </div>
        </div>
    );
}

let setup = false;
export function setupMonaco(monaco: Monaco) {
    if (setup) return;
    setup = true;

    monaco.editor.defineTheme("custom", {
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
}
