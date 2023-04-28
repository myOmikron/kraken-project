import { login, test, registerKey, authenticate, logout } from "./auth";
import { handleError } from "./error";
import {
    BruteforceSubdomainsRequest,
    CreateLeechRequest,
    CreateUserRequest,
    CreateWorkspaceRequest,
    QueryCertificateTransparencyRequest,
    ScanTcpPortsRequest,
    UpdateLeechRequest,
    UpdateMeRequest,
    UpdateWorkspaceRequest,
} from "./generated/models";
import { Configuration } from "./generated/runtime";
import {
    AdminWorkspacesApi,
    AttacksApi,
    LeechManagementApi,
    UserAdminManagementApi,
    UserManagementApi,
    WorkspacesApi,
} from "./generated/apis";

/** Database id i.e. and u32 */
export type ID = number;

/** Hyphen separated uuid */
export type UUID = string;

const configuration = new Configuration({
    basePath: window.location.origin,
});
const userAdminManagement = new UserAdminManagementApi(configuration);
const adminWorkspaces = new AdminWorkspacesApi(configuration);
const attacks = new AttacksApi(configuration);
// const authentication = new generated.AuthenticationApi(configuration);
const leechManagement = new LeechManagementApi(configuration);
const userManagement = new UserManagementApi(configuration);
const workspaces = new WorkspacesApi(configuration);

export const Api = {
    admin: {
        users: {
            all: () => handleError(userAdminManagement.getAllUsers()),
            create: (user: CreateUserRequest) =>
                handleError(userAdminManagement.createUser({ createUserRequest: user })),
            get: (uuid: UUID) => handleError(userAdminManagement.getUser({ uuid })),
            delete: (uuid: UUID) => handleError(userAdminManagement.deleteUser({ uuid })),
        },
        workspaces: {
            all: () => handleError(adminWorkspaces.getAllWorkspacesAdmin()),
            get: (id: ID) => handleError(adminWorkspaces.getWorkspaceAdmin({ id })),
        },
        leeches: {
            all: () => handleError(leechManagement.getAllLeeches()),
            create: (leech: CreateLeechRequest) =>
                handleError(leechManagement.createLeech({ createLeechRequest: leech })),
            get: (id: ID) => handleError(leechManagement.getLeech({ id })),
            update: (id: ID, leech: UpdateLeechRequest) =>
                handleError(leechManagement.updateLeech({ id, updateLeechRequest: leech })),
            delete: (id: ID) => handleError(leechManagement.deleteLeech({ id })),
        },
    },
    attacks: {
        bruteforceSubdomains: (attack: BruteforceSubdomainsRequest) =>
            handleError(attacks.bruteforceSubdomains({ bruteforceSubdomainsRequest: attack })),
        queryCertificateTransparency: (attack: QueryCertificateTransparencyRequest) =>
            handleError(attacks.queryCertificateTransparency({ queryCertificateTransparencyRequest: attack })),
        scanTcpPorts: (attack: ScanTcpPortsRequest) =>
            handleError(attacks.scanTcpPorts({ scanTcpPortsRequest: attack })),
    },
    auth: {
        login,
        logout,
        test,
        registerKey,
        authenticate,
    },
    user: {
        get: () => handleError(userManagement.getMe()),
        update: (user: UpdateMeRequest) => handleError(userManagement.updateMe({ updateMeRequest: user })),
        setPassword: (currentPassword: string, newPassword: string) =>
            handleError(userManagement.setPassword({ setPasswordRequest: { currentPassword, newPassword } })),
    },
    workspaces: {
        all: () => handleError(workspaces.getAllWorkspaces()),
        create: (workspace: CreateWorkspaceRequest) =>
            handleError(workspaces.createWorkspace({ createWorkspaceRequest: workspace })),
        get: (id: ID) => handleError(workspaces.getWorkspace({ id })),
        update: (id: ID, workspace: UpdateWorkspaceRequest) =>
            handleError(workspaces.updateWorkspace({ id, updateWorkspaceRequest: workspace })),
        delete: (id: ID) => handleError(workspaces.deleteWorkspace({ id })),
    },
};
