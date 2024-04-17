import React from "react";
import Popup from "reactjs-popup";
import { OsType } from "../api/generated";
import AndroidIcon from "../svg/android";
import AnonymousIcon from "../svg/anonymous";
import AppleIcon from "../svg/apple";
import FreeBSDIcon from "../svg/freebsd";
import TuxIcon from "../svg/tux";
import WindowsIcon from "../svg/windows";

/** React props for [`<OsIcon />`]{@link OsIcon} */
type OsIconProps = {
    /** The OS whose icon to display */
    os: OsType;
    /** Show the OS' name in a tooltip? */
    tooltip?: boolean;
    /** Sets `style`'s `width` and `height` property */
    size?: string;
    /** Html's `style` attribute */
    style?: React.CSSProperties;
};

/** Shows an OS specific icon */
export default function OsIcon(props: OsIconProps) {
    const style = { ...props.style };
    if (props.size !== undefined) style.width = style.height = props.size;
    let icon;
    switch (props.os) {
        case "Linux":
            icon = <TuxIcon style={style} />;
            break;
        case "Apple":
            icon = <AppleIcon style={style} />;
            break;
        case "Windows":
            icon = <WindowsIcon style={style} />;
            break;
        case "FreeBSD":
            icon = <FreeBSDIcon style={style} />;
            break;
        case "Android":
            icon = <AndroidIcon style={style} />;
            break;
        case "Unknown":
        default:
            icon = <AnonymousIcon style={style} />;
            break;
    }
    return props.tooltip ? (
        <Popup trigger={<div>{icon}</div>} position={"bottom center"} on={"hover"} arrow={true}>
            <div className="pane-thin">{props.os}</div>
        </Popup>
    ) : (
        icon
    );
}
