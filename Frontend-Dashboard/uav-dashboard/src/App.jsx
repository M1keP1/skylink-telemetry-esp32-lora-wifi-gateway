// src/App.jsx
import "./App.css";
import { useState } from "react";
import Pfd from "./components/pfd.jsx";
import TelemetryPanel from "./components/TelemetryPanel.jsx";
import MapPanel from "./components/MapPanel.jsx";
import FlightStatusOverlay from "./components/FlightStatusOverlay.jsx";
import useTelemetry from "./hooks/useTelemetry";

export default function App() {
    const [isConnected, setIsConnected] = useState(false);
    const telemetry = useTelemetry(isConnected ? "ws://localhost:9091/ws/stream" : null);

    return (
        <div className="app">
            <aside className="left">
                <div className="panel top" style={{ padding: 0 }}>
                    <Pfd
                        rollDeg={telemetry?.roll ?? 0}
                        pitchDeg={telemetry?.pitch ?? 0}
                        headingDeg={telemetry?.heading ?? telemetry?.yaw ?? 0}
                        speedMps={telemetry?.ground_speed ?? 0}
                        altM={telemetry?.altitude_baro ?? 0}
                        climbMps={telemetry?.vertical_speed ?? 0}
                        width={600}
                        height={500}
                    />
                </div>

                <div className="panel bottom">
                    <TelemetryPanel telemetry={telemetry} />
                </div>
            </aside>

            <main className="right">
                <FlightStatusOverlay telemetry={telemetry} />
                <button
                    className={`map-connect-btn ${!isConnected ? "disconnected" : ""}`}
                    onClick={() => setIsConnected(!isConnected)}
                >
                    {isConnected ? "Disconnect" : "Connect"}
                </button>
                <MapPanel telemetry={telemetry} />
            </main>
        </div>
    );
}