import React from "react";
import "../styling/settings.css";
import Input from "../components/input";
import { Api } from "../api/api";
import { toast } from "react-toastify";
import { SettingsFull } from "../api/generated";

type SettingsProps = {};
type SettingsState = {
    settings: SettingsFull | null;
};

export default class Settings extends React.Component<SettingsProps, SettingsState> {
    constructor(props: SettingsProps) {
        super(props);

        this.state = {
            settings: null,
        };
    }

    componentDidMount() {
        Api.admin.settings.get().then((res) => {
            res.match(
                (settings) => this.setState({ settings }),
                (err) => toast.error(err.message)
            );
        });
    }

    async saveSettings() {
        let { settings } = this.state;

        if (settings === null) {
            return;
        }

        (await Api.admin.settings.update(settings)).match(
            (_) => toast.success("Settings updated"),
            (err) => toast.error(err.message)
        );
    }

    render() {
        return (
            <div className={"settings-container"}>
                <div className={"settings-heading pane"}>
                    <h1 className={"heading"}>Kraken Settings</h1>
                </div>
                {this.state.settings !== null ? (
                    <form
                        className={"settings-dehashed pane"}
                        method={"post"}
                        onSubmit={async (x) => {
                            x.preventDefault();
                            await this.saveSettings();
                        }}
                    >
                        <h2 className={"heading"}>Dehashed API</h2>
                        <Input
                            value={
                                this.state.settings.dehashedEmail !== null &&
                                this.state.settings.dehashedEmail !== undefined
                                    ? this.state.settings.dehashedEmail
                                    : ""
                            }
                            onChange={(v) => {
                                let { settings } = this.state;
                                if (settings !== null) {
                                    settings.dehashedEmail = v;
                                }
                                this.setState({ settings });
                            }}
                            placeholder={"email"}
                        />
                        <Input
                            value={
                                this.state.settings.dehashedApiKey !== null &&
                                this.state.settings.dehashedApiKey !== undefined
                                    ? this.state.settings.dehashedApiKey
                                    : ""
                            }
                            onChange={(v) => {
                                let { settings } = this.state;
                                if (settings !== null) {
                                    settings.dehashedApiKey = v;
                                }
                                this.setState({ settings });
                            }}
                            placeholder={"api-key"}
                        />
                        <button className={"button"}>Save</button>
                    </form>
                ) : (
                    <div className={"pane"}>Loading ...</div>
                )}
            </div>
        );
    }
}
