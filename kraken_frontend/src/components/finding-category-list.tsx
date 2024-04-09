import { SimpleFindingCategory } from "../api/generated";
import FindingCategory from "./finding-category";

type FindingCategoryListProps = {
    categories: SimpleFindingCategory[];
};

export default function FindingCategoryList(props: FindingCategoryListProps) {
    return (
        <div className="finding-category-list">
            {props.categories.map((c) => (
                <FindingCategory name={c.name} />
            ))}
        </div>
    );
}
