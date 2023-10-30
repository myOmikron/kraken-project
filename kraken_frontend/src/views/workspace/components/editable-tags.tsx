import { SimpleTag, TagType } from "../../../api/generated";
import React from "react";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import Creatable from "react-select/creatable";
import { selectStyles } from "../../../components/select-menu";
import Tag from "../../../components/tag";
import Popup from "reactjs-popup";
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
};

/** A multi `<Select />` for editing a list of tags */
export default function EditableTags(props: EditableTagsProps) {
    const { workspace, tags, onChange } = props;

    // State for create new tag modal
    const [newTag, setNewTag] = React.useState<string | null>(null);

    // Load tags from backend
    const [allTags, setAllTags] = React.useState<Array<SimpleTag>>([]);
    React.useEffect(() => {
        setAllTags([]);
        Api.globalTags
            .all()
            .then(
                handleApiError(({ globalTags }) =>
                    setAllTags((workspaceTags) => [
                        ...workspaceTags,
                        ...globalTags.map((tag) => ({ ...tag, tagType: TagType.Global })),
                    ]),
                ),
            );
        Api.workspaces.tags
            .all(workspace)
            .then(
                handleApiError(({ workspaceTags }) =>
                    setAllTags((globalTags) => [
                        ...workspaceTags.map((tag) => ({ ...tag, tagType: TagType.Workspace })),
                        ...globalTags,
                    ]),
                ),
            );
    }, [workspace]);

    return (
        <>
            <Creatable<SimpleTag, true>
                styles={selectStyles("default")}
                isMulti={true}
                onCreateOption={setNewTag}
                value={tags}
                onChange={(tags) => onChange([...tags])}
                options={allTags}
                formatOptionLabel={(tag) =>
                    "value" in tag ? (
                        <>
                            Create <Tag name={String(tag.value)} />
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
