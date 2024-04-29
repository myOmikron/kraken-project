import Popup from "reactjs-popup";
import { Api, UUID } from "../../../api/api";
import { SimpleUser, SimpleWorkspace } from "../../../api/generated";
import "../../../styling/invitation.css";
import WorkspaceIcon from "../../../svg/workspace";
import { handleApiError } from "../../../utils/helper";

/** React props for <Invitation />*/
type InvitationProps = {
    /**
     * UUID of invitation
     */
    invitationUuid: UUID;
    /**
     * Workspace the invitation is for
     */
    workspace: SimpleWorkspace;
    /**
     * User who sent out the invitation
     */
    from: SimpleUser;
    /**
     * Callback after invitation is accepted or declined
     */
    onFinish(): void;
};

/**
 * Popup for invited user to accept or decline workspace invitation
 */
export default function Invitation(props: InvitationProps) {
    /**
     *Api call to accept workspace invitation
     * calls callback function on success
     *
     * @returns Promise<void>
     */
    function acceptInvitation() {
        return Api.invitations.accept(props.invitationUuid).then(handleApiError(() => props.onFinish()));
    }

    /**
     * Api call to decline workspace invitation
     * calls callback function on success
     *
     * @returns Promise<void>
     */
    function declineInvitation() {
        return Api.invitations.decline(props.invitationUuid).then(handleApiError(() => props.onFinish()));
    }

    return (
        <Popup closeOnEscape={false} closeOnDocumentClick={false} modal nested open>
            <div className="invitation-popup pane">
                <div className="invitation-icon">
                    <WorkspaceIcon />
                </div>
                <h2 className="sub-heading">Workspace invitation</h2>
                <span className="invitation-text">
                    <span>
                        {props.from.displayName} ({props.from.username}) invited you to join
                    </span>
                    <span>"{props.workspace.name}"</span>
                </span>
                <button
                    className="invitation-button button"
                    onClick={async (e) => {
                        e.preventDefault();
                        await acceptInvitation();
                    }}
                >
                    Accept
                </button>
                <button
                    className="invitation-button button"
                    onClick={async (e) => {
                        e.preventDefault();
                        await declineInvitation();
                    }}
                >
                    Decline
                </button>
            </div>
        </Popup>
    );
}
