import { ROUTES } from "../../routes";
import "../../styling/list-finding-definition.css";
import { FindingSeverity, SimpleFindingDefinition } from "../../api/generated";
import React from "react";
import Input from "../../components/input";
import PlusIcon from "../../svg/plus";
import { Api } from "../../api/api";
import { handleApiError } from "../../utils/helper";

type ListFindingDefinitionProps = {};

export function ListFindingDefinition(props: ListFindingDefinitionProps) {
    const [search, setSearch] = React.useState("");
    const [defs, setDefs] = React.useState([] as Array<SimpleFindingDefinition>);
    const [hover, setHover] = React.useState(null as number | null);

    React.useEffect(() => {
        Api.knowledgeBase.findingDefinitions
            .all()
            .then(handleApiError(({ findingDefinitions }) => setDefs(findingDefinitions)));
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
                {defs.map(({ uuid, name, severity, summary }, index) => (
                    <div
                        className={"list-finding-definition-item pane"}
                        onPointerEnter={() => setHover(index)}
                        onPointerLeave={() => setHover((oldIndex) => (oldIndex === index ? null : oldIndex))}
                        {...ROUTES.FINDING_DEFINITION_EDIT.clickHandler({ uuid })}
                    >
                        <h2 className={"sub-heading"}>
                            {name} <small>{severity}</small>
                        </h2>
                    </div>
                ))}
            </div>
            {hover === null ? <div /> : <Details {...defs[hover]} />}
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
