import React, { ChangeEvent, forwardRef } from "react";
import "../../../styling/logfile-input.css";
import { toast } from "react-toastify";
import CloseIcon from "../../../svg/close";

export type LogFile = {
    file: File;
    dataURL: string;
};

export type LogFileInputProps = {
    logfile: LogFile | undefined;
    onChange: React.Dispatch<LogFile | undefined>;
    affected?: boolean;
} & Omit<React.HTMLProps<HTMLDivElement>, "onChange">;

export const LogFileInput = forwardRef<HTMLDivElement, LogFileInputProps>(
    ({ logfile, onChange, affected, ...props }, ref) => {
        const fileHandler = (e: ChangeEvent<HTMLInputElement>) => {
            const fileUploadInput = e.target;

            if (!fileUploadInput) {
                return toast.error("No file input element found");
            }

            // @ts-ignore
            const f = fileUploadInput.files[0];

            if (!f) {
                return toast.error("No file selected");
            }

            if (f.size > 100 * 1024 * 1024) {
                return toast.error("File too large");
            }

            let url = URL.createObjectURL(f);
            onChange({ file: f, dataURL: url });
        };

        return (
            <div {...props} className={`logfile-input ${props.className}`}>
                {affected ? (
                    <>
                        <div className="content">
                            {props.children}
                            {logfile ? (
                                <>
                                    <button
                                        type={"button"}
                                        title="Remove file"
                                        className="remove"
                                        onClick={() => {
                                            onChange(undefined);
                                        }}
                                    >
                                        <CloseIcon />
                                    </button>
                                    <a download={logfile.file.name} href={logfile.dataURL}>
                                        <span>{logfile.file.name}</span>
                                    </a>
                                </>
                            ) : (
                                <>
                                    <input id="upload2" type="file" onChange={fileHandler} />
                                    <span>Upload Attachment</span>
                                </>
                            )}
                        </div>
                    </>
                ) : (
                    <>
                        <div>
                            <label className="button" htmlFor="upload">
                                Upload
                            </label>
                            <input className="input" id="upload" type="file" onChange={fileHandler} />
                        </div>
                        {logfile ? (
                            <div className="create-finding-file-grid">
                                <button
                                    type={"button"}
                                    title="Remove file"
                                    className="remove"
                                    onClick={() => {
                                        onChange(undefined);
                                    }}
                                >
                                    <CloseIcon />
                                </button>
                                <a download={logfile.file.name} href={logfile.dataURL}>
                                    <span>{logfile.file.name}</span>
                                </a>
                            </div>
                        ) : undefined}{" "}
                    </>
                )}
            </div>
        );
    },
);
