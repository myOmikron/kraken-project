import { Editor } from "@monaco-editor/react";
import { ReactNode } from "react";
import Popup from "reactjs-popup";
import { GithubMarkdown } from "../../../components/github-markdown";
import "../../../styling/markdown-editor-popup.css";
import EditIcon from "../../../svg/edit";
import PlusIcon from "../../../svg/plus";
import { configureMonaco } from "../../../utils/monaco";

/** React props for {@link MarkdownEditorPopup `<MarkdownEditorPopup />`} */
type MarkdownEditorPopupProps = {
    /** Element the details are for */
    label: ReactNode;
    /** The content */
    content: string;
    /** Callback when the content in the editor changed */
    onChange: (content: string) => void;
};

/**
 * <Popup /> with Markdown Editor to add Details to an Affected Data when editing or creating a finding
 */
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
                        theme={"kraken"}
                        beforeMount={configureMonaco}
                        language={"markdown"}
                        value={content}
                        onChange={(value) => {
                            if (value !== undefined) onChange(value);
                        }}
                    />
                </div>
            </div>
        </Popup>
    );
}
