import { Editor } from "@monaco-editor/react";
import { editor } from "monaco-editor";
import React from "react";
import Popup from "reactjs-popup";
import { GithubMarkdown } from "../../../components/github-markdown";
import ModelEditor from "../../../components/model-editor";
import "../../../styling/editor-popup.css";
import { configureMonaco } from "../../../utils/monaco";

/**
 * React props for [`<EditorPopup />`]{@link EditorPopup}
 *
 * Be choosing between passing `onChange` or `model` you choose the actual editor component used internally.
 * `onChange` uses the "old" implementation from `@monaco-editor/react` while `model` uses our
 * custom implementation which supports live editing.
 */
export type EditorPopupProps = {
    /** Html element to trigger the popup */
    trigger: React.JSX.Element;

    /** The editor's value */
    value: string;

    /**
     * React node which renders `value` in a preview
     *
     * (Defaults to `<GithubMarkdown>{value}</GithubMarkdown>`)
     */
    preview?: React.ReactNode;

    /** Main heading for the popup */
    heading: React.ReactNode;

    /** Optional sub heading for the popup providing some more detail */
    subHeading?: React.ReactNode;

    /** Callback when the popup is opened */
    onOpen?: () => void;
} & (
    | {
          /** Callback invoked by the editor to change its value */
          onChange: (newValue: string) => void;

          /**
           * The language the editor should provide syntax highlighting for
           *
           * (Defaults to `"markdown"`)
           */
          language?: string;
      }
    | {
          /** Monaco model to use for the editor */
          model: editor.ITextModel | null;
      }
);

/** A [`<Popup />`]{@link EditorPopup} which opens an editor */
export default function EditorPopup(props: EditorPopupProps) {
    const { trigger, value, preview, heading, subHeading, onOpen } = props;

    return (
        <Popup className="editor-popup" trigger={trigger} nested modal on={"click"} onOpen={onOpen}>
            <div className="pane">
                <div className="label">
                    <h1 className="sub-heading">{heading}</h1>
                    {subHeading && <h3 className="sub-heading">{subHeading}</h3>}
                </div>
                <div className="grid">
                    {preview !== undefined ? preview : <GithubMarkdown>{value}</GithubMarkdown>}
                    {"onChange" in props ? (
                        <Editor
                            className={"knowledge-base-editor"}
                            theme={"kraken"}
                            beforeMount={configureMonaco}
                            language={props.language ?? "markdown"}
                            value={value}
                            onChange={(value) => {
                                if (value !== undefined) props.onChange(value);
                            }}
                        />
                    ) : (
                        <ModelEditor model={props.model} />
                    )}
                </div>
            </div>
        </Popup>
    );
}
