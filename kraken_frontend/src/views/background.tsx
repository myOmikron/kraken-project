import React from "react";

export default class Background extends React.Component<any, any> {
    canvas: HTMLCanvasElement | null | undefined;
    ctx: CanvasRenderingContext2D | null | undefined;
    columns: Array<number> = [];
    // @ts-ignore
    interval: NodeJS.Timer | null = null;

    renderCanvas() {
        if (this.ctx && this.canvas) {
            this.ctx.fillStyle = "#0001";
            this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

            this.ctx.fillStyle = "#222";
            this.ctx.font = "1em monospace";

            this.columns.forEach((y, ind) => {
                const text = String.fromCharCode(Math.floor(Math.random() * 96 + 32));
                const x = ind * 12;
                this.ctx?.fillText(text, x, y);
                if (y > 100 + Math.random() * 10000) {
                    this.columns[ind] = 0;
                } else {
                    this.columns[ind] = y + 12;
                }
            });
        }
    }

    updateMatrix() {
        if (this.ctx && this.canvas) {
            const w = (this.canvas.width = document.body.offsetWidth);
            const h = (this.canvas.height = document.body.offsetHeight);
            this.columns = Array.from(
                { length: Math.floor(w / 12) + 1 },
                () => Math.floor((Math.random() * h) / 12) * 12
            );
        }
    }

    componentDidMount() {
        this.interval = setInterval(() => {
            window.requestAnimationFrame(this.renderCanvas.bind(this));
        }, 50);
        this.updateMatrix();
        window.addEventListener("resize", this.updateMatrix.bind(this));
    }

    componentWillUnmount() {
        this.interval && clearInterval(this.interval);
        this.interval = null;
    }

    render() {
        return (
            <canvas
                className="background"
                ref={(c) => {
                    this.ctx = c?.getContext("2d");
                    this.canvas = c;
                }}
            />
        );
    }
}
