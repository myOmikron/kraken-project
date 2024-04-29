import { ROUTES } from "../routes";
import "../styling/knowledge-base.css";

/**
 * Page to display knowledge base
 *
 * currently redirecting to {@link ListFindingDefinition `< ListFindingDefinition  />`}
 *
 * @returns JSX.Element page displaying knowledge base
 */
export default function KnowledgeBase() {
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
