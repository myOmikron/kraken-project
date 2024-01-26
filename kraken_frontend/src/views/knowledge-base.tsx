import React, { useState } from "react";
import "../styling/knowledge-base.css";
import { Remark } from "react-remark";
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
                        beforeMount={setupThemes}
                    />
                </div>
                <div>
                    <Remark
                        remarkPlugins={[]}
                        rehypePlugins={[]}
                        onError={(err) => {
                            console.error(err);
                        }}
                    >
                        {userInput}
                    </Remark>
                </div>
            </div>
        </div>
    );
}

function setupThemes(monaco: Monaco) {
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
