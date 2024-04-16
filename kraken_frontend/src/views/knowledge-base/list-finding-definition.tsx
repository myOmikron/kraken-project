import React from "react";
import { Api } from "../../api/api";
import { SimpleFindingDefinition } from "../../api/generated";
import FindingCategoryList from "../../components/finding-category-list";
import Input from "../../components/input";
import { ROUTES } from "../../routes";
import "../../styling/knowledge-base.css";
import "../../styling/list-finding-definition.css";
import PlusIcon from "../../svg/plus";
import { handleApiError } from "../../utils/helper";

/** React props for {@link ListFindingDefinition `<ListFindingDefinition />`} */
type ListFindingDefinitionProps = {};

/**
 * View for listing all finding definitions
 *
 * It also links to the edit view for a single definition as well as the view to create a new one.
 *
 * It is routed under {@link ROUTES.FINDING_DEFINITION_LIST `ROUTES.FINDING_DEFINITION_LIST`}.
 */
export function ListFindingDefinition(props: ListFindingDefinitionProps) {
    const [search, setSearch] = React.useState("");
    const [defs, setDefs] = React.useState([] as Array<SimpleFindingDefinition>);
    const [hover, setHover] = React.useState<SimpleFindingDefinition>();

    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions
            .all()
            .then(
                handleApiError(({ findingDefinitions }) =>
                    setDefs(findingDefinitions.sort(({ name: a }, { name: b }) => a.localeCompare(b))),
                ),
            );
    }, []);

    return (
        <div className={"list-finding-definition-container"}>
            <div className={"list-finding-definition-heading pane"}>
                <h1 className={"heading"}>Finding Definitions</h1>
            </div>
            <div className={"list-finding-definition-searchbar pane"}>
                <Input placeholder={"Search definition..."} value={search} onChange={setSearch} />
                <button className={"button"} {...ROUTES.FINDING_DEFINITION_CREATE.clickHandler({})}>
                    <PlusIcon />
                </button>
            </div>
            <div className={"list-finding-definition-list"}>
                {defs
                    .filter(({ name }) => name.includes(search))
                    .map((def) => (
                        <div
                            className={"list-finding-definition-item pane"}
                            onPointerEnter={() => setHover(def)}
                            onPointerLeave={() => setHover(undefined)}
                            {...ROUTES.FINDING_DEFINITION_EDIT.clickHandler({ uuid: def.uuid })}
                        >
                            <h2 className={"sub-heading"}>{def.name}</h2>
                            <FindingCategoryList categories={def.categories} />
                            <div className="sub-heading">{def.severity}</div>
                        </div>
                    ))}
            </div>
            {hover === undefined ? (
                <div />
            ) : (
                <div className={"list-finding-definition-details pane"}>
                    <h1 className={"sub-heading"}>
                        {hover.name} <small>{hover.severity}</small>
                    </h1>
                    <FindingCategoryList categories={hover.categories} />
                    <p>{hover.summary}</p>
                </div>
            )}
        </div>
    );
}
