import React from "react";
import { Api, UUID } from "../../../api/api";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import Input from "../../../components/input";
import { toast } from "react-toastify";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-bsd.css";
import Select from "react-select";
import SelectMenu from "../../../components/select-menu";

type WorkspaceAttacksBruteforceSubdomainsProps = {
    workspaceUuid: UUID;
};
type WorkspaceAttacksBruteforceSubdomainsState = {
    domain: string;
    taskLimit: number;
    wordlist: null | WordlistEntry;

    wordlists: Array<WordlistEntry>;

    showAdvanced: boolean;
};

type WordlistEntry = { value: string; label: string };

export default class WorkspaceAttacksBruteforceSubdomains extends React.Component<
    WorkspaceAttacksBruteforceSubdomainsProps,
    WorkspaceAttacksBruteforceSubdomainsState
> {
    constructor(props: WorkspaceAttacksBruteforceSubdomainsProps) {
        super(props);

        this.state = {
            showAdvanced: false,
            domain: "",
            taskLimit: 1000,
            wordlists: [],
            wordlist: null,
        };
    }

    componentDidMount() {
        this.retrieveWordlists().then();
    }

    async retrieveWordlists() {
        const wordlists = [{ label: "Test", value: "" }];

        this.setState({ wordlists });
    }

    async startAttack() {
        if (this.state.domain === "") {
            toast.error("");
        }

        await Api.attacks.bruteforceSubdomains({
            workspaceUuid: this.props.workspaceUuid,
            domain: this.state.domain,
            concurrentLimit: this.state.taskLimit,
            wordlistPath: "",
        });
    }

    render() {
        return (
            <div className={"workspace-attacks-bsd-ct"}>
                <div className={"workspace-attacks-bsd"}>
                    <label htmlFor={"domain"}>Domain</label>
                    <Input id={"domain"} value={this.state.domain} onChange={(v) => this.setState({ domain: v })} />
                    <label htmlFor={"wordlist"}>Wordlist</label>
                    <SelectMenu
                        id={"wordlist"}
                        options={this.state.wordlists}
                        theme={"default"}
                        value={this.state.wordlist}
                        onChange={(wordlist) => {
                            this.setState({ wordlist });
                        }}
                    />
                    <span
                        className={"neon workspace-attacks-bsd-advanced-button"}
                        onClick={() => {
                            this.setState({ showAdvanced: !this.state.showAdvanced });
                        }}
                    >
                        Advanced
                        {this.state.showAdvanced ? <CollapseIcon /> : <ExpandIcon />}
                    </span>
                    <div
                        className={
                            this.state.showAdvanced
                                ? "workspace-attacks-bsd-advanced workspace-attacks-bsd-advanced-open"
                                : "workspace-attacks-bsd-advanced"
                        }
                    >
                        <label htmlFor={"task-limit"}>Task limit</label>
                        <Input
                            id={"task-limit"}
                            placeholder={"task limit"}
                            value={this.state.taskLimit.toString()}
                            onChange={(taskLimit) => {
                                const n = Number(taskLimit);
                                if (n === null || !Number.isSafeInteger(n) || n <= 0) {
                                    return;
                                }

                                this.setState({ taskLimit: n });
                            }}
                        />
                    </div>
                </div>
                <StartAttack active={this.state.domain !== ""} onClick={() => this.startAttack().then()} />
            </div>
        );
    }
}
