import { FullGlobalTag, FullWorkspaceTag } from "../../../api/generated";
import React from "react";
import USER_CONTEXT from "../../../context/user";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";
import Checkbox from "../../../components/checkbox";
import Tag from "../../../components/tag";
import "../../../styling/workspace-create-tag.css";

export type WorkspaceCreateTagProps = {
    /** The workspace to add this tag to (if not set as global) */
    workspace: string;

    /**
     * Set an initial name for the new tag
     *
     * Everytime this value changes, it will overwrite any changes the user might have made to the name.
     */
    initialName: string;

    /** Callback after the tag has been created */
    onCreated: (tag: FullWorkspaceTag | FullGlobalTag) => void;
};

/** `<form />` for creating a new tag */
export default function WorkspaceCreateTag(props: WorkspaceCreateTagProps) {
    const { workspace, initialName, onCreated } = props;

    const {
        user: { admin },
    } = React.useContext(USER_CONTEXT);

    // State
    const [name, setName] = React.useState<string>("");
    const [isGlobal, setIsGlobal] = React.useState<boolean>(false);
    const [colorString, setColorString] = React.useState("#000000");
    const [colorAlpha, setColorAlpha] = React.useState(128);

    // Overwrite `name` with `initialName`
    React.useEffect(() => setName(initialName), [initialName]);

    // Convert `colorString` and `colorAlpha` into `color`
    const colorMatch = colorString.match(/#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i);
    const [r, g, b] = colorMatch === null ? [0, 0, 0] : colorMatch.splice(1).map((hex) => parseInt(hex, 16));
    const tag = {
        name,
        color: { r, g, b, a: colorAlpha },
    };

    return (
        <form
            className={"workspace-create-tag pane"}
            onSubmit={(event) => {
                event.preventDefault();

                if (isGlobal) {
                    Api.globalTags.create(tag).then(
                        handleApiError(({ uuid }) => {
                            toast.success("Created new global tag");
                            onCreated({ uuid, ...tag });
                        }),
                    );
                } else {
                    Api.workspaces.tags.create(workspace, tag).then(
                        handleApiError(({ uuid }) => {
                            toast.success("Created new workspace tag");
                            onCreated({ uuid, ...tag, workspace });
                        }),
                    );
                }
            }}
        >
            <h3 className={"heading workspace-create-tag-heading"}>Create new tag</h3>
            <label>
                Name: <Input value={name} onChange={setName} />
            </label>
            <label>
                Color:
                <div className={"workspace-create-tag-color-picker"}>
                    <Input className={undefined} type={"color"} value={colorString} onChange={setColorString} />
                    <Tag {...tag} />
                </div>
            </label>
            <label>
                Alpha:
                <Input
                    className={undefined}
                    type={"range"}
                    min={0}
                    max={255}
                    value={String(colorAlpha)}
                    onChange={(string) => setColorAlpha(parseInt(string))}
                />
            </label>
            {admin ? (
                <label>
                    Global: <Checkbox value={isGlobal} onChange={setIsGlobal} />
                </label>
            ) : null}
            <button className={"button workspace-create-tag-submit"} type={"submit"}>
                Create and add
            </button>
        </form>
    );
}
