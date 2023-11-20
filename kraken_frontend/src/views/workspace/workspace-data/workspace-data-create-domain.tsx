import React from "react";
import { Api } from "../../../api/api";
import { handleApiError } from "../../../utils/helper";
import { toast } from "react-toastify";
import Input from "../../../components/input";

type CreateDomainFormProps = {
    workspace: string;
    onSubmit: () => void;
};

export function CreateDomainForm(props: CreateDomainFormProps) {
    const { workspace, onSubmit } = props;
    const [domain, setDomain] = React.useState("");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                Api.workspaces.domains.create(workspace, { domain }).then(
                    handleApiError(() => {
                        toast.success("Added domain");
                        onSubmit();
                    }),
                );
            }}
        >
            <h2 className={"sub-heading"}>Manually add a domain</h2>
            <label>
                Domain:
                <Input value={domain} onChange={setDomain} />
            </label>
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
