import React from "react";
import "../styling/background.css";

export default function Background() {
    const canvas = React.useRef<HTMLCanvasElement | null>(null);
    const ctx = React.useRef<CanvasRenderingContext2D | null | undefined>(undefined);
    const columns = React.useRef<Array<number>>([]);
    const interval = React.useRef<number | null>(null);

    function renderCanvas() {
        if (ctx.current && canvas.current) {
            ctx.current.fillStyle = "#0001";
            ctx.current.fillRect(0, 0, canvas.current.width, canvas.current.height);

            ctx.current.fillStyle = "#222";
            ctx.current.font = "1em monospace";

            columns.current.forEach((y, ind) => {
                const text = String.fromCharCode(Math.floor(Math.random() * 96 + 32));
                const x = ind * 12;
                ctx.current?.fillText(text, x, y);
                if (y > 100 + Math.random() * 10000) {
                    columns.current[ind] = 0;
                } else {
                    columns.current[ind] = y + 12;
                }
            });
        }
    }

    function updateMatrix() {
        if (ctx.current && canvas.current) {
            const w = (canvas.current.width = document.body.offsetWidth);
            const h = (canvas.current.height = document.body.offsetHeight);
            columns.current = Array.from(
                { length: Math.floor(w / 12) + 1 },
                () => Math.floor((Math.random() * h) / 12) * 12,
            );
        }
    }

    React.useEffect(() => {
        interval.current = setInterval(() => {
            window.requestAnimationFrame(renderCanvas);
        }, 50);
        updateMatrix();
        window.addEventListener("resize", updateMatrix);
    }, []);

    React.useEffect(
        () => () => {
            interval.current && clearInterval(interval.current);
            interval.current = null;
        },
        [],
    );

    return (
        <canvas
            className="background"
            ref={(c) => {
                ctx.current = c?.getContext("2d");
                canvas.current = c;
            }}
        />
    );
}
