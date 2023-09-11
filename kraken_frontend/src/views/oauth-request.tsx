import React from "react";
import { UUID } from "../api/api";
import "../styling/oauth-request.css";
import { SimpleWorkspace } from "../api/generated";

type OAuthRequestProps = {
    uuid: UUID;
};
type OAuthRequestState = {
    workspace: SimpleWorkspace | null;
    oauthApplication: { uuid: UUID; name: string } | null;
};

export default class OauthRequest extends React.Component<OAuthRequestProps, OAuthRequestState> {
    constructor(props: OAuthRequestProps) {
        super(props);

        this.state = {
            workspace: null,
            oauthApplication: null,
        };
    }

    componentDidMount() {}

    render() {
        return (
            <div className={"oauth-container"}>
                <div className={"pane oauth-panel"}>
                    {this.state.workspace !== null && this.state.oauthApplication !== null ? (
                        <>
                            <h1 className={"heading"}>
                                {this.state.oauthApplication.name} wants to request access to workspace
                                {this.state.workspace.name}.
                            </h1>
                            <p>
                                Granting will give {this.state.oauthApplication.name} read-only access to all
                                information and all future updates in the workspace {this.state.workspace.name} until
                                the access is revoked.
                            </p>
                            <p>You can always revoke the access in the settings of the workspace.</p>
                            <div className={"oauth-buttons"}>
                                <button
                                    className={"button"}
                                    onClick={() => {
                                        window.location.href = `/api/v1/oauth/accept/${this.props.uuid}`;
                                    }}
                                >
                                    Grant Access
                                </button>
                                <button
                                    className={"button"}
                                    onClick={() => {
                                        window.location.href = `/api/v1/oauth/deny/${this.props.uuid}`;
                                    }}
                                >
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
}
