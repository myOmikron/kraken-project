export enum StatusCode {
    JsonDecodeError = -1,
    LoginFailed = 1000,
    NotFound = 1001,
    InvalidContentType = 1002,
    InvalidJson = 1003,
    PayloadOverflow = 1004,
    Unauthenticated = 1005,
    Missing2fa = 1006,
    MissingPrivileges = 1007,
    InternalServerError = 2000,
    DatabaseError = 2001,
    SessionError = 2002,
}

export type ApiError = {
    status_code: StatusCode;
    message: string;
};
