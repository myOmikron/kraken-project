import Home from "./views/home";
import React from "react";
import { Router } from "./utils/router";
import KrakenNetwork from "./views/kraken-network";
import Me from "./views/me";
import AdminUsers from "./views/admin/users";
import AdminWorkspaces from "./views/admin/workspaces";
import AdminGuard from "./components/admin-guard";
import WorkspaceOverview from "./views/workspace-overview";
import OauthRequest from "./views/oauth-request";
import { ContentWithMenu } from "./views/menu";
import Settings from "./views/settings";
import Workspace from "./views/workspace/workspace";
import WorkspaceSettings from "./views/workspace/workspace-settings";
import WorkspaceData from "./views/workspace/workspace-data";
import WorkspaceAttacks, { TargetType } from "./views/workspace/workspace-attacks";
import WorkspaceHosts from "./views/workspace/workspace-hosts";
import WorkspaceHost from "./views/workspace/workspace-host";
import KnowledgeBase from "./views/knowledge-base";
import { CreateFindingDefinition } from "./views/knowledge-base/create-finding-definition";
import { EditFindingDefinition } from "./views/knowledge-base/edit-finding-definition";
import { ListFindingDefinition } from "./views/knowledge-base/list-finding-definition";
import WorkspaceNotes from "./views/workspace/workspace-notes";
import WorkspaceFindings from "./views/workspace/workspace-findings";
import { WorkspaceCreateFinding } from "./views/workspace/workspace-finding/workspace-create-finding";
import WorkspaceEditFinding from "./views/workspace/workspace-finding/workspace-edit-finding";

export const ROUTER = new Router();

export const ROUTES = {
    HOME: ROUTER.add({
        url: "",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <Home />
            </ContentWithMenu>
        ),
    }),
    ME: ROUTER.add({
        url: "me",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <Me />
            </ContentWithMenu>
        ),
    }),
    WORKSPACES: ROUTER.add({
        url: "workspaces",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <WorkspaceOverview />
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_SETTINGS: ROUTER.add({
        url: "workspaces/{uuid}/settings",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"settings"} uuid={uuid}>
                    <WorkspaceSettings />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_DATA: ROUTER.add({
        url: "workspaces/{uuid}/data",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"data"} uuid={uuid}>
                    <WorkspaceData />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_ATTACKS: ROUTER.add({
        url: "workspaces/{uuid}/attacks",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"attacks"} uuid={uuid}>
                    <WorkspaceAttacks />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_SELECTION_ATTACKS: ROUTER.add<
        {
            workspaceUuid: string;
        },
        {
            domains: string[];
            hosts: string[];
            ports: string[];
            services: string[];
        }
    >({
        url: "workspaces/{workspaceUuid}/attacks/selection",
        parser: { workspaceUuid: String },
        render: ({ workspaceUuid }, hiddenParams) => {
            if (hiddenParams === undefined) {
                ROUTES.WORKSPACE_ATTACKS.visit({ uuid: workspaceUuid });
                return <></>;
            }
            let { domains, hosts, ports, services } = hiddenParams;
            return (
                <ContentWithMenu>
                    <Workspace view={"attacks"} uuid={workspaceUuid}>
                        <WorkspaceAttacks
                            targetType="selection"
                            domains={domains}
                            hosts={hosts}
                            ports={ports}
                            services={services}
                        />
                    </Workspace>
                </ContentWithMenu>
            );
        },
    }),
    WORKSPACE_TARGETED_ATTACKS: ROUTER.add({
        url: "workspaces/{workspaceUuid}/attacks/{targetType}/{targetUuid}",
        parser: { workspaceUuid: String, targetType: TargetType, targetUuid: String },
        render: ({ workspaceUuid, targetType, targetUuid }) => (
            <ContentWithMenu>
                <Workspace view={"attacks"} uuid={workspaceUuid}>
                    <WorkspaceAttacks targetType={targetType} targetUuid={targetUuid} />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_FINDINGS: ROUTER.add({
        url: "workspaces/{uuid}/findings",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace uuid={uuid} view={"findings"}>
                    <WorkspaceFindings />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_FINDINGS_CREATE: ROUTER.add({
        url: "workspaces/{uuid}/findings/create",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace uuid={uuid} view={"findings"}>
                    <WorkspaceCreateFinding />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_FINDINGS_EDIT: ROUTER.add({
        url: "workspaces/{wUuid}/findings/{fUuid}",
        parser: { wUuid: String, fUuid: String },
        render: ({ wUuid, fUuid }) => (
            <ContentWithMenu>
                <Workspace uuid={wUuid} view={"findings"}>
                    <WorkspaceEditFinding uuid={fUuid} />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_SEARCH: ROUTER.add({
        url: "workspaces/{uuid}/search",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"search"} uuid={uuid}>
                    {null}
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_HOSTS: ROUTER.add({
        url: "workspaces/{uuid}/hosts",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"hosts"} uuid={uuid}>
                    <WorkspaceHosts />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_SINGLE_HOST: ROUTER.add({
        url: "workspaces/{w_uuid}/hosts/{h_uuid}",
        parser: { w_uuid: String, h_uuid: String },
        render: ({ w_uuid, h_uuid }) => (
            <ContentWithMenu>
                <Workspace view={"hosts"} uuid={w_uuid}>
                    <WorkspaceHost uuid={h_uuid} />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    WORKSPACE_NOTES: ROUTER.add({
        url: "workspaces/{uuid}/notes",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <Workspace view={"notes"} uuid={uuid}>
                    <WorkspaceNotes />
                </Workspace>
            </ContentWithMenu>
        ),
    }),
    KNOWLEDGE_BASE: ROUTER.add({
        url: "knowledge",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <KnowledgeBase />
            </ContentWithMenu>
        ),
    }),
    FINDING_DEFINITION_LIST: ROUTER.add({
        url: "knowledge/finding-definition",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <ListFindingDefinition />
            </ContentWithMenu>
        ),
    }),
    FINDING_DEFINITION_CREATE: ROUTER.add({
        url: "knowledge/finding-definition/create",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <CreateFindingDefinition />
            </ContentWithMenu>
        ),
    }),
    FINDING_DEFINITION_EDIT: ROUTER.add({
        url: "knowledge/finding-definition/{uuid}",
        parser: { uuid: String },
        render: ({ uuid }) => (
            <ContentWithMenu>
                <EditFindingDefinition uuid={uuid} />
            </ContentWithMenu>
        ),
    }),
    OAUTH_REQUEST: ROUTER.add({
        url: "oauth-request/{uuid}",
        parser: { uuid: String },
        render: ({ uuid }) => <OauthRequest uuid={uuid} />,
    }),

    KRAKEN_NETWORK: ROUTER.add({
        url: "kraken-network",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <AdminGuard>
                    <KrakenNetwork />
                </AdminGuard>
            </ContentWithMenu>
        ),
    }),
    ADMIN_USER_MANAGEMENT: ROUTER.add({
        url: "admin/users",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <AdminGuard>
                    <AdminUsers />
                </AdminGuard>
            </ContentWithMenu>
        ),
    }),
    ADMIN_WORKSPACE_MANAGEMENT: ROUTER.add({
        url: "admin/workspaces",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <AdminGuard>
                    <AdminWorkspaces />
                </AdminGuard>
            </ContentWithMenu>
        ),
    }),
    ADMIN_SETTINGS: ROUTER.add({
        url: "admin/settings",
        parser: {},
        render: () => (
            <ContentWithMenu>
                <AdminGuard>
                    <Settings />
                </AdminGuard>
            </ContentWithMenu>
        ),
    }),
};

ROUTER.finish();
