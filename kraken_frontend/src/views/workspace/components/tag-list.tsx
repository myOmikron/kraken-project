import React from "react";
import { SimpleTag } from "../../../api/generated";
import Tag from "../../../components/tag";
import { UseFilterReturn } from "./filter-input";

export type TagClickCallback = (
    event: React.MouseEvent<HTMLDivElement> | React.KeyboardEvent<HTMLDivElement>,
    tag: SimpleTag,
) => void;

type TagListProps = {
    tags: Array<SimpleTag>;
    globalFilter?: UseFilterReturn;
    filter?: UseFilterReturn;
    onClickTag?: TagClickCallback;
};
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
