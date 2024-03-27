import React from "react";
import { Api } from "../../api/api";
import { SimpleFindingDefinition } from "../../api/generated";
import Input from "../../components/input";
import { ROUTES } from "../../routes";
import "../../styling/knowledge-base.css";
import "../../styling/list-finding-definition.css";
import PlusIcon from "../../svg/plus";
import { handleApiError } from "../../utils/helper";

type ListFindingDefinitionProps = {};

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
                            <h2 className={"sub-heading"}>
                                {def.name} <small>{def.severity}</small>
                            </h2>
                        </div>
                    ))}
            </div>
            {hover === undefined ? <div /> : <Details {...hover} />}
        </div>
    );
}

export function Details(props: SimpleFindingDefinition) {
    const { name, severity, summary } = props;
    return (
        <div className={"pane"}>
            <h1 className={"sub-heading"}>
                {name} <small>{severity}</small>
            </h1>
            <p>{summary}</p>
        </div>
    );
}
