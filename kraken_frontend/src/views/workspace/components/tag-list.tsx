import { SimpleTag } from "../../../api/generated";
import Tag from "../../../components/tag";
import React from "react";

type TagListProps = {
    tags: Array<SimpleTag>;
};
export default function TagList(props: TagListProps) {
    return (
        <div className={"tag-list"}>
            {props.tags.map((tag) => (
                <Tag key={tag.uuid} {...tag} />
            ))}
        </div>
    );
}
