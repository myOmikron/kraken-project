import React from "react";
import { Api, UUID } from "../api/api";
import { SimpleOauthClient, SimpleWorkspace } from "../api/generated";
import Checkbox from "../components/checkbox";
import "../styling/oauth-request.css";
import { handleApiError } from "../utils/helper";

type OAuthRequestProps = {
    uuid: UUID;
};

export default function OauthRequest(props: OAuthRequestProps) {
    const { uuid } = props;
    const [workspace, setWorkspace] = React.useState<SimpleWorkspace | null>(null);
    const [oauthApplication, setOauthApplication] = React.useState<SimpleOauthClient | null>(null);
    const [remember, setRemember] = React.useState<boolean>(false);

    React.useEffect(() => {
        Api.oauth.info(uuid).then(
            handleApiError(({ workspace, oauthApplication }) => {
                setWorkspace(workspace);
                setOauthApplication(oauthApplication);
            }),
        );
    }, []);

    function redirect(choice: "accept" | "deny") {
        let url = `/api/v1/oauth/${choice}/${uuid}`;
        if (remember) url = url + "?remember=true";
        window.location.href = url;
    }

    return (
        <div className={"oauth-container"}>
            <div className={"pane oauth-panel"}>
                {workspace !== null && oauthApplication !== null ? (
                    <>
                        <h1 className={"heading"}>
                            {oauthApplication.name} wants to request access to workspace {workspace.name}.
                        </h1>
                        <p>
                            Granting will give {oauthApplication.name} read-only access to all information and all
                            future updates in the workspace {workspace.name} until the access is revoked.
                        </p>
                        <p>You can always revoke the access in the settings of the workspace.</p>
                        <label>
                            <Checkbox value={remember} onChange={(remember) => setRemember(remember)} />
                            Remember my decision
                        </label>
                        <div className={"oauth-buttons"}>
                            <button className={"button"} onClick={() => redirect("accept")}>
                                Grant Access
                            </button>
                            <button className={"button"} onClick={() => redirect("deny")}>
                                Deny Access
                            </button>
                        </div>
                    </>
                ) : (
                    <p>Loading information... </p>
                )}
            </div>
        </div>
    );
}
