import EditIcon from "../../../svg/edit";
import React, { ReactNode } from "react";
import Popup from "reactjs-popup";
import { Editor } from "@monaco-editor/react";
import "../../../styling/markdown-editor-popup.css";
import { GithubMarkdown } from "../../../components/github-markdown";
import PlusIcon from "../../../svg/plus";
import { configureMonaco } from "../../../utils/monaco";

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
                    {content.length > 0 ? ["Edit User Details", <EditIcon />] : ["Add User Details", <PlusIcon />]}
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
                        beforeMount={configureMonaco}
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
