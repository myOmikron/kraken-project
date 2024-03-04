import { CSSProperties, HTMLAttributes, forwardRef, useEffect, useRef, useState } from "react";
import "../../../styling/viewport.css";

export type ViewportProps = {
    connections?: {
        from: [number, number];
        to: [number, number];
    }[];
} & HTMLAttributes<HTMLDivElement>;

/**
 * Viewport component that can drag around and zoom its contents. 0,0 defaults
 * to center of screen at start.
 */
export const Viewport = forwardRef(({ connections, className, children, ...props }: ViewportProps, propsRef) => {
    const ref = useRef<HTMLDivElement | null>(null);
    const [position, setPosition] = useState([0, 0]);
    const [zoom, setZoom] = useState(0);
    // round the whole result since otherwise the text gets blurry with more precision
    const zoomToScale = (zoom: number) => Math.round(Math.pow(2, Math.round(zoom * 8) / 8) * 100) / 100;
    const scale = zoomToScale(zoom);

    if (connections === undefined) connections = [];

    useEffect(() => {
        if (ref.current) {
            const element = ref.current;
            let pressedPointer: number | undefined = undefined;
            const pointerdown = (e: PointerEvent) => {
                if (pressedPointer !== undefined) return;
                if (e.button != 1) return;
                pressedPointer = e.pointerId;
            };
            const pointerup = (e: PointerEvent) => {
                if (pressedPointer !== pressedPointer) return;
                pressedPointer = undefined;
            };
            const pointermove = (e: PointerEvent) => {
                if (pressedPointer !== pressedPointer || pressedPointer === undefined) return;
                setPosition((p) => [p[0] + e.movementX, p[1] + e.movementY]);
            };
            const wheel = (e: WheelEvent) => {
                setZoom((zoom) => {
                    let newZoom = Math.max(-2, Math.min(2, zoom - e.deltaY / 480));
                    setPosition((position) => {
                        let bounds = ref.current!.getBoundingClientRect();
                        // offset by mouse cursor position
                        let ox = e.clientX - bounds.left - bounds.width / 2;
                        let oy = e.clientY - bounds.top - bounds.height / 2;
                        console.log(ox, oy);
                        // global origin / scale -> origin
                        let lx = (position[0] - ox) / zoomToScale(zoom);
                        let ly = (position[1] - oy) / zoomToScale(zoom);
                        // origin * new scale -> new global origin
                        lx *= zoomToScale(newZoom);
                        ly *= zoomToScale(newZoom);
                        return [lx + ox, ly + oy];
                    });
                    return newZoom;
                });
            };

            element.addEventListener("pointerdown", pointerdown);
            element.addEventListener("pointercancel", pointerup);
            element.addEventListener("pointerup", pointerup);
            element.addEventListener("pointermove", pointermove);
            element.addEventListener("wheel", wheel);

            return () => {
                element.removeEventListener("pointerdown", pointerdown);
                element.removeEventListener("pointercancel", pointerup);
                element.removeEventListener("pointerup", pointerup);
                element.removeEventListener("pointermove", pointermove);
                element.removeEventListener("wheel", wheel);
            };
        }
    }, [ref]);

    const padding = 300;
    let cx = {
        minX: Math.min(...connections.map((v) => Math.min(v.from[0], v.to[0]))) - padding,
        minY: Math.min(...connections.map((v) => Math.min(v.from[1], v.to[1]))) - padding,
        maxX: Math.max(...connections.map((v) => Math.max(v.from[0], v.to[0]))) + padding,
        maxY: Math.max(...connections.map((v) => Math.max(v.from[1], v.to[1]))) + padding,
        width: 0,
        height: 0,
    };
    cx.width = cx.maxX - cx.minX;
    cx.height = cx.maxY - cx.minY;

    return (
        <div
            className={`viewport ${className}`}
            {...props}
            ref={(e) => {
                ref.current = e;
                if (typeof propsRef === "function") propsRef(e);
                else if (propsRef) propsRef.current = e;
            }}
        >
            <div
                className="pane"
                style={
                    {
                        backgroundPosition: `${position[0]}px ${position[1]}px`,
                        "--scale": `${Math.max(0.25, scale)}`,
                    } as CSSProperties
                }
            >
                <div
                    className="nodes"
                    style={{
                        transform: `translate(${position[0]}px, ${position[1]}px) scale(${scale})`,
                    }}
                >
                    <svg
                        className="connections"
                        style={{
                            left: cx.minX + "px",
                            top: cx.minY + "px",
                            width: cx.width + "px",
                            height: cx.height + "px",
                        }}
                        viewBox={`${cx.minX} ${cx.minY} ${cx.width} ${cx.height}`}
                    >
                        {connections.map((c, i) => {
                            const curviness = Math.abs(c.from[0] - c.to[0] + 40) / 2;
                            const padding = 10;
                            const path =
                                `M${c.from[0]},${c.from[1]}` +
                                `h${padding}` +
                                `C${c.from[0] + padding + curviness},${c.from[1]},${c.to[0] - curviness - padding},${
                                    c.to[1]
                                },${c.to[0] - padding},${c.to[1]}` +
                                `h${padding}`;
                            return (
                                <>
                                    <path key={"connection-" + i} d={path} stroke="white" strokeWidth={2} fill="none" />
                                    <path
                                        key={"select-connection-" + i}
                                        d={path}
                                        strokeWidth={12}
                                        stroke="transparent"
                                        fill="none"
                                    />
                                </>
                            );
                        })}
                    </svg>
                    {children}
                </div>
            </div>
        </div>
    );
});
