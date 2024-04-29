import React from "react";
import Popup from "reactjs-popup";
import { DomainCertainty, HostCertainty, PortCertainty, ServiceCertainty } from "../../../api/generated";
import HistoricalIcon from "../../../svg/historical";
import UnknownIcon from "../../../svg/unknown";
import UnverifiedIcon from "../../../svg/unverified";
import VerifiedIcon from "../../../svg/verified";

/** React props for {@link CertaintyIcon `<CertaintyIcon />`} */
type CertaintyIconProps = {
    /**
     * type of certainty
     */
    certainty: DomainCertainty | HostCertainty | PortCertainty | ServiceCertainty;
    /**
     * toggle if name is visible next to the certainty icon
     */
    nameVisible?: boolean | undefined;
};

/** Global lookup of certainties' icons, names and descriptions */
const CERTAINTIES: Record<
    DomainCertainty | HostCertainty | PortCertainty | ServiceCertainty,
    {
        /**
         * Svg icon for certainty
         */
        icon: React.ReactNode;
        /**
         * displayed certainty name
         */
        name: string;
        /**
         * displayed certainty description
         */
        description: string;
    }
> = {
    Verified: {
        icon: <VerifiedIcon />,
        name: "Verified",
        description: "The kraken has verified it",
    },

    // domain specific
    Unverified: {
        icon: <UnverifiedIcon />,
        name: "Unverified",
        description: "The kraken hasn't queried this domain yet.",
    },

    // host/port specific
    SupposedTo: {
        icon: <span className="workspace-data-certainty-letter">S</span>,
        name: "Supposed to",
        description: "This should represent current state, but the kraken hasn't checked it yet.",
    },
    Historical: {
        icon: <HistoricalIcon />,
        name: "Historical",
        description: "This might have been the state in the past.",
    },

    // service specific
    DefinitelyVerified: {
        icon: (
            <div>
                <VerifiedIcon />
                <span className="workspace-data-certainty-letter">D</span>
            </div>
        ),
        name: "Definitely Verified",
        description: "The service detection had an exact match",
    },
    MaybeVerified: {
        icon: (
            <div>
                <VerifiedIcon />
                <span className="workspace-data-certainty-letter">M</span>
            </div>
        ),
        name: "Maybe Verified",
        description: "The service detection had a partial match",
    },
    UnknownService: {
        icon: <UnknownIcon />,
        name: "Unknown Service",
        description: "",
    },
};

/**
 * Displays certainty icon.
 * On hover displays popup with certainty name and description
 */
export default function CertaintyIcon(props: CertaintyIconProps) {
    const { certainty, nameVisible } = props;
    const { icon, name, description } = CERTAINTIES[certainty];
    return (
        <Popup
            trigger={
                <span className="workspace-data-certainty-icon icon">
                    {icon} {nameVisible && name}
                </span>
            }
            position={"bottom center"}
            on={"hover"}
            arrow={true}
        >
            <div className="pane-thin">
                <h2 className="sub-heading">{name}</h2>
                <span>{description}</span>
            </div>
        </Popup>
    );
}
