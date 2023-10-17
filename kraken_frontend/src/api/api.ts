import { login, test, registerKey, authenticate, logout } from "./auth";
import { handleError } from "./error";
import {
    ApiKeysApi,
    BruteforceSubdomainsRequest,
    CreateAppRequest,
    CreateGlobalTagRequest,
    CreateLeechRequest,
    CreateUserRequest,
    CreateWordlistRequest,
    CreateWorkspaceRequest,
    CreateWorkspaceTagRequest,
    DomainsApi,
    GlobalTagsApi,
    HostsAliveRequest,
    HostsApi,
    OAuthApi,
    OAuthApplicationApi,
    PortsApi,
    Query,
    QueryCertificateTransparencyRequest,
    ScanTcpPortsRequest,
    ServiceDetectionRequest,
    ServicesApi,
    SettingsManagementApi,
    UpdateAppRequest,
    UpdateGlobalTag,
    UpdateHostRequest,
    UpdateLeechRequest,
    UpdateMeRequest,
    UpdateSettingsRequest,
    UpdateWordlistRequest,
    UpdateWorkspaceRequest,
    UpdateWorkspaceTag,
    WordlistApi,
    WordlistManagementApi,
    WorkspaceTagsApi,
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
const oauth = new OAuthApi(configuration);
const oauthApplications = new OAuthApplicationApi(configuration);
const settingsManagement = new SettingsManagementApi(configuration);
const globalTags = new GlobalTagsApi(configuration);
const workspaceTags = new WorkspaceTagsApi(configuration);
const hosts = new HostsApi(configuration);
const ports = new PortsApi(configuration);
const domains = new DomainsApi(configuration);
const services = new ServicesApi(configuration);
const apiKeys = new ApiKeysApi(configuration);
const wordlists = new WordlistApi(configuration);
const wordlistsManagement = new WordlistManagementApi(configuration);

export const Api = {
    admin: {
        users: {
            all: () => handleError(userAdminManagement.getAllUsersAdmin()),
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
        settings: {
            get: () => handleError(settingsManagement.getSettings()),
            update: (settings: UpdateSettingsRequest) =>
                handleError(settingsManagement.updateSettings({ updateSettingsRequest: settings })),
        },
        oauthApplications: {
            all: () => handleError(oauthApplications.getAllOauthApps({})),
            get: (uuid: UUID) => handleError(oauthApplications.getOauthApp({ uuid })),
            create: (oauthApplication: CreateAppRequest) =>
                handleError(oauthApplications.createOauthApp({ createAppRequest: oauthApplication })),
            update: (uuid: UUID, updateAppRequest: UpdateAppRequest) =>
                handleError(oauthApplications.updateOauthApp({ uuid, updateAppRequest })),
            delete: (uuid: UUID) => handleError(oauthApplications.deleteOauthApp({ uuid })),
        },
        globalTags: {
            create: (createGlobalTagRequest: CreateGlobalTagRequest) =>
                handleError(globalTags.createGlobalTag({ createGlobalTagRequest })),
            delete: (uuid: UUID) => handleError(globalTags.deleteGlobalTag({ uuid })),
            update: (uuid: UUID, updateGlobalTag: UpdateGlobalTag) =>
                handleError(globalTags.updateGlobalTag({ uuid, updateGlobalTag })),
        },
        wordlists: {
            all: () => handleError(wordlistsManagement.getAllWordlistsAdmin({})),
            create: (createWordlistRequest: CreateWordlistRequest) =>
                handleError(wordlistsManagement.createWordlistAdmin({ createWordlistRequest })),
            update: (uuid: UUID, updateWordlistRequest: UpdateWordlistRequest) =>
                handleError(wordlistsManagement.updateWordlistAdmin({ uuid, updateWordlistRequest })),
            delete: (uuid: UUID) => handleError(wordlistsManagement.deleteWordlistAdmin({ uuid })),
        },
    },
    attacks: {
        bruteforceSubdomains: (attack: BruteforceSubdomainsRequest) =>
            handleError(attacks.bruteforceSubdomains({ bruteforceSubdomainsRequest: attack })),
        queryCertificateTransparency: (attack: QueryCertificateTransparencyRequest) =>
            handleError(attacks.queryCertificateTransparency({ queryCertificateTransparencyRequest: attack })),
        hostAlive: (hostsAliveRequest: HostsAliveRequest) =>
            handleError(attacks.hostsAliveCheck({ hostsAliveRequest })),
        scanTcpPorts: (attack: ScanTcpPortsRequest) =>
            handleError(attacks.scanTcpPorts({ scanTcpPortsRequest: attack })),
        getTcpPortScanResults: (uuid: UUID, offset: number, limit: number) =>
            handleError(attacks.getTcpPortScanResults({ uuid, limit, offset })),
        serviceDetection: (attack: ServiceDetectionRequest) =>
            handleError(attacks.serviceDetection({ serviceDetectionRequest: attack })),
        queryDehashed: (uuid: UUID, query: Query) =>
            handleError(attacks.queryDehashed({ queryDehashedRequest: { workspaceUuid: uuid, query } })),
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
        all: () => handleError(userManagement.getAllUsers()),
        apiKeys: {
            create: (name: string) => handleError(apiKeys.createApiKey({ createApiKeyRequest: { name } })),
            all: () => handleError(apiKeys.getApiKeys()),
            delete: (uuid: UUID) => handleError(apiKeys.deleteApiKey({ uuid })),
            update: (uuid: UUID, name: string) =>
                handleError(apiKeys.updateApiKey({ uuid, updateApiKeyRequest: { name } })),
        },
    },
    workspaces: {
        all: () => handleError(workspaces.getAllWorkspaces()),
        create: (workspace: CreateWorkspaceRequest) =>
            handleError(workspaces.createWorkspace({ createWorkspaceRequest: workspace })),
        get: (uuid: UUID) => handleError(workspaces.getWorkspace({ uuid })),
        update: (uuid: UUID, workspace: UpdateWorkspaceRequest) =>
            handleError(workspaces.updateWorkspace({ uuid, updateWorkspaceRequest: workspace })),
        delete: (uuid: UUID) => handleError(workspaces.deleteWorkspace({ uuid })),
        hosts: {
            all: (workspaceUuid: UUID, limit: number, offset: number) =>
                handleError(hosts.getAllHosts({ uuid: workspaceUuid, limit, offset })),
            get: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.getHost({ wUuid: workspaceUuid, hUuid: hostUuid })),
            update: (workspaceUuid: UUID, hostUuid: UUID, updateHostRequest: UpdateHostRequest) =>
                handleError(hosts.updateHost({ wUuid: workspaceUuid, hUuid: hostUuid, updateHostRequest })),
        },
        ports: {
            all: (workspaceUuid: UUID, limit: number, offset: number, filter: { host?: UUID } = {}) =>
                handleError(ports.getAllPorts({ uuid: workspaceUuid, limit, offset, ...filter })),
        },
        domains: {
            all: (workspaceUuid: UUID, limit: number, offset: number) =>
                handleError(domains.getAllDomains({ uuid: workspaceUuid, limit, offset })),
        },
        services: {
            all: (workspaceUuid: UUID, limit: number, offset: number, filter: { host?: UUID } = {}) =>
                handleError(services.getAllServices({ uuid: workspaceUuid, limit, offset, ...filter })),
        },
        tags: {
            all: (workspaceUuid: UUID) => handleError(workspaceTags.getAllWorkspaceTags({ uuid: workspaceUuid })),
            create: (workspaceUuid: UUID, createWorkspaceTagRequest: CreateWorkspaceTagRequest) =>
                handleError(workspaceTags.createWorkspaceTag({ uuid: workspaceUuid, createWorkspaceTagRequest })),
            update: (workspaceUuid: UUID, tagUuid: UUID, updateWorkspaceTag: UpdateWorkspaceTag) =>
                handleError(
                    workspaceTags.updateWorkspaceTag({ wUuid: workspaceUuid, tUuid: tagUuid, updateWorkspaceTag }),
                ),
            delete: (workspaceUuid: UUID, tagUuid: UUID) =>
                workspaceTags.deleteWorkspaceTag({ wUuid: workspaceUuid, tUuid: tagUuid }),
        },
    },
    oauth: {
        info: (uuid: UUID) => handleError(oauth.info({ uuid })),
    },
    globalTags: {
        all: () => handleError(globalTags.getAllGlobalTags()),
    },
    wordlists: {
        all: () => handleError(wordlists.getAllWordlists()),
    },
};
