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
} from "./generated";
import { Configuration } from "./generated";
import {
    AdminWorkspacesApi,
    AttacksApi,
    LeechManagementApi,
    UserAdminManagementApi,
    UserManagementApi,
    WorkspacesApi,
} from "./generated";

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
            get: (uuid: UUID) => handleError(adminWorkspaces.getWorkspaceAdmin({ uuid })),
        },
        leeches: {
            all: () => handleError(leechManagement.getAllLeeches()),
            create: (leech: CreateLeechRequest) =>
                handleError(leechManagement.createLeech({ createLeechRequest: leech })),
            get: (uuid: UUID) => handleError(leechManagement.getLeech({ uuid })),
            update: (uuid: UUID, leech: UpdateLeechRequest) =>
                handleError(leechManagement.updateLeech({ uuid, updateLeechRequest: leech })),
            delete: (uuid: UUID) => handleError(leechManagement.deleteLeech({ uuid })),
        },
    },
    attacks: {
        bruteforceSubdomains: (attack: BruteforceSubdomainsRequest) =>
            handleError(attacks.bruteforceSubdomains({ bruteforceSubdomainsRequest: attack })),
        queryCertificateTransparency: (attack: QueryCertificateTransparencyRequest) =>
            handleError(attacks.queryCertificateTransparency({ queryCertificateTransparencyRequest: attack })),
        scanTcpPorts: (attack: ScanTcpPortsRequest) =>
            handleError(attacks.scanTcpPorts({ scanTcpPortsRequest: attack })),
        getTcpPortScanResults: (uuid: UUID, offset: number, limit: number) =>
            handleError(attacks.getTcpPortScanResults({ uuid, limit, offset })),
        get: (uuid: UUID) => handleError(attacks.getAttack({ uuid })),
        delete: (uuid: UUID) => handleError(attacks.deleteAttack({ uuid })),
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
        get: (uuid: UUID) => handleError(workspaces.getWorkspace({ uuid })),
        update: (uuid: UUID, workspace: UpdateWorkspaceRequest) =>
            handleError(workspaces.updateWorkspace({ uuid, updateWorkspaceRequest: workspace })),
        delete: (uuid: UUID) => handleError(workspaces.deleteWorkspace({ uuid })),
    },
};
