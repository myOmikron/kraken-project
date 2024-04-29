import React from "react";
import { toast } from "react-toastify";
import { Api } from "../../../api/api";
import Input from "../../../components/input";
import { handleApiError } from "../../../utils/helper";
import { WORKSPACE_CONTEXT } from "../workspace";

/** React props for [`<CreateDomainForm />`]{@link CreateDomainForm} */
type CreateDomainFormProps = {
    /**
     * Callback when new Domain was successfully created
     */
    onSubmit: () => void;
};

/**
 * `<form />`including all inputs to manually create a new Domain
 *
 * Allows submitting to create in the current workspace.
 */
export function CreateDomainForm(props: CreateDomainFormProps) {
    const { onSubmit } = props;
    const {
        workspace: { uuid: workspace },
    } = React.useContext(WORKSPACE_CONTEXT);
    const [domain, setDomain] = React.useState("");
    return (
        <form
            className={"pane workspace-data-create-form"}
            onSubmit={(event) => {
                event.preventDefault();
                if (domain.search(" ") >= 0) {
                    toast.error("Domain must not contain whitespace");
                    return;
                }
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
                <Input value={domain} onChange={setDomain} required />
            </label>
            <button className={"button"} type={"submit"}>
                Add
            </button>
        </form>
    );
}
