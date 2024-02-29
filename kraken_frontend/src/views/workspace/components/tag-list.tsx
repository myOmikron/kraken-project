import { SimpleTag } from "../../../api/generated";
import Tag from "../../../components/tag";
import { FilterOutput } from "./filter-input";

type TagListProps = {
    tags: Array<SimpleTag>;
    globalFilter?: FilterOutput;
    filter?: FilterOutput;
};
export default function TagList(props: TagListProps) {
    return (
        <div className={"tag-list"}>
            {props.tags.map((tag) => (
                <Tag
                    key={tag.uuid}
                    {...tag}
                    onClick={
                        props.filter || props.globalFilter
                            ? (e) =>
                                  (e.ctrlKey ? props.globalFilter : props.filter)?.addColumn("tag", tag.name, e.altKey)
                            : undefined
                    }
                />
            ))}
        </div>
    );
}
