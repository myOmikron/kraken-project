import { ROUTES } from "../../routes";
import "../../styling/list-finding-definition.css";
import { FindingSeverity, SimpleFindingDefinition } from "../../api/generated";
import React from "react";
import Input from "../../components/input";
import PlusIcon from "../../svg/plus";

type ListFindingDefinitionProps = {};

export function ListFindingDefinition(props: ListFindingDefinitionProps) {
    const [search, setSearch] = React.useState("");
    const [defs, setDefs] = React.useState([] as Array<SimpleFindingDefinition>);
    const [hover, setHover] = React.useState(null as number | null);

    React.useEffect(() => {
        setDefs([
            {
                uuid: "00000000-0000-0000-0000-000000000000",
                name: "Cyber Alarm 1337",
                severity: FindingSeverity.Critical,
                summary: SUMMARYS[0],
                createdAt: new Date(),
            },
            {
                uuid: "00000000-0000-0000-0000-000000000001",
                name: "Hier könnte ihre Werbung stehen",
                severity: FindingSeverity.Medium,
                summary: SUMMARYS[1],
                createdAt: new Date(),
            },
            {
                uuid: "00000000-0000-0000-0000-000000000002",
                name: "Nich so legger",
                severity: FindingSeverity.High,
                summary: SUMMARYS[2],
                createdAt: new Date(),
            },
            {
                uuid: "00000000-0000-0000-0000-000000000003",
                name: "Meh",
                severity: FindingSeverity.Low,
                summary: SUMMARYS[3],
                createdAt: new Date(),
            },
            {
                uuid: "00000000-0000-0000-0000-000000000004",
                name: "Uff wer kann sich denn dauernd scheiße ausdenken?",
                severity: FindingSeverity.Low,
                summary: SUMMARYS[4],
                createdAt: new Date(),
            },
        ]);
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

const SUMMARYS = [
    "Aspernatur aliquam id reiciendis. Magnam autem quod amet soluta ut cupiditate. Itaque expedita corporis et laborum ea dolorum. Earum reprehenderit et dolores possimus provident totam cumque sequi. Mollitia qui inventore sequi.",
    "Sapiente id molestias doloremque odit numquam ut. Soluta quam asperiores vero atque praesentium rem. Quis eos ut non non qui eum. Sit quos laborum ratione magni. Et aut qui nostrum amet dolor. Repellat illum esse aut ad.",
    "Rerum ea quod vero maiores. Ea quisquam est tempora natus voluptatem eum voluptatem id. Et corporis nostrum fugiat dolor natus maxime ipsum accusamus. Ipsa hic vel ratione. Dolorem quia culpa repellat tempore. Cupiditate voluptatem inventore accusamus.",
    "Dolorum quia possimus et omnis nisi. Laborum quia voluptatem explicabo sit debitis dolores nulla. Qui quidem odit incidunt eos quis fugiat qui ut.",
    "Necessitatibus accusamus enim ducimus nostrum vero. Ea earum et dolore ut ad. Est quia voluptatem harum provident libero. Quas officia quos ea sint accusamus. Non sed dolores libero eum accusamus qui quia vel. Accusantium labore sapiente est quis.",
];
