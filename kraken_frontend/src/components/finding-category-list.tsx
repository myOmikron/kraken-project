import { SimpleFindingCategory } from "../api/generated";
import FindingCategory from "./finding-category";

/** React props for [`<FindingCategoryList />`]{@link FindingCategoryList} */
type FindingCategoryListProps = {
    /** The list of categories to display */
    categories: SimpleFindingCategory[];
};

/** A list of [`<FindingCategory />`]{@link FindingCategory} */
export default function FindingCategoryList(props: FindingCategoryListProps) {
    return (
        <div className="finding-category-list">
            {props.categories.map((c) => (
                <FindingCategory name={c.name} />
            ))}
        </div>
    );
}
