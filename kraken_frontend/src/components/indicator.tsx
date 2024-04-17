/** React props for [`<Indicator />`]{@link Indicator} */
type IndicatorProps = {
    /** Is the indicator "off" i.e. empty? */
    off?: boolean;
};

/** A circle which can be empty or filled representing some boolean value */
export default function Indicator(props: IndicatorProps) {
    return <div className={`indicator neon ${props.off ? "off" : ""}`} />;
}
