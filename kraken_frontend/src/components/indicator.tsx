type IndicatorProps = {
    off?: boolean;
};
export default function Indicator(props: IndicatorProps) {
    return <div className={`indicator neon ${props.off ? "off" : ""}`} />;
}
