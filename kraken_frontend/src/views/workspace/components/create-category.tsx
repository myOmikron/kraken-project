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
    const [colorString, setColorString] = React.useState("#000000");
    const [colorAlpha, setColorAlpha] = React.useState(128);

    // Overwrite `name` with `initialName`
    React.useEffect(() => setName(initialName), [initialName]);

    // Convert `colorString` and `colorAlpha` into `color`
    const colorMatch = colorString.match(/#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i);
    const [r, g, b] = colorMatch === null ? [0, 0, 0] : colorMatch.splice(1).map((hex) => parseInt(hex, 16));
    const category = {
        name,
        color: { r, g, b, a: colorAlpha },
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
            <h2 className={"sub-heading workspace-create-tag-heading"}>Create new category</h2>
            <FindingCategory {...category} />
            <div className="workspace-create-tag-body">
                <label>
                    <span>Name:</span>
                    <Input value={name} onChange={setName} />
                </label>
                <label>
                    <span>Color:</span>
                    <Input type={"color"} value={colorString} onChange={setColorString} />
                </label>
                <label>
                    <span>Alpha:</span>
                    <Input
                        className={undefined}
                        type={"range"}
                        min={0}
                        max={255}
                        value={String(colorAlpha)}
                        onChange={(string) => setColorAlpha(parseInt(string))}
                    />
                </label>
            </div>
            <button className="button" type={"submit"} disabled={name.length === 0}>
                Create and add
            </button>
        </form>
    );
}
