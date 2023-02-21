export type WebSocketState = "disconnected" | "connecting" | "connected" | "waiting";
export class WebSocketWrapper {
    ws: WebSocket | null = null;
    url: string = "";
    state: WebSocketState = "disconnected";

    connect(url: string) {
        this.url = url;
        this.ws = new WebSocket(url);
        this.ws.onerror = (event) => console.error("The websocket encountered an error:", event);
        this.ws.onclose;
    }

    disconnect() {
        if (this.ws !== null) {
            this.ws.close();
            this.ws = null;
        }
    }

    _reconnect = () => {
        this.ws = new WebSocket(this.url);
        this.ws.onopen = () => {
            this.state = "connected";
        };
        this.ws.onmessage = console.log;
        this.ws.onclose = (event) => {
            switch (this.state) {
                case "disconnected":
                case "waiting":
                    console.error("There shouldn't be any open websocket to close");
                    break;
                case "connecting":
                    console.info("Failed to connect. Retry in 10s");
                    this.state = "waiting";
                    setTimeout(this._reconnect, 10000);
                    break;
                case "connected":
                    if (event.wasClean) console.info("Websocket has been closed cleanly", event.reason);
                    else console.error("Websocket lost connection", event.reason);
                    this._reconnect();
                    break;
            }
            this.ws = null;
        };
    };
}

const WS = new WebSocketWrapper();
export default WS;
