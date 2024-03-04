import Popup from "reactjs-popup";
import AttacksIcon from "../../svg/attacks";

type WorkflowAttackSelectorProps = {
    open: boolean;
    onClose: () => any;
    x: number;
    y: number;
};

export function WorkflowAttackSelector(props: WorkflowAttackSelectorProps) {
    let contentWidth = 800;
    let contentHeight = 700;
    let x = Math.max(contentWidth / 2, Math.min(document.body.clientWidth - contentWidth / 2, props.x));
    let y = Math.max(contentHeight / 2, Math.min(document.body.clientHeight - contentHeight / 2, props.y));
    return (
        <Popup
            modal
            position="center center"
            open={props.open}
            onClose={props.onClose}
            contentStyle={{
                position: "absolute",
                left: x + "px",
                top: y + "px",
            }}
        >
            <AttacksIcon
                className="workflow-attack-selector visible"
                onAttackHover={(hoverAttack) => null}
                activeAttack={null}
                onAttackSelect={(selectedAttack) => null}
                activeAttackCategory={null}
                disabled={{}}
                onClickOutside={() => props.onClose()}
            />
        </Popup>
    );
}
