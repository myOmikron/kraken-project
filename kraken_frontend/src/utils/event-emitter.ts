import CONSOLE from "./console";

/** An event handler is any function taking an event */
export type EventHandler<Event> = (event: Event) => void;

/** Handle returned by {@link EventEmitter#addEventListener `addEventListener`} to identify the added listener */
export type ListenerHandle<K> = [K, number];

/**
 * Base class for things which emit events others can listen for
 *
 * The generic `Events` should be an object mapping event types to their events' data:
 * ```ts
 * type Events = {
 *     // event of type "foo" consists of a number
 *     foo: number;
 *     // event of type "bar" consists of a string
 *     bar: string;
 *     // event of type "baz" consists of a custom event object
 *     baz: BazEvent;
 * };
 * ```
 */
export default class EventEmitter<Events extends {}> {
    /** Map from event type to group of listeners */
    listeners: { [K in keyof Events]?: ListenerGroup<Events[K]> } = {};

    /**
     * Emit an event invoking all listeners for the event's type
     *
     * @param type the type of event whose listeners to invoke
     * @param event the event's data to invoke the listeners with
     */
    emitEvent<K extends keyof Events>(type: K, event: Events[K]) {
        const group = this.listeners[type];
        if (group !== undefined) {
            group.emit(event);
        }
    }

    /**
     * Append a new event listener for a specific event type
     *
     * @param type the type of event (identified by a string) to listen for
     * @param listener the callback function to invoke when the event is emitted
     * @returns a handle which can be used to remove the appended event listener via {@link EventEmitter#removeEventListener `removeEventListener`}
     */
    addEventListener<K extends keyof Events>(type: K, listener: EventHandler<Events[K]>): ListenerHandle<K> {
        let group: ListenerGroup<Events[K]> | undefined = this.listeners[type];
        if (group === undefined) {
            group = new ListenerGroup<Events[K]>();
            this.listeners[type] = group;
        }
        return [type, group.add(listener)];
    }

    /**
     * Remove an existing event listener identified by the handle returned from {@link EventEmitter#addEventListener `addEventListener`}
     *
     * @param handle returned from {@link EventEmitter#addEventListener `addEventListener`} to identify the event listener
     */
    removeEventListener<K extends keyof Events>(handle: ListenerHandle<K>) {
        const [type, id] = handle;
        const group = this.listeners[type];
        if (group !== undefined) group.remove(id);
    }
}

/** Internal class managing the listeners for a single event type */
class ListenerGroup<E> {
    /** Simple counter to generate unique ids */
    nextId: number = 0;

    /** Map from id to event handler */
    listener: Map<number, EventHandler<E>> = new Map();

    /**
     * Emit an event
     *
     * @param event data to call all current listeners with
     */
    emit(event: E) {
        for (const perm of this.listener.values()) {
            try {
                perm(event);
            } catch (error) {
                CONSOLE.error("Error inside event listener:", error);
            }
        }
    }

    /**
     * Add an event listener
     *
     * @param eventListener function to be called when the event is emitted
     * @returns id used to remove the event listener
     */
    add(eventListener: EventHandler<E>): number {
        const id = this.nextId;
        this.listener.set(id, eventListener);
        this.nextId += 1;
        return id;
    }

    /**
     * Remove an event listener
     *
     * @param id id obtained when adding the event listener
     */
    remove(id: number) {
        this.listener.delete(id);
    }
}
