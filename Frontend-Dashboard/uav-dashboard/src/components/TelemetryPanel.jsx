const demo = [
    { label: "Distance", value: "-- km" },
    { label: "Speed", value: "-- m/s" },
    { label: "Battery", value: "-- %" },
    { label: "Consumption", value: "-- mAh" },
];

export default function TelemetryPanel() {
    return (
        <div>
            <h3 style={{ margin: "0 0 8px 0" }}>Telemetry</h3>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
                {demo.map((m) => (
                    <div
                        key={m.label}
                        style={{
                            background: "#0c1526",
                            border: "1px solid #1c2a44",
                            borderRadius: 10,
                            padding: 10,
                        }}
                    >
                        <div style={{ opacity: 0.8, fontSize: 12 }}>{m.label}</div>
                        <div style={{ fontSize: 22, fontWeight: 700 }}>{m.value}</div>
                    </div>
                ))}
            </div>
        </div>
    );
}