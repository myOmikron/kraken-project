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
    size?: string;
    style?: React.CSSProperties;
};
export default function OsIcon(props: OsIconProps) {
    let style: any = { ...props.style };
    if (props.size !== undefined) style.width = style.height = props.size;
    switch (props.os) {
        case "Linux":
            return <TuxIcon style={style} />;
        case "Apple":
            return <AppleIcon style={style} />;
        case "Windows":
            return <WindowsIcon style={style} />;
        case "FreeBSD":
            return <FreeBSDIcon style={style} />;
        case "Android":
            return <AndroidIcon style={style} />;
        case "Unknown":
        default:
            return <AnonymousIcon style={style} />;
    }
}
