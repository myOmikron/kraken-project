import { CSSProperties } from "react";
import { XYCoord, useDragLayer } from "react-dnd";
import { DragType, DragTypeNode } from "./common";
import { DesignWorkflowNodeEditor } from "./node";

const layerStyles: CSSProperties = {
    position: "fixed",
    pointerEvents: "none",
    zIndex: 100,
    left: 0,
    top: 0,
    width: "100%",
    height: "100%",
};

function getItemStyles(initialOffset: XYCoord | null, currentOffset: XYCoord | null) {
    if (initialOffset == null || currentOffset == null) {
        return {
            display: "none",
        };
    }

    let { x, y } = currentOffset;
    const transform = `translate(${x}px, ${y}px)`;
    return { transform };
}

export const WorkflowEditorDragLayer = () => {
    const { itemType, isDragging, item, initialOffset, currentOffset } = useDragLayer((monitor) => ({
        item: monitor.getItem() as DragTypeNode,
        itemType: monitor.getItemType() as DragType | null,
        initialOffset: monitor.getInitialSourceClientOffset(),
        currentOffset: monitor.getSourceClientOffset(),
        isDragging: monitor.isDragging(),
    }));

    function renderItem() {
        switch (itemType) {
            case DragType.node:
                let { x, y, id, ...props } = item;
                return <DesignWorkflowNodeEditor preview={true} {...props} />;
            default:
                return null;
        }
    }

    return (
        <div style={layerStyles}>
            {isDragging && <div style={getItemStyles(initialOffset, currentOffset)}>{renderItem()}</div>}
        </div>
    );
};
