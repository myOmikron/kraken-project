import CONSOLE from "../utils/console";
import EventEmitter from "../utils/event-emitter";
import { WsClientMessage, WsClientMessageToJSON, WsMessage, WsMessageFromJSON } from "./generated";

/**
 * Declaration of all events the {@link WebSocketWrapper} exposes:
 *
 * - **`"message"`**: received any websocket message
 * - **`"message.<type>"`**: received websocket message of type `"<type>"`
 * - **`"state"`**: changed websocket's connection state
 * - **`"state.<state>"`**: changed websocket's connection state to `"<state"`
 *
 * ```ts
 * WS.addEventListener("message.InvitationToWorkspace", ({from}) => {
 *     toast.info(`${form.displayName} invited you to one of his workspaces`);
 * });
 * WS.addEventListener("state.connected", () => {
 *     toast.info("Websocket has connected");
 * });
 * ```
 */
export type WebSocketEvents = {
    [Event in WsMessage as `message.${Event["type"]}`]: Event;
} & {
    [State in WebSocketState as `state.${State}`]: void;
} & {
    /** A {@link WsMessage} has been received*/
    message: WsMessage;

    /** The websocket's connection state changed */
    state: WebSocketState;
};

/**
 * The websocket's connection state:
 * - **disconnected**: the socket is completely inactive and waits for someone to call {@link WebSocketWrapper.connect `connect`}
 * - **connecting**: the socket is currently trying to connect
 * - **connected**: the socket is connected and ready for operation
 * - **waiting**: the socket failed to connect and is waiting before retrying later
 */
export type WebSocketState = "disconnected" | "connecting" | "connected" | "waiting";

/**
 * Wrapper around the browser's {@link WebSocket `WebSocket`} which implements auto reconnecting and a high-level event interface.
 *
 * Use {@link EventEmitter.addEventListener `addEventListener`} and {@link EventEmitter.removeEventListener `removeEventListener`} to listen for {@link WebSocketEvents events}.
 */
export class WebSocketWrapper extends EventEmitter<WebSocketEvents> {
    private ws: WebSocket | null = null;
    private timeout: number | null = null;
    private url: string = "";
    private _state: WebSocketState = "disconnected";

    /** Open a new connection discarding any previous one */
    connect(url: string) {
        this.url = url;
        this.clearOld();
        this.reconnect();
    }

    send(msg: WsClientMessage) {
        if (this.ws !== null && this.state === "connected") {
            this.ws.send(JSON.stringify(WsClientMessageToJSON(msg)));
        }
    }

    /** Explicitly discards the connection without opening a new one */
    disconnect() {
        this.clearOld();
        this.state = "disconnected";
    }

    /** Sets `ws = null` and handles the old websocket (if any) */
    private clearOld() {
        if (this.timeout !== null) {
            clearTimeout(this.timeout);
            this.timeout = null;
        }
        if (this.ws !== null) {
            if (this.state === "connected") {
                this.ws.close();
            } else {
                this.ws.onopen = function () {
                    this.close();
                };
            }
            this.ws.onmessage = null;
            this.ws.onclose = null;
            this.ws = null;
        }
    }

    private reconnect = () => {
        this.state = "connecting";
        this.ws = new WebSocket(this.url);
        this.ws.onopen = () => {
            this.state = "connected";
        };
        this.ws.onmessage = (event) => {
            if (typeof event.data !== "string") {
                CONSOLE.error("Received a non string websocket message: ", event.data);
            } else {
                try {
                    const message = WsMessageFromJSON(JSON.parse(event.data));
                    this.emitEvent(`message`, message);
                    this.emitEvent(`message.${message.type}`, message);
                } catch (e) {
                    if (e instanceof SyntaxError) CONSOLE.error("Received a non json websocket message: ", event.data);
                    else CONSOLE.error("Received malformed json websocket message: ", JSON.parse(event.data));
                }
            }
        };
        this.ws.onclose = (event) => {
            switch (this.state) {
                case "disconnected":
                case "waiting":
                    CONSOLE.error("There shouldn't be any open websocket to close");
                    break;
                case "connecting":
                    CONSOLE.info("Failed to connect. Retry in 10s");
                    this.state = "waiting";
                    this.timeout = setTimeout(this.reconnect, 10000);
                    break;
                case "connected":
                    if (event.wasClean) CONSOLE.info("Websocket has been closed cleanly", event.reason);
                    else CONSOLE.error("Websocket lost connection", event.reason);
                    this.reconnect();
                    break;
            }
            this.ws = null;
        };
    };

    get state() {
        return this._state;
    }

    private set state(value: WebSocketState) {
        this._state = value;
        this.emitEvent(`state`, value);
        this.emitEvent(`state.${value}`, undefined);
    }
}

/** The global websocket singleton */
const WS = new WebSocketWrapper();
export default WS;
