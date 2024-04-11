import { SimpleFindingCategory } from "../../../api/generated";
import FindingCategory from "../../../components/finding-category";

export type CategoryClickCallback = (
    e: { ctrlKey: boolean; shiftKey: boolean; altKey: boolean },
    category: SimpleFindingCategory,
) => void;

type CategoryListProps = {
    categories: Array<SimpleFindingCategory>;
};

export default function CategoryList(props: CategoryListProps) {
    return (
        <div className="finding-category-list">
            {props.categories.map((category) => (
                <FindingCategory key={category.uuid} {...category} />
            ))}
        </div>
    );
}
