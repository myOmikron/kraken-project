import { SimpleTag } from "../../../api/generated";
import Tag from "../../../components/tag";

type TagListProps = {
    tags: Array<SimpleTag>;
    onClick?: (tag: SimpleTag, negate: boolean) => any;
    onCtrlClick?: (tag: SimpleTag, negate: boolean) => any;
};
export default function TagList(props: TagListProps) {
    return (
        <div className={"tag-list"}>
            {props.tags.map((tag) => (
                <Tag
                    key={tag.uuid}
                    {...tag}
                    onClick={
                        props.onClick || props.onCtrlClick
                            ? (e) => (e.ctrlKey ? props.onCtrlClick?.(tag, e.altKey) : props.onClick?.(tag, e.altKey))
                            : undefined
                    }
                />
            ))}
        </div>
    );
}
