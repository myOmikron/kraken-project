import React from "react";
import { Api, UUID } from "../../../api/api";
import CollapseIcon from "../../../svg/collapse";
import ExpandIcon from "../../../svg/expand";
import Input from "../../../components/input";
import { toast } from "react-toastify";
import StartAttack from "../components/start-attack";
import "../../../styling/workspace-attacks-bsd.css";
import SelectMenu from "../../../components/select-menu";
import { WORKSPACE_CONTEXT } from "../workspace";
import { PrefilledAttackParams } from "../workspace-attacks";
import { handleApiError } from "../../../utils/helper";

type WorkspaceAttacksBruteforceSubdomainsProps = { prefilled: PrefilledAttackParams };
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
    static contextType = WORKSPACE_CONTEXT;
    declare context: React.ContextType<typeof WORKSPACE_CONTEXT>;

    constructor(props: WorkspaceAttacksBruteforceSubdomainsProps) {
        super(props);

        this.state = {
            showAdvanced: false,
            domain: this.props.prefilled.domain || "",
            taskLimit: 1000,
            wordlists: [],
            wordlist: null,
        };
    }

    componentDidUpdate(prevProps: Readonly<WorkspaceAttacksBruteforceSubdomainsProps>) {
        if (this.props.prefilled.domain !== undefined && this.props.prefilled.domain !== prevProps.prefilled.domain)
            this.setState({ domain: this.props.prefilled.domain });
    }

    componentDidMount() {
        this.retrieveWordlists().then();
    }

    async retrieveWordlists() {
        await Api.wordlists.all().then(
            handleApiError((wordlists) =>
                this.setState({
                    wordlists: wordlists.wordlists.map((x) => {
                        return { label: x.name, value: x.uuid };
                    }),
                }),
            ),
        );
    }

    startAttack() {
        if (this.state.domain === "") {
            toast.error("Domain must not be empty");
            return;
        }

        if (this.state.wordlist === null) {
            toast.error("Wordlist must not be empty");
            return;
        }

        Api.attacks
            .bruteforceSubdomains({
                workspaceUuid: this.context.workspace.uuid,
                domain: this.state.domain,
                concurrentLimit: this.state.taskLimit,
                wordlistUuid: this.state.wordlist.value,
            })
            .then(handleApiError((_) => toast.success("Attack started")));
    }

    render() {
        return (
            <form
                className={"workspace-attacks-bsd-ct"}
                onSubmit={(event) => {
                    event.preventDefault();
                    this.startAttack();
                }}
            >
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
                <StartAttack active={this.state.domain !== "" && this.state.wordlist !== null} />
            </form>
        );
    }
}
