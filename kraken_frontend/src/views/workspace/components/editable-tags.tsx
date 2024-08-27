import React from "react";
import Creatable from "react-select/creatable";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { SimpleTag, TagType } from "../../../api/generated";
import { selectStyles } from "../../../components/select-menu";
import Tag from "../../../components/tag";
import WorkspaceCreateTag from "./workspace-create-tag";

export type EditableTagsProps = {
    /**
     * The workspace containing the item whose tags to edit
     *
     * It is required for managing workspace specific tags
     */
    workspace: string;

    /** List of currently set tags */
    tags: Array<SimpleTag>;

    /** Callback when the list changed */
    onChange: (tags: Array<SimpleTag>) => void;

    /** called when all tags are loaded */
    onTagsLoaded?: (tags: Array<SimpleTag>) => void;

    /** can be set to false to disallow creation, otherwise enabled */
    allowCreate?: boolean;
};

/** A multi `<Select />` for editing a list of tags */
export default function EditableTags(props: EditableTagsProps) {
    const { workspace, tags, onChange } = props;
    const allowCreate = props.allowCreate ?? true;

    // State for create new tag modal
    const [newTag, setNewTag] = React.useState<string | null>(null);

    // Load tags from backend
    const [allTags, setAllTags] = React.useState<Array<SimpleTag>>([]);
    React.useEffect(() => {
        setAllTags([]);
        Promise.all([
            Api.globalTags.all().then((v) => v.unwrap().globalTags.map((tag) => ({ ...tag, tagType: TagType.Global }))),
            Api.workspaces.tags
                .all(workspace)
                .then((v) => v.unwrap().workspaceTags.map((tag) => ({ ...tag, tagType: TagType.Workspace }))),
        ])
            .then(([global, workspace]) => {
                const all = [...global, ...workspace];
                setAllTags(all);
                props.onTagsLoaded?.(all);
            })
            .catch((e) => {
                toast.error(e.message);
            });
    }, [workspace]);

    return (
        <>
            <Creatable<SimpleTag, true>
                styles={selectStyles("default")}
                isMulti={true}
                onCreateOption={allowCreate ? setNewTag : undefined}
                value={tags}
                onChange={(tags) => onChange([...tags])}
                options={allTags}
                formatOptionLabel={(tag) =>
                    "value" in tag ? (
                        <>
                            {allowCreate ? "Create " : "Unknown tag "}
                            <Tag name={String(tag.value)} />
                        </>
                    ) : (
                        <Tag {...tag} />
                    )
                }
                getOptionLabel={({ name }) => name}
                getOptionValue={({ uuid }) => uuid}
            />
            <Popup nested modal open={newTag !== null} onClose={() => setNewTag(null)}>
                <WorkspaceCreateTag
                    initialName={newTag || ""}
                    workspace={workspace}
                    onCreated={(tag) => {
                        const simpleTag: SimpleTag = {
                            ...tag,
                            tagType: "workspace" in tag ? TagType.Workspace : TagType.Global,
                        };
                        setAllTags((tags) => [simpleTag, ...tags]);
                        setNewTag(null);
                        onChange([...tags, simpleTag]);
                    }}
                />
            </Popup>
        </>
    );
}
