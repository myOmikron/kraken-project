import { OsType } from "../api/generated";
import AnonymousIcon from "../svg/anonymous";
import TuxIcon from "../svg/tux";
import AppleIcon from "../svg/apple";
import WindowsIcon from "../svg/windows";
import FreeBSDIcon from "../svg/freebsd";
import AndroidIcon from "../svg/android";
import React from "react";

type OsIconProps = {
    os: OsType;
};
export default function OsIcon(props: OsIconProps) {
    switch (props.os) {
        case "Linux":
            return <TuxIcon />;
        case "Apple":
            return <AppleIcon />;
        case "Windows":
            return <WindowsIcon />;
        case "FreeBSD":
            return <FreeBSDIcon />;
        case "Android":
            return <AndroidIcon />;
        case "Unknown":
        default:
            return <AnonymousIcon />;
    }
}
