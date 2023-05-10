import Input from "./input";

type EditableListProps = {
    value: Array<string>;
    onChange: (newValue: Array<string>) => any;
};

export default function EditableList(props: EditableListProps) {
    const { value, onChange } = props;
    return (
        <div className="editable-list">
            <div className="neon" onClick={() => onChange([...value, ""])}>
                <span>New Entry</span>
            </div>
            {value.map((v, i) => (
                <div key={i}>
                    <Input
                        value={v}
                        onChange={(newV) => {
                            const newValue = [...value];
                            newValue[i] = newV;
                            onChange(newValue);
                        }}
                    />
                    <div
                        className="neon"
                        onClick={() => {
                            const newValue = [...value];
                            newValue.splice(i, 1);
                            onChange(newValue);
                        }}
                    >
                        <span>Remove Entry</span>
                    </div>
                </div>
            ))}
        </div>
    );
}
