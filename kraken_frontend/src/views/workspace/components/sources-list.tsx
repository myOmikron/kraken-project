import { SimpleAggregationSource } from "../../../api/generated";
import Bubble from "../../../components/bubble";
import { AttackResolver, ATTACKS } from "../../../utils/attack-resolver";

type SourcesListProps = {
    sources: SimpleAggregationSource;
};

const { Undefined: _, ...SOURCE_LIST_ATTACKS } = ATTACKS;
const SOURCE_LIST_TYPES: Omit<AttackResolver, "Undefined"> = SOURCE_LIST_ATTACKS;

export default function SourcesList(props: SourcesListProps) {
    return (
        <div className={"bubble-list"}>
            {Object.values(SOURCE_LIST_TYPES).map(({ key, abbreviation, long }) => {
                if (key === "undefined") return null;
                const count = props.sources[key];
                return count ? <Bubble color={"primary"} name={`${abbreviation} ${count}`} title={long} /> : null;
            })}
            {props.sources.manual ? <Bubble name={"MI"} title={"Manually inserted"} /> : null}
        </div>
    );
}
