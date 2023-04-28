import React from "react";
import { ROUTES } from "../routes";

type MenuProps = {};
type MenuState = {};

export default class Menu extends React.Component<MenuProps, MenuState> {
    constructor(props: MenuProps) {
        super(props);

        this.state = {};
    }

    render() {
        return (
            <div className="menu pane">
                <div className="menu-item pane" {...ROUTES.KRAKEN_NETWORK.clickHandler({})}>
                    Kraken Network
                </div>
                <div className="menu-item pane" {...ROUTES.ME.clickHandler({})}>
                    Me
                </div>
            </div>
        );
    }
}
