import React from "react";
import StartAttack from "../components/start-attack";
import Select from "react-select";
import Input from "../../../components/input";
import "../../../styling/workspace-attacks-dehashed.css";
import { Api, UUID } from "../../../api/api";
import { toast } from "react-toastify";
import { Query } from "../../../api/generated";
import SelectMenu from "../../../components/select-menu";

export type DehashedQueryType =
    | "email"
    | "domain"
    | "vin"
    | "username"
    | "password"
    | "hashed_password"
    | "address"
    | "phone"
    | "name"
    | "ip_address";

type SelectValue = {
    label: string;
    value: DehashedQueryType;
};

const DEHASHED_SEARCH_TYPES: Array<SelectValue> = [
    { label: "Domain", value: "domain" },
    { label: "Email", value: "email" },
    { label: "Name", value: "name" },
    { label: "Username", value: "username" },
    { label: "Password", value: "password" },
    { label: "Hashed password", value: "hashed_password" },
    { label: "Address", value: "address" },
    { label: "Phone", value: "phone" },
    { label: "IP Address", value: "ip_address" },
    { label: "Vin", value: "vin" },
];

type WorkspaceAttacksDehashedProps = {
    workspaceUuid: UUID;
};
type WorkspaceAttacksDehashedState = {
    type: null | SelectValue;
    search: string;
};

export default class WorkspaceAttacksDehashed extends React.Component<
    WorkspaceAttacksDehashedProps,
    WorkspaceAttacksDehashedState
> {
    constructor(props: WorkspaceAttacksDehashedProps) {
        super(props);

        this.state = {
            search: "",
            type: null,
        };
    }

    async startAttack() {
        if (this.state.search === "" || this.state.type === null) {
            toast.error("Search and type necessary to start an attack");
            return;
        }

        let query;
        switch (this.state.type.value) {
            case "email":
                query = { email: { simple: this.state.search } };
                break;
            case "domain":
                query = { domain: { simple: this.state.search } };
                break;
            case "vin":
                query = { vin: { simple: this.state.search } };
                break;
            case "username":
                query = { username: { simple: this.state.search } };
                break;
            case "password":
                query = { password: { simple: this.state.search } };
                break;
            case "hashed_password":
                query = { hashedPassword: { simple: this.state.search } };
                break;
            case "address":
                query = { address: { simple: this.state.search } };
                break;
            case "phone":
                query = { phone: { simple: this.state.search } };
                break;
            case "name":
                query = { name: { simple: this.state.search } };
                break;
            case "ip_address":
                query = { ipAddress: { simple: this.state.search } };
                break;
            default:
                toast.error("Encountered unknown type");
                return;
        }

        (await Api.attacks.queryDehashed(this.props.workspaceUuid, query)).match(
            (uuid) => toast.success("Attack started"),
            (err) => toast.error(err.message)
        );
    }

    render() {
        return (
            <div className={"workspace-attacks-dehashed-container"}>
                <div className={"workspace-attacks-dehashed"}>
                    <SelectMenu
                        options={DEHASHED_SEARCH_TYPES}
                        theme={"default"}
                        value={this.state.type}
                        onChange={(type) => {
                            this.setState({ type });
                        }}
                    />
                    <Input
                        placeholder={"dehashed query"}
                        value={this.state.search}
                        onChange={(search) => {
                            this.setState({ search });
                        }}
                    />
                </div>
                <StartAttack
                    active={this.state.search !== "" && this.state.type !== null}
                    onClick={() => {
                        this.startAttack().then();
                    }}
                />
            </div>
        );
    }
}
