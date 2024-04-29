import React from "react";
import { SimpleTag } from "../../../api/generated";
import Tag from "../../../components/tag";
import { UseFilterReturn } from "./filter-input";

export type TagClickCallback = (
    e: React.KeyboardEvent<HTMLDivElement> | React.MouseEvent<HTMLDivElement, MouseEvent>,
    tag: SimpleTag,
) => void;

/** React props for [`<TagList />`]{@link TagList} */
type TagListProps = {
    /**
     * List of tags to display
     */
    tags: Array<SimpleTag>;
    globalFilter?: UseFilterReturn;
    filter?: UseFilterReturn;
    /**
     * Callback when tag is clicked
     */
    onClickTag?: TagClickCallback;
};

/**
 * Component to display a list of tags in a row
 */
export default function TagList(props: TagListProps) {
    return (
        <div className={"tag-list"}>
            {props.tags.map((tag) => (
                <Tag
                    key={tag.uuid}
                    {...tag}
                    onClick={
                        props.filter || props.globalFilter || props.onClickTag
                            ? (e) => {
                                  props.onClickTag?.(e, tag);
                                  (e.ctrlKey ? props.globalFilter : props.filter)?.addColumn("tag", tag.name, e.altKey);
                              }
                            : undefined
                    }
                />
            ))}
        </div>
    );
}
