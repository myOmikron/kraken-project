/** Database id i.e. and u32 */
export type ID = number;

/** Hyphen separated uuid */
export type UUID = string;

/** RFC3339 encoded date time */
export type RFC3339 = string;

/** Like `Partial<T>` but makes fields `| null` instead of `?` */
type Optional<T> = { [P in keyof T]: T[P] | null };

export type GetLeechResponse = {
    leeches: Array<GetLeech>;
};
export type GetLeech = {
    id: ID;
    name: string;
    address: string;
};
export type CreateLeechRequest = {
    name: string;
    address: string;
    description: string;
};
export type CreateLeechResponse = {
    id: ID;
};
export type CreateUserRequest = {
    username: string;
    display_name: string;
    password: string;
    admin: boolean;
};
export type UpdateLeechRequest = (
    | {
          name: string;
      }
    | {
          address: string;
      }
    | {
          description: string;
      }
) &
    Optional<CreateLeechRequest>;
export type CreateUserResponse = {
    uuid: UUID;
};
export type GetUserResponse = {
    users: Array<GetUser>;
};
export type GetUser = {
    uuid: UUID;
    username: string;
    display_name: string;
    admin: boolean;
    created_at: RFC3339;
};
export type SetPasswordRequest = {
    current_password: string;
    new_password: string;
};
export type GetWorkspacesResponse = {
    workspaces: Array<GetWorkspace>;
};
export type GetWorkspace = {
    id: ID;
    name: string;
    description: string | null;
};
export type CreateWorkspaceRequest = {
    name: string;
    description: string | null;
};
export type CreateWorkspaceResponse = {
    id: ID;
};
