import { Err, Ok, Result } from "../utils/result";
import { authenticate, login, logout, registerKey, test } from "./auth";
import { ApiError, StatusCode, parseError } from "./error";
import {
    AdminWorkspacesApi,
    ApiKeysApi,
    AttacksApi,
    Configuration,
    CreateAppRequest,
    CreateDomainRequest,
    CreateFindingAffectedRequest,
    CreateFindingDefinitionRequest,
    CreateFindingRequest,
    CreateGlobalTagRequest,
    CreateHostRequest,
    CreateLeechRequest,
    CreatePortRequest,
    CreateServiceRequest,
    CreateUserRequest,
    CreateWordlistRequest,
    CreateWorkspaceRequest,
    CreateWorkspaceTagRequest,
    DomainsApi,
    FindingsApi,
    GlobalTagsApi,
    HostsApi,
    KnowledgeBaseApi,
    LeechManagementApi,
    OAuthApi,
    OAuthApplicationApi,
    PortsApi,
    RequiredError,
    ResponseError,
    ServicesApi,
    SettingsManagementApi,
    UpdateAppRequest,
    UpdateDomainRequest,
    UpdateFindingAffectedRequest,
    UpdateFindingRequest,
    UpdateGlobalTag,
    UpdateHostRequest,
    UpdateLeechRequest,
    UpdateMeRequest,
    UpdatePortRequest,
    UpdateServiceRequest,
    UpdateSettingsRequest,
    UpdateWordlistRequest,
    UpdateWorkspaceRequest,
    UpdateWorkspaceTag,
    UserAdminManagementApi,
    UserManagementApi,
    WordlistApi,
    WordlistManagementApi,
    WorkspaceInvitationsApi,
    WorkspaceTagsApi,
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
const findings = new FindingsApi(configuration);
// const authentication = new generated.AuthenticationApi(configuration);
const leechManagement = new LeechManagementApi(configuration);
const userManagement = new UserManagementApi(configuration);
const workspaces = new WorkspacesApi(configuration);
const workspaceInvitations = new WorkspaceInvitationsApi(configuration);
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
const knowledgeBase = new KnowledgeBaseApi(configuration);

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
            genConfig: (uuid: UUID) => handleError(leechManagement.genLeechConfig({ uuid })),
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
        impl: attacks,
        all: () => handleError(attacks.getAllAttacks()),
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
        transferOwnership: (uuid: UUID, user: UUID) =>
            handleError(workspaces.transferOwnership({ uuid, transferWorkspaceRequest: { user } })),
        attacks: {
            all: (uuid: UUID) => handleError(attacks.getWorkspaceAttacks({ uuid })),
        },
        findings: {
            all: (workspace: UUID) => handleError(findings.getAllFindings({ uuid: workspace })),
            create: (workspace: UUID, options: CreateFindingRequest) =>
                handleError(findings.createFinding({ uuid: workspace, createFindingRequest: options })),
            get: (workspace: UUID, finding: UUID) =>
                handleError(findings.getFinding({ wUuid: workspace, fUuid: finding })),
            update: (workspace: UUID, finding: UUID, options: UpdateFindingRequest) =>
                handleError(
                    findings.updateFinding({ wUuid: workspace, fUuid: finding, updateFindingRequest: options }),
                ),
            delete: (workspace: UUID, finding: UUID) =>
                handleError(findings.deleteFinding({ wUuid: workspace, fUuid: finding })),
            addAffected: (workspace: UUID, finding: UUID, affected: CreateFindingAffectedRequest) =>
                handleError(
                    findings.createFindingAffected({
                        wUuid: workspace,
                        fUuid: finding,
                        createFindingAffectedRequest: affected,
                    }),
                ),
            getAffected: (workspace: UUID, finding: UUID, affected: UUID) =>
                handleError(
                    findings.getFindingAffected({
                        wUuid: workspace,
                        fUuid: finding,
                        aUuid: affected,
                    }),
                ),
            updateAffected: (workspace: UUID, finding: UUID, affected: UUID, options: UpdateFindingAffectedRequest) =>
                handleError(
                    findings.updateFindingAffected({
                        wUuid: workspace,
                        fUuid: finding,
                        aUuid: affected,
                        updateFindingAffectedRequest: options,
                    }),
                ),
            removeAffected: (workspace: UUID, finding: UUID, affected: UUID) =>
                handleError(
                    findings.deleteFindingAffected({
                        wUuid: workspace,
                        fUuid: finding,
                        aUuid: affected,
                    }),
                ),
        },
        invitations: {
            all: (uuid: UUID) => handleError(workspaces.getAllWorkspaceInvitations({ uuid })),
            create: (uuid: UUID, user: UUID) =>
                handleError(workspaces.createInvitation({ uuid, inviteToWorkspaceRequest: { user } })),
            retract: (wUuid: UUID, invitation: UUID) =>
                handleError(workspaces.retractInvitation({ wUuid, iUuid: invitation })),
        },
        hosts: {
            all: (
                workspaceUuid: UUID,
                limit: number,
                offset: number,
                filter: { host?: UUID; globalFilter?: string; hostFilter?: string } = {},
            ) =>
                handleError(hosts.getAllHosts({ uuid: workspaceUuid, getAllHostsQuery: { limit, offset, ...filter } })),
            get: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.getHost({ wUuid: workspaceUuid, hUuid: hostUuid })),
            update: (workspaceUuid: UUID, hostUuid: UUID, updateHostRequest: UpdateHostRequest) =>
                handleError(hosts.updateHost({ wUuid: workspaceUuid, hUuid: hostUuid, updateHostRequest })),
            create: (workspaceUuid: UUID, createHostRequest: CreateHostRequest) =>
                handleError(hosts.createHost({ uuid: workspaceUuid, createHostRequest })),
            delete: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.deleteHost({ wUuid: workspaceUuid, hUuid: hostUuid })),
            sources: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.getHostSources({ wUuid: workspaceUuid, hUuid: hostUuid })),
            relations: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.getHostRelations({ wUuid: workspaceUuid, hUuid: hostUuid })),
            findings: (workspaceUuid: UUID, hostUuid: UUID) =>
                handleError(hosts.getHostFindings({ wUuid: workspaceUuid, hUuid: hostUuid })),
        },
        ports: {
            all: (
                workspaceUuid: UUID,
                limit: number,
                offset: number,
                filter: { host?: UUID; globalFilter?: string; portFilter?: string } = {},
            ) =>
                handleError(ports.getAllPorts({ uuid: workspaceUuid, getAllPortsQuery: { limit, offset, ...filter } })),
            get: (workspaceUuid: UUID, portUuid: UUID) =>
                handleError(ports.getPort({ wUuid: workspaceUuid, pUuid: portUuid })),
            update: (workspaceUuid: UUID, portUuid: UUID, updatePortRequest: UpdatePortRequest) =>
                handleError(ports.updatePort({ wUuid: workspaceUuid, pUuid: portUuid, updatePortRequest })),
            create: (workspaceUuid: UUID, createPortRequest: CreatePortRequest) =>
                handleError(ports.createPort({ uuid: workspaceUuid, createPortRequest })),
            delete: (workspaceUuid: UUID, portUuid: UUID) =>
                handleError(ports.deletePort({ wUuid: workspaceUuid, pUuid: portUuid })),
            sources: (workspaceUuid: UUID, portUuid: UUID) =>
                handleError(ports.getPortSources({ wUuid: workspaceUuid, pUuid: portUuid })),
            relations: (workspaceUuid: UUID, portUuid: UUID) =>
                handleError(ports.getPortRelations({ wUuid: workspaceUuid, pUuid: portUuid })),
            findings: (workspaceUuid: UUID, portUuid: UUID) =>
                handleError(ports.getPortFindings({ wUuid: workspaceUuid, pUuid: portUuid })),
        },
        domains: {
            all: (
                workspaceUuid: UUID,
                limit: number,
                offset: number,
                filter: { host?: UUID; globalFilter?: string; domainFilter?: string } = {},
            ) =>
                handleError(
                    domains.getAllDomains({ uuid: workspaceUuid, getAllDomainsQuery: { limit, offset, ...filter } }),
                ),
            get: (workspaceUuid: UUID, domainUuid: UUID) =>
                handleError(domains.getDomain({ wUuid: workspaceUuid, dUuid: domainUuid })),
            update: (workspaceUuid: UUID, domainUuid: UUID, updateDomainRequest: UpdateDomainRequest) =>
                handleError(domains.updateDomain({ wUuid: workspaceUuid, dUuid: domainUuid, updateDomainRequest })),
            create: (workspaceUuid: UUID, createDomainRequest: CreateDomainRequest) =>
                handleError(domains.createDomain({ uuid: workspaceUuid, createDomainRequest })),
            delete: (workspaceUuid: UUID, domainUuid: UUID) =>
                handleError(domains.deleteDomain({ wUuid: workspaceUuid, dUuid: domainUuid })),
            sources: (workspaceUuid: UUID, domainUuid: UUID) =>
                handleError(domains.getDomainSources({ wUuid: workspaceUuid, dUuid: domainUuid })),
            relations: (workspaceUuid: UUID, domainUuid: UUID) =>
                handleError(domains.getDomainRelations({ wUuid: workspaceUuid, dUuid: domainUuid })),
            findings: (workspaceUuid: UUID, domainUuid: UUID) =>
                handleError(domains.getDomainFindings({ wUuid: workspaceUuid, dUuid: domainUuid })),
        },
        services: {
            all: (
                workspaceUuid: UUID,
                limit: number,
                offset: number,
                filter: { host?: UUID; globalFilter?: string; serviceFilter?: string } = {},
            ) =>
                handleError(
                    services.getAllServices({ uuid: workspaceUuid, getAllServicesQuery: { limit, offset, ...filter } }),
                ),
            get: (workspaceUuid: UUID, serviceUuid: UUID) =>
                handleError(services.getService({ wUuid: workspaceUuid, sUuid: serviceUuid })),
            update: (workspaceUuid: UUID, serviceUuid: UUID, updateServiceRequest: UpdateServiceRequest) =>
                handleError(services.updateService({ wUuid: workspaceUuid, sUuid: serviceUuid, updateServiceRequest })),
            create: (workspaceUuid: UUID, createServiceRequest: CreateServiceRequest) =>
                handleError(services.createService({ uuid: workspaceUuid, createServiceRequest })),
            delete: (workspaceUuid: UUID, serviceUuid: UUID) =>
                handleError(services.deleteService({ wUuid: workspaceUuid, sUuid: serviceUuid })),
            sources: (workspaceUuid: UUID, serviceUuid: UUID) =>
                handleError(services.getServiceSources({ wUuid: workspaceUuid, sUuid: serviceUuid })),
            relations: (workspaceUuid: UUID, serviceUuid: UUID) =>
                handleError(services.getServiceRelations({ wUuid: workspaceUuid, sUuid: serviceUuid })),
            findings: (workspaceUuid: UUID, serviceUuid: UUID) =>
                handleError(services.getServiceFindings({ wUuid: workspaceUuid, sUuid: serviceUuid })),
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
        create: (tag: CreateGlobalTagRequest) =>
            handleError(globalTags.createGlobalTag({ createGlobalTagRequest: tag })),
        update: (uuid: UUID, tag: UpdateGlobalTag) =>
            handleError(globalTags.updateGlobalTag({ uuid, updateGlobalTag: tag })),
        delete: (uuid: UUID) => globalTags.deleteGlobalTag({ uuid }),
    },
    wordlists: {
        all: () => handleError(wordlists.getAllWordlists()),
    },
    invitations: {
        all: () => handleError(workspaceInvitations.getAllInvitations()),
        accept: (uuid: UUID) => handleError(workspaceInvitations.acceptInvitation({ uuid })),
        decline: (uuid: UUID) => handleError(workspaceInvitations.declineInvitation({ uuid })),
    },
    knowledgeBase: {
        findingDefinitions: {
            all: () => handleError(knowledgeBase.getAllFindingDefinitions()),
            get: (findingDefinition: UUID) =>
                handleError(knowledgeBase.getFindingDefinition({ uuid: findingDefinition })),
            create: (createFindingDefinitionRequest: CreateFindingDefinitionRequest) =>
                handleError(knowledgeBase.createFindingDefinition({ createFindingDefinitionRequest })),
        },
    },
};

/**
 * Wraps a promise returned by the generated SDK which handles its errors and returns a {@link Result}
 */
export async function handleError<T>(promise: Promise<T>): Promise<Result<T, ApiError>> {
    try {
        return Ok(await promise);
    } catch (e) {
        if (e instanceof ResponseError) {
            return Err(await parseError(e.response));
        } else if (e instanceof RequiredError) {
            console.error(e);
            return Err({
                status_code: StatusCode.JsonDecodeError,
                message: "The server's response didn't match the spec",
            });
        } else {
            console.error("Unknown error occurred:", e);
            return Err({
                status_code: StatusCode.ArbitraryJSError,
                message: "Unknown error occurred",
            });
        }
    }
}
