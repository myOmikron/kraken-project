import EditIcon from "../../../svg/edit";
import React, { ReactNode } from "react";
import Popup from "reactjs-popup";
import { Editor } from "@monaco-editor/react";
import { setupMonaco } from "../../knowledge-base";
import "../../../styling/markdown-editor-popup.css";
import { GithubMarkdown } from "../../../components/github-markdown";

type MarkdownEditorPopupProps = {
    label: ReactNode;
    content: string;
    onChange: (content: string) => void;
};
export default function MarkdownEditorPopup(props: MarkdownEditorPopupProps) {
    const { label, content, onChange } = props;

    return (
        <Popup
            className="markdown-editor-popup"
            trigger={
                <div className="details">
                    Edit Details
                    <EditIcon />
                </div>
            }
            nested
            modal
            on={"click"}
        >
            <div className="pane">
                <div className="label">
                    <h1 className="sub-heading">Details</h1>
                    <h3 className="sub-heading">{label}</h3>
                </div>
                <div className="grid">
                    <GithubMarkdown>{content}</GithubMarkdown>
                    <Editor
                        className={"knowledge-base-editor"}
                        theme={"custom"}
                        beforeMount={setupMonaco}
                        language={"markdown"}
                        value={content}
                        onChange={(value, event) => {
                            if (value !== undefined) onChange(value);
                        }}
                    />
                </div>
            </div>
        </Popup>
    );
}
