import React, {
    CSSProperties,
    HTMLAttributes,
    forwardRef,
    useEffect,
    useImperativeHandle,
    useRef,
    useState,
} from "react";
import "../../../styling/viewport.css";

export type ViewportProps = {
    connections?: {
        from: [number, number];
        to: [number, number];
        className?: string;
    }[];
    originX?: number;
    originY?: number;
    /// Number from -2 to 2
    initialZoom?: number;
    /// Extra elements inserted after the pane
    decoration?: React.ReactNode;
} & HTMLAttributes<HTMLDivElement>;

export type ViewportRef = {
    getRootElement(): HTMLElement | null;
    getPosition(): [number, number];
    setPosition(position: [number, number]): void;
    getZoom(): number;
    setZoom(zoom: number): void;
    getScale(): number;
};

/**
 * Viewport component that can drag around and zoom its contents. 0,0 defaults
 * to center of screen at start.
 */
export const Viewport = forwardRef(
    (
        { connections, className, children, originX, originY, initialZoom, decoration, ...props }: ViewportProps,
        propsRef,
    ) => {
        const minZoom = -2;
        const maxZoom = 2;

        const ref = useRef<HTMLDivElement | null>(null);
        const [position, setPosition] = useState<[number, number]>([0, 0]);
        const [zoom, setZoom] = useState(initialZoom ?? 0);
        // round the whole result since otherwise the text gets blurry with more precision
        const zoomToScale = (zoom: number) => Math.round(Math.pow(2, Math.round(zoom * 8) / 8) * 100) / 100;
        const scale = zoomToScale(zoom);

        const isBackground = (e: HTMLElement | null) =>
            e ? e.classList.contains("pane") || e.classList.contains("nodes") || e == ref.current : false;

        originX ??= 0.5;
        originY ??= 0.5;

        if (connections === undefined) connections = [];

        useEffect(() => {
            if (ref.current) {
                const element = ref.current;
                let pressedPointer: number | undefined = undefined;
                const pointerdown = (e: PointerEvent) => {
                    if (pressedPointer !== undefined) return;
                    if (e.button == 1 || (e.button == 0 && isBackground(e.target as HTMLElement))) {
                        pressedPointer = e.pointerId;
                        element.setPointerCapture(e.pointerId);
                        // pointer lock can be quite disorienting for the user,
                        // especially during short movements, since the mouse
                        // "teleports back" to the drag start location (e.g.
                        // never changes position) - however the feature of
                        // dragging infinitely is quite useful, so we keep it
                        // here, behind the Alt key for power users.
                        // The spec doesn't yet support any kind of wrapping
                        // similar to what blender would offer here. See
                        // https://github.com/w3c/pointerlock/issues/73
                        if (e.altKey) element.requestPointerLock();
                    }
                };
                const pointerup = (e: PointerEvent) => {
                    if (pressedPointer !== e.pointerId) return;
                    element.releasePointerCapture(e.pointerId);
                    document.exitPointerLock();
                    pressedPointer = undefined;
                };
                const pointermove = (e: PointerEvent) => {
                    if (pressedPointer !== pressedPointer || pressedPointer === undefined) return;
                    setPosition((p) => [p[0] + e.movementX, p[1] + e.movementY]);
                };
                const wheel = (e: WheelEvent) => {
                    setZoom((zoom) => {
                        let newZoom = Math.max(minZoom, Math.min(maxZoom, zoom - e.deltaY / 480));
                        setPosition((position) => {
                            let bounds = ref.current!.getBoundingClientRect();
                            // offset by mouse cursor position
                            let ox = e.clientX - bounds.left - bounds.width * originX!;
                            let oy = e.clientY - bounds.top - bounds.height * originY!;
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

        useImperativeHandle<unknown, ViewportRef>(
            propsRef,
            () => ({
                getRootElement: () => ref.current,
                getPosition: () => position,
                setPosition: (position: [number, number]) => setPosition(position),
                getZoom: () => zoom,
                setZoom: (zoom: number) => setZoom(zoom),
                getScale: () => scale,
            }),
            [ref.current, zoom, position, scale],
        );

        return (
            <div className={`viewport ${className}`} {...props} ref={ref}>
                <div
                    className="pane"
                    style={
                        {
                            backgroundPosition: `${position[0]}px ${position[1]}px, 0 0`,
                            "--scale": `${Math.max(0.25, scale)}`,
                        } as CSSProperties
                    }
                >
                    <div
                        className="nodes"
                        style={{
                            transform: `translate(${position[0]}px, ${position[1]}px) scale(${scale})`,
                            left: `${originX * 100}%`,
                            top: `${originY * 100}%`,
                        }}
                    >
                        {connections.length != 0 && (
                            <svg
                                key={"connections"}
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
                                        <React.Fragment key={i}>
                                            <path
                                                d={path}
                                                stroke="white"
                                                strokeWidth={2}
                                                fill="none"
                                                className={`path ${c.className ?? ""}`}
                                            />
                                            <path
                                                d={path}
                                                strokeWidth={12}
                                                stroke="transparent"
                                                fill="none"
                                                className={`click-target ${c.className ?? ""}`}
                                            />
                                        </React.Fragment>
                                    );
                                })}
                            </svg>
                        )}
                        {children}
                    </div>
                </div>
                {decoration}
            </div>
        );
    },
);
