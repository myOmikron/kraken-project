import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import { SimpleFindingCategory } from "../../../api/generated";
import FindingCategory from "../../../components/finding-category";
import Input from "../../../components/input";
import "../../../styling/create-category.css";
import { handleApiError } from "../../../utils/helper";

export type CreateCategoryProps = {
    /**
     * Set an initial name for the new category
     *
     * Everytime this value changes, it will overwrite any changes the user might have made to the name.
     */
    initialName: string;

    /** Callback after the category has been created */
    onCreated: (category: SimpleFindingCategory) => void;
};

/** `<form />` for creating a new category */
export default function CreateCategory(props: CreateCategoryProps) {
    const { initialName, onCreated } = props;

    // State
    const [name, setName] = React.useState<string>("");

    // Overwrite `name` with `initialName`
    React.useEffect(() => setName(initialName), [initialName]);

    const category = {
        name,
    };

    return (
        <form
            className={"workspace-create-category pane"}
            onSubmit={(event) => {
                event.preventDefault();

                Api.findingCategories.admin.create(category).then(
                    handleApiError(({ uuid }) => {
                        toast.success("Created new category");
                        onCreated({ uuid, ...category });
                    }),
                );
            }}
        >
            <h2>Create new category</h2>
            <FindingCategory {...category} />
            <label>
                <span>Name:</span>
                <Input value={name} onChange={setName} />
            </label>
            <button className="button" type={"submit"} disabled={name.length === 0}>
                Create and add
            </button>
        </form>
    );
}
