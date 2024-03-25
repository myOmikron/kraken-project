import { editor } from "monaco-editor";
import React from "react";
import { loader, Monaco } from "@monaco-editor/react";
import Loading from "./loading";

export type ModelEditorProps = {
    model: editor.ITextModel | null;
    setEditor?: (editorInstance: editor.IStandaloneCodeEditor) => void;
};

export default function ModelEditor(props: ModelEditorProps) {
    const { model, setEditor } = props;
    const [_, setIsMonacoMounting] = React.useState(true);
    const [isEditorReady, setIsEditorReady] = React.useState(false);
    const monacoRef = React.useRef<Monaco | null>(null);
    const editorRef = React.useRef<editor.IStandaloneCodeEditor | null>(null);
    const containerRef = React.useRef<HTMLDivElement>(null);

    React.useEffect(() => {
        const cancelable = loader.init();

        cancelable
            .then((monaco) => (monacoRef.current = monaco) && setIsMonacoMounting(false))
            .catch((error) => error?.type !== "cancelation" && console.error("Monaco initialization: error:", error));

        return () => (editorRef.current ? editorRef.current.dispose() : cancelable.cancel());
    }, []);

    /*
    useUpdate(
        () => {
            if (!editorRef.current || value === undefined) return;
            if (editorRef.current.getOption(monacoRef.current!.editor.EditorOption.readOnly)) {
                editorRef.current.setValue(value);
            } else if (value !== editorRef.current.getValue()) {
                preventTriggerChangeEvent.current = true;
                editorRef.current.executeEdits("", [
                    {
                        range: editorRef.current.getModel()!.getFullModelRange(),
                        text: value,
                        forceMoveMarkers: true,
                    },
                ]);

                editorRef.current.pushUndoStop();
                preventTriggerChangeEvent.current = false;
            }
        },
        [value],
        isEditorReady,
    );
    */

    React.useEffect(() => {
        if (!containerRef.current || !monacoRef.current) return;
        if (!editorRef.current) {
            editorRef.current = monacoRef.current.editor.create(containerRef.current, {
                model,
                automaticLayout: true,
                theme: "kraken",
            });
            if (setEditor) {
                setEditor(editorRef.current);
            }

            setIsEditorReady(true);
        }
    }, [containerRef.current, monacoRef.current, editorRef.current]);

    React.useEffect(() => {
        if (editorRef.current) {
            editorRef.current?.setModel(model);
        }
    }, [model]);

    /*
    // onChange
    useEffect(() => {
        if (isEditorReady && onChange) {
            subscriptionRef.current?.dispose();
            subscriptionRef.current = editorRef.current?.onDidChangeModelContent((event) => {
                if (!preventTriggerChangeEvent.current) {
                    onChange(editorRef.current!.getValue(), event);
                }
            });
        }
    }, [isEditorReady, onChange]);
    */

    return (
        <section style={{ display: "flex", position: "relative", textAlign: "initial", width: "100%", height: "100%" }}>
            {(!isEditorReady || model === null) && <Loading />}
            <div
                ref={containerRef}
                style={{ width: "100%", display: !isEditorReady || model === null ? "none" : undefined }}
                className={"knowledge-base-editor"}
            />
        </section>
    );
}
