import React from "react";
import { toast } from "react-toastify";
import { Api } from "../api/api";
import { SimpleWorkspace } from "../api/generated";
import Checkbox from "../components/checkbox";
import Input from "../components/input";
import Loading from "../components/loading";
import Textarea from "../components/textarea";
import USER_CONTEXT from "../context/user";
import { ROUTES } from "../routes";
import "../styling/workspace-overview.css";
import WorkspaceIcon from "../svg/workspace";
import { check, handleApiError } from "../utils/helper";

/**
 * type to select how the displayed workspaces are sorted
 */
type Sorting = "none" | "name" | "createdAt";

/**
 * View to expose the `/api/v1/workspaces` endpoints
 *
 * @returns JSX.Element
 */
export default function WorkspaceOverview() {
    const context = React.useContext(USER_CONTEXT);
    // queried data
    const [workspaces, setWorkspaces] = React.useState<Array<SimpleWorkspace> | undefined>(undefined);

    // controlled state
    /** New workspace name */
    const [newName, setNewName] = React.useState<string>("");
    /** New workspace description */
    const [newDesc, setNewDesc] = React.useState<string>("");
    /** The search query */
    const [search, setSearch] = React.useState<string>("");

    const [onlyOwner, setOnlyOwner] = React.useState<boolean>(false);
    const [onlyMember, setOnlyMember] = React.useState<boolean>(false);
    const [onlyArchived, setOnlyArchived] = React.useState<boolean>(false);

    const [sorting, setSorting] = React.useState<Sorting>("none");

    /**
     * Api call to get all the existing workspaces
     */
    function retrieveAllWorkspaces() {
        Api.workspaces.all().then(handleApiError(({ workspaces }) => setWorkspaces(workspaces)));
    }

    React.useEffect(() => retrieveAllWorkspaces(), []);

    /**
     * Api call to create a new workspace
     */
    async function createWorkspace() {
        if (!check([[newName.length > 0, "Empty name"]])) return;

        await Api.workspaces.create({ name: newName, description: newDesc.length > 0 ? newDesc : null }).then(
            handleApiError((_) => {
                toast.success("Created new workspace");
                setNewName("");
                setNewDesc("");
                retrieveAllWorkspaces();
            }),
        );
    }

    return (
        <>
            {workspaces === undefined ? (
                <Loading />
            ) : (
                <div className={"workspace-list-outer-container"}>
                    <div className={"workspace-list-creation pane"}>
                        <WorkspaceIcon />
                        <form
                            className={"workspace-list-creation-form"}
                            method={"post"}
                            onSubmit={async (e) => {
                                e.preventDefault();
                                await createWorkspace();
                            }}
                        >
                            <h2 className={"heading"}>Create a new workspace</h2>
                            <div className={"workspace-list-creation-table"}>
                                <span>Name</span>
                                <Input
                                    value={newName}
                                    onChange={(v) => {
                                        setNewName(v);
                                    }}
                                />
                                <span>Description</span>
                                <Textarea
                                    value={newDesc}
                                    onChange={(v) => {
                                        setNewDesc(v);
                                    }}
                                />
                                <button className={"button"}>Create</button>
                            </div>
                        </form>
                    </div>
                    <div className={"workspace-list-filter pane"}>
                        <Input
                            placeholder={"Search"}
                            value={search}
                            onChange={(v) => {
                                setSearch(v);
                            }}
                        />
                        <div className={"workspace-list-filter-ownership"}>
                            <h3 className={"heading"}>Filter</h3>
                            <div className={"workspace-list-checkbox-table"}>
                                <label>
                                    <Checkbox
                                        value={onlyOwner}
                                        onChange={(v) => {
                                            setOnlyOwner(v);
                                            setOnlyMember(false);
                                        }}
                                    />
                                    <span>Owner</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={onlyMember}
                                        onChange={(v) => {
                                            setOnlyOwner(false);
                                            setOnlyMember(v);
                                        }}
                                    />
                                    <span>Member</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={onlyArchived}
                                        onChange={(v) => {
                                            setOnlyArchived(v);
                                        }}
                                    />
                                    <span>Archived</span>
                                </label>
                            </div>
                        </div>
                        <div className={"workspace-list-sorting"}>
                            <h3 className={"heading"}>Sorting</h3>
                            <div className={"workspace-list-checkbox-table"}>
                                <label>
                                    <Checkbox
                                        value={sorting === "name"}
                                        onChange={() => {
                                            setSorting(sorting === "name" ? "none" : "name");
                                        }}
                                    />
                                    <span>Name</span>
                                </label>
                                <label>
                                    <Checkbox
                                        value={sorting === "createdAt"}
                                        onChange={() => {
                                            setSorting(sorting === "createdAt" ? "none" : "createdAt");
                                        }}
                                    />
                                    <span>Created timestamp</span>
                                </label>
                            </div>
                        </div>
                    </div>
                    <div className={"workspace-list-container"}>
                        {workspaces
                            .filter((e) => {
                                if (search !== "" && !e.name.toLowerCase().includes(search.toLowerCase())) return false;

                                const isOwner = e.owner.uuid === context.user.uuid;

                                if (onlyOwner && !isOwner) return false;
                                if (onlyMember && isOwner) return false;

                                return onlyArchived == (e.archived ?? false);
                            })
                            .sort((a, b) => {
                                switch (sorting) {
                                    case "createdAt":
                                        return a.createdAt.getTime() - b.createdAt.getTime();
                                    case "name":
                                        return a.name.localeCompare(b.name);
                                    case "none":
                                        return 0;
                                }
                            })
                            .map((w) => {
                                return (
                                    <div
                                        className={`pane workspace-list-item ${w.archived ? "archived" : ""}`}
                                        {...ROUTES.WORKSPACE_DATA.clickHandler({ uuid: w.uuid })}
                                    >
                                        <h3 className={"heading"}>{w.name}</h3>
                                        <div className={"workspace-list-table"}>
                                            <span>Owner:</span>
                                            <span>{w.owner.displayName}</span>
                                            <span>Description:</span>
                                            <span>{w.description}</span>
                                            <span>Created at:</span>
                                            <span>{w.createdAt.toLocaleString()}</span>
                                        </div>
                                    </div>
                                );
                            })}
                    </div>
                </div>
            )}
        </>
    );
}
