import EventEmitter from "../utils/event-emitter";
import { WsMessage, WsMessageFromJSON } from "./generated";

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
    ws: WebSocket | null = null;
    url: string = "";
    _state: WebSocketState = "disconnected";

    connect(url: string) {
        this.url = url;
        this._reconnect();
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
        this.ws.onmessage = (event) => {
            if (typeof event.data !== "string") {
                console.error("Received a non string websocket message: ", event.data);
            } else {
                try {
                    const message = WsMessageFromJSON(JSON.parse(event.data));
                    this.emitEvent(`message`, message);
                    this.emitEvent(`message.${message.type}`, message);
                } catch (e) {
                    if (e instanceof SyntaxError) console.error("Received a non json websocket message: ", event.data);
                    else console.error("Received malformed json websocket message: ", JSON.parse(event.data));
                }
            }
        };
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

    get state() {
        return this._state;
    }
    set state(value: WebSocketState) {
        this._state = value;
        this.emitEvent(`state`, value);
        this.emitEvent(`state.${value}`, undefined);
    }
}

/** The global websocket singleton */
const WS = new WebSocketWrapper();
export default WS;
