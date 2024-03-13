import React, {
    Dispatch,
    ElementType,
    PropsWithChildren,
    ReactNode,
    useCallback,
    useEffect,
    useRef,
    useState,
} from "react";
import Popup from "reactjs-popup";
import { PopupActions } from "reactjs-popup/dist/types";
import "../../../styling/context-menu.css";

type ClickHandlerArgs = { ctrlKey: boolean; altKey: boolean };
type ClickHandler = (e: ClickHandlerArgs) => any;

export type PlainMenuItem = [ReactNode | "pending" | undefined, ClickHandler | undefined];
export type GroupedMenuItem = PlainMenuItem | { icon?: ReactNode; group: string; items: ContextMenuEntry[] };
export type LazyMenuItem = () => Promise<ContextMenuEntry[]>;
export type ContextMenuEntry = PlainMenuItem | GroupedMenuItem | LazyMenuItem;

export type ContextMenuProps<E extends ElementType = "div"> = PropsWithChildren<{
    as?: E;
    menu: ContextMenuEntry[];
    onOpen?: Function;
}> &
    React.ComponentPropsWithoutRef<E>;

export default function ContextMenu<E extends ElementType = "div">({
    as,
    menu,
    onOpen,
    children,
    ...props
}: ContextMenuProps<E>) {
    const Component = as ?? "div";
    const popupRef = useRef<PopupActions>(null);
    const [keyboardNav, setKeyboardNav] = useState(false);
    const componentRef = useRef<any>();
    const [open, setOpen] = useState(false);
    const [submenuOpen, setSubmenuOpen] = useState(0);

    if ("className" in props) (props as any).className += " context-menu-wrapper";
    else (props as any).className = "context-menu-wrapper";

    const clickHandler = useCallback(
        (handler: ClickHandler, e: React.MouseEvent<HTMLLIElement>) => {
            e.preventDefault();
            if (!e.shiftKey) popupRef.current?.close();
            return handler(e);
        },
        [menu, popupRef],
    );

    const keyDownHandler = useCallback(
        (handler: ClickHandler, e: React.KeyboardEvent<HTMLLIElement>) => {
            if (e.key == "Enter") {
                e.preventDefault();
                if (!e.shiftKey) popupRef.current?.close();
                return handler(e);
            }
        },
        [clickHandler],
    );

    const keyboardNavCb = useCallback((e: React.KeyboardEvent<HTMLUListElement>) => {
        let selected = document.activeElement;
        while (selected && selected.tagName != "LI") selected = selected.parentElement;
        if (!selected) return;
        if (e.key == "ArrowDown") {
            setKeyboardNav(true);
            selected = selected.nextElementSibling;
            if (selected && "focus" in selected && typeof selected.focus == "function") selected.focus();
        } else if (e.key == "ArrowUp") {
            setKeyboardNav(true);
            selected = selected.previousElementSibling;
            if (selected && "focus" in selected && typeof selected.focus == "function") selected.focus();
        } else if (e.key == "Tab") {
            setKeyboardNav(true);
        }
    }, []);

    const openHandler = () => {
        setOpen(true);
        onOpen?.();
    };

    const closeHandler = () => {
        if (submenuOpen > 0) return;
        setOpen(false);
        // restore focus:
        let c = componentRef.current;
        if (typeof c == "object" && "focus" in c && typeof c.focus == "function") c.focus();
    };

    const onContextMenu = (event: React.MouseEvent | React.KeyboardEvent) => {
        event.preventDefault();
        componentRef.current = event.target as any;
        popupRef.current?.toggle?.();
    };

    return (
        <Popup
            ref={popupRef}
            on={[]}
            position={["bottom center", "top center"]}
            open={open}
            onOpen={openHandler}
            onClose={closeHandler}
            trigger={
                <Component
                    onKeyDown={(e) =>
                        /* various shortcuts since we don't have many, people
                         * might come up and memorize with them in different
                         * ways, e.g. `[e]dit`, `[f]ilter`, `[c]ontext` */
                        !e.ctrlKey &&
                        !e.altKey &&
                        !e.shiftKey &&
                        (e.key == "e" || e.key == "f" || e.key == "c" || e.key == "ContextMenu") &&
                        onContextMenu(e)
                    }
                    onContextMenu={onContextMenu}
                    tabIndex={0}
                    {...props}
                >
                    {children}
                </Component>
            }
            keepTooltipInside
            arrow
            nested
        >
            <ul
                onKeyDown={keyboardNavCb}
                role="menu"
                className={`pane-thin context-menu ${keyboardNav ? "keyboard-nav" : ""}`}
            >
                {menu.map((v, i) => (
                    <ContextMenuEntryRenderer
                        key={i}
                        v={v}
                        autoFocus={i == 0}
                        open={open}
                        onClick={clickHandler}
                        onKeyDown={keyDownHandler}
                        useMouseNav={() => keyboardNav && setKeyboardNav(false)}
                        setSubmenuOpen={setSubmenuOpen}
                    />
                ))}
            </ul>
        </Popup>
    );
}

function ContextMenuEntryRenderer(props: {
    v: ContextMenuEntry;
    autoFocus?: boolean;
    open: boolean;
    onClick: (handler: ClickHandler, e: React.MouseEvent<HTMLLIElement>) => any;
    onKeyDown: (handler: ClickHandler, e: React.KeyboardEvent<HTMLLIElement>) => any;
    useMouseNav: () => any;
    setSubmenuOpen: Dispatch<React.SetStateAction<number>>;
}) {
    return Array.isArray(props.v) ? (
        <MenuItemRenderer
            item={props.v}
            autoFocus={props.autoFocus}
            onClick={props.onClick}
            onKeyDown={props.onKeyDown}
            useMouseNav={props.useMouseNav}
        />
    ) : typeof props.v === "object" ? (
        <ContextSubMenu
            group={
                <>
                    {props.v.icon}
                    {props.v.group}
                </>
            }
            items={props.v.items}
            autoFocus={props.autoFocus}
            onClick={props.onClick}
            onKeyDown={props.onKeyDown}
            useMouseNav={props.useMouseNav}
            setSubmenuOpen={props.setSubmenuOpen}
        />
    ) : (
        <LazyMenuItemRenderer
            item={props.v}
            open={props.open}
            onClick={props.onClick}
            onKeyDown={props.onKeyDown}
            useMouseNav={props.useMouseNav}
            setSubmenuOpen={props.setSubmenuOpen}
        />
    );
}

function ContextSubMenu(
    props: {
        group: ReactNode;
        items: ContextMenuEntry[];
        setSubmenuOpen: Dispatch<React.SetStateAction<number>>;
    } & Omit<MenuItemRendererProps, "item">,
) {
    const popupRef = useRef(null);
    const liRefHack = useRef<HTMLElement>(null); // needed because popup doesn't expose the trigger element's ref and breaks if we try to assign it
    const [open, setOpen] = useState(false);

    const openHandler = () => {
        setOpen((open) => {
            if (!open) {
                props.setSubmenuOpen((v) => ++v);
            }
            return true;
        });
    };

    const closeHandler = () => {
        setOpen((open) => {
            if (open) {
                props.setSubmenuOpen((v) => --v);
            }
            return false;
        });
        let li = liRefHack.current;
        while (li && li.tagName != "LI") li = li.parentElement;
        if (li) li.focus();
    };

    return (
        <Popup
            ref={popupRef}
            on={["hover", "click"]}
            position={["right top", "right bottom", "right center", "left top", "left bottom", "left center"]}
            open={open}
            onOpen={openHandler}
            onClose={closeHandler}
            trigger={(open) => (
                <li
                    className={`group ${open ? "open" : ""}`}
                    tabIndex={0}
                    role="menuitem"
                    onMouseOver={(e) => props.useMouseNav()}
                    onKeyDown={(e) => (e.key == "ArrowRight" || e.key == "Enter") && setOpen(true)}
                >
                    {props.group}
                    <span style={{ display: "none" }} ref={liRefHack}></span>
                </li>
            )}
            keepTooltipInside
            arrow={false}
            nested
        >
            <ul role="menu" className={`pane-thin context-menu sub-menu`}>
                {props.items.map((v, i) => (
                    <ContextMenuEntryRenderer
                        key={i}
                        v={v}
                        open={open}
                        onClick={props.onClick}
                        onKeyDown={(handler, e) =>
                            e.key == "ArrowLeft" ? closeHandler() : props.onKeyDown(handler, e)
                        }
                        useMouseNav={props.useMouseNav}
                        setSubmenuOpen={props.setSubmenuOpen}
                    />
                ))}
            </ul>
        </Popup>
    );
}

type MenuItemRendererProps = {
    item: PlainMenuItem | undefined;
    autoFocus?: boolean;
    onClick: (handler: ClickHandler, e: React.MouseEvent<HTMLLIElement>) => any;
    onKeyDown: (handler: ClickHandler, e: React.KeyboardEvent<HTMLLIElement>) => any;
    useMouseNav: () => any;
};

function MenuItemRenderer({ item, autoFocus, onClick, onKeyDown, useMouseNav }: MenuItemRendererProps) {
    const ref = useRef<HTMLLIElement>(null);
    useEffect(() => {
        if (autoFocus && ref.current) {
            let e = ref.current;
            requestAnimationFrame(() => e.focus());
        }
    }, [item]);
    return item === undefined ? (
        <li
            key={0}
            ref={ref}
            className={`disabled`}
            aria-disabled
            tabIndex={0}
            autoFocus={autoFocus}
            role="menuitem"
            onMouseOver={(e) => useMouseNav()}
        >
            Loading...
        </li>
    ) : (
        <li
            key={0}
            ref={ref}
            className={`${item[1] === undefined ? "disabled" : ""}`}
            aria-disabled={item[1] === undefined}
            tabIndex={0}
            autoFocus={autoFocus}
            role="menuitem"
            onClick={(e) => item[1] && onClick(item[1], e)}
            onKeyDown={(e) => item[1] && onKeyDown(item[1], e)}
            onMouseOver={(e) => useMouseNav()}
        >
            {item[0]}
        </li>
    );
}

function LazyMenuItemRenderer({
    item,
    open,
    autoFocus,
    setSubmenuOpen,
    ...rest
}: {
    setSubmenuOpen: Dispatch<React.SetStateAction<number>>;
    item: LazyMenuItem;
    open: boolean;
} & Omit<MenuItemRendererProps, "item">) {
    const items = useLazyMenuItems(item, open);
    return items === undefined ? (
        <MenuItemRenderer key={-1} item={undefined} {...rest} />
    ) : (
        <>
            {items.map(
                (e, i) =>
                    e && (
                        <ContextMenuEntryRenderer
                            setSubmenuOpen={setSubmenuOpen}
                            key={i}
                            v={e}
                            open={open}
                            {...rest}
                            autoFocus={autoFocus && i == 0}
                        />
                    ),
            )}
        </>
    );
}

export function useLazyMenuItems(
    items: () => Promise<ContextMenuEntry[]> | ContextMenuEntry[],
    open: boolean,
): ContextMenuEntry[] | undefined {
    let [result, setResult] = useState<ContextMenuEntry[] | undefined>(undefined);
    let [fn] = useState(items());
    useEffect(() => {
        if (result === undefined && open) {
            (async function () {
                let v = await fn;
                setResult(v);
            })();
        }
    }, [fn, items, open]);
    return result;
}
