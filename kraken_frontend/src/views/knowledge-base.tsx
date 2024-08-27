import { ROUTES } from "../routes";
import "../styling/knowledge-base.css";

type KnowledgeBaseProps = {};

export default function KnowledgeBase(props: KnowledgeBaseProps) {
    return (
        <div className={"knowledge-base-container"}>
            <div className={"pane"}>
                <h2 className={"sub-heading"}>Knowledge Base</h2>
            </div>
            <div className={"pane knowledge-base-content"}>
                <div className={"knowledge-base-mask"}></div>
                <div className={"knowledge-base-eyes"}></div>
                <div />
                <div>
                    <button className={"button"} {...ROUTES.FINDING_DEFINITION_LIST.clickHandler({})}>
                        Finding Definitions
                    </button>
                </div>
            </div>
        </div>
    );
}
