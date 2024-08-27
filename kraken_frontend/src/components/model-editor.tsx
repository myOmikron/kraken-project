import { loader, Monaco } from "@monaco-editor/react";
import { editor } from "monaco-editor";
import React from "react";
import { toast } from "react-toastify";
import CONSOLE from "../utils/console";
import Loading from "./loading";

/** React props for [`<ModelEditor />`]{@link ModelEditor} */
export type ModelEditorProps = {
    /** The model to edit */
    model: editor.ITextModel | null;
    /** Callback which exposes the editor instance to the parent component */
    setEditor?: (editorInstance: editor.IStandaloneCodeEditor) => void;
};

/**
 * A monaco editor for an externally managed model
 *
 * Use `useModel` or `useModelStore` for managing the model.
 */
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
            .catch((error) => {
                if (error?.type !== "cancelation") {
                    CONSOLE.error("Monaco initialization: error:", error);
                    toast.error("Monaco initialization");
                }
            });

        return () => (editorRef.current ? editorRef.current.dispose() : cancelable.cancel());
    }, []);

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

    return (
        <section className={"model-editor"}>
            {(!isEditorReady || model === null) && <Loading />}
            <div
                ref={containerRef}
                style={{ display: !isEditorReady || model === null ? "none" : undefined }}
                className={"knowledge-base-editor"}
            />
        </section>
    );
}
