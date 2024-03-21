import Popup from "reactjs-popup";
import { FindingSeverity } from "../../../api/generated";
import "../../../index.css";

type SeverityIconProps = {
    severity: FindingSeverity | null | undefined;
    className?: string;
    tooltip?: boolean;
};

export default function SeverityIcon(props: SeverityIconProps) {
    const { className, severity, tooltip } = props;
    if (!severity) return null;
    let index =
        severity === FindingSeverity.Okay
            ? 0
            : severity === FindingSeverity.Low
              ? 1
              : severity === FindingSeverity.Medium
                ? 2
                : severity === FindingSeverity.High
                  ? 3
                  : severity === FindingSeverity.Critical
                    ? 4
                    : 5;
    const severities = [
        "severity-icon-ok",
        "severity-icon-low",
        "severity-icon-medium",
        "severity-icon-high",
        "severity-icon-critical",
        "neon",
    ] as const;
    const look = 0;
    return (
        <Popup
            trigger={
                <div className={className ?? "icon"}>
                    <svg
                        width="800px"
                        height="800px"
                        className={"severity-icon " + severities[index]}
                        viewBox="0 0 24 24"
                        stroke="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        {[...Array(4)].map((_, i) => (
                            <rect
                                x={4}
                                y={19 - i * 5}
                                rx={0.5}
                                width={16}
                                height={1}
                                strokeWidth={1}
                                className={`${index > i ? "filled" : ""}`}
                                strokeLinecap="round"
                            />
                        ))}
                    </svg>
                </div>
            }
            position={"bottom center"}
            on={tooltip === undefined || tooltip ? ["hover"] : []}
            arrow={true}
        >
            <div className="pane-thin">
                <span>
                    Severity: <b>{severity}</b>
                </span>
            </div>
        </Popup>
    );
}
