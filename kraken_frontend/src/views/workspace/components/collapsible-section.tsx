import React, { HTMLProps, ReactNode, useEffect } from "react";
import "../../../styling/collapsible-section.css";
import ArrowDownIcon from "../../../svg/arrow-down";

/** React props for {@link CollapsibleSection `<CollapsibleSection />`} */
export type CollapsibleSectionProps = {
    /**
     * ReactNode used for section heading
     */
    summary: ReactNode;
    /**
     * if section content is initially visible or not
     */
    defaultVisible?: boolean;
    visible?: boolean;
    /**
     * Optional callback when visibility of body is toggled
     */
    onChangeVisible?: (visible: boolean) => void;
} & Omit<HTMLProps<HTMLDivElement>, "summary">;

/**
 * Component for section with heading which is collapsible through an arrow.
 */
export default function CollapsibleSection(props: CollapsibleSectionProps) {
    const { summary, children, defaultVisible, visible: wantVisible, onChangeVisible } = props;
    const [visible, setVisible] = React.useState(defaultVisible ?? true);

    useEffect(() => {
        if (wantVisible !== undefined) setVisible(wantVisible);
    }, [wantVisible]);

    return (
        <div className="collapsible-section" {...props}>
            <h2 className={"sub-heading"}>
                {summary}
                <div
                    className="toggle"
                    onClick={() => {
                        onChangeVisible?.(!visible);
                        if (wantVisible === undefined) setVisible((v) => !v);
                    }}
                >
                    <ArrowDownIcon inverted={visible} />
                </div>
            </h2>
            {visible && children}
        </div>
    );
}
