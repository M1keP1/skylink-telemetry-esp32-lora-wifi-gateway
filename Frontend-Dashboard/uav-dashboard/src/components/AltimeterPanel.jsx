export default function AltimeterPanel() {
    return (
        <div>
            <h3 style={{ margin: "0 0 8px 0" }}>Altimeter</h3>
            <div style={{ fontSize: 48, fontWeight: 700 }}>-- m</div>
            <div style={{ opacity: 0.8 }}>Altitude (AGL/MSL later)</div>
        </div>
    );
}