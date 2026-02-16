import { useEffect, useState, useRef } from "react";

export default function FlightStatusOverlay({ telemetry }) {
    const [elapsed, setElapsed] = useState(0);
    const [distance, setDistance] = useState(0);
    const [isActive, setIsActive] = useState(false);

    // Refs to persist across renders without triggering re-renders
    const stateRef = useRef({
        startTime: null,
        lastPos: null,
        active: false,
        encouteredFlight: false
    });

    // Timer for UI updates
    useEffect(() => {
        const timer = setInterval(() => {
            if (stateRef.current.active && stateRef.current.startTime) {
                setElapsed(Date.now() - stateRef.current.startTime);
            }
        }, 1000);
        return () => clearInterval(timer);
    }, []);

    // Telemetry logic
    useEffect(() => {
        if (!telemetry) return;
        // Debugging: Check keys
        console.log("Overlay Telemetry:", telemetry);

        const phase = telemetry.flight_phase || "Unknown";
        // Phases that count as "flying"
        const flyingPhases = ["Taking Off", "Ascent", "Cruise", "Descent", "Landing"];
        const isFlying = flyingPhases.includes(phase);

        const lat = Number(telemetry.latitude);
        const lon = Number(telemetry.longitude);
        const validPos = Number.isFinite(lat) && Number.isFinite(lon);

        // Detect Flight Start
        if (isFlying && !stateRef.current.active) {
            // If we haven't started a flight yet in this session, or we just took off
            if (!stateRef.current.encouteredFlight) {
                stateRef.current.startTime = Date.now();
                stateRef.current.encouteredFlight = true;
                setDistance(0);
            }
            stateRef.current.active = true;
            setIsActive(true);
        }

        // Detect Flight End (Landed)
        if (!isFlying && stateRef.current.active) {
            stateRef.current.active = false;
            setIsActive(false);
        }

        // Distance Calculation
        if (stateRef.current.active && validPos) {
            if (stateRef.current.lastPos) {
                const d = haversineDistance(
                    stateRef.current.lastPos.lat,
                    stateRef.current.lastPos.lon,
                    lat,
                    lon
                );
                // Accumulate if reasonable movement (> 0.5m to avoid noise)
                if (d > 0.0005) {
                    setDistance(prev => prev + d);
                }
            }
            stateRef.current.lastPos = { lat, lon };
        }
    }, [telemetry]);

    const formatTime = (ms) => {
        const totalSecs = Math.floor(ms / 1000);
        const h = Math.floor(totalSecs / 3600);
        const m = Math.floor((totalSecs % 3600) / 60);
        const s = totalSecs % 60;
        if (h > 0) return `${h}h ${m}m ${s}s`;
        return `${m}m ${s}s`;
    };

    const formatDist = (km) => {
        if (km < 1) return `${(km * 1000).toFixed(0)} m`;
        return `${km.toFixed(2)} km`;
    };

    const phase = telemetry?.flight_phase || "Disconnected";

    return (
        <div className="flight-status-overlay">
            <div className="status-row">
                <span className="label">STATUS</span>
                <span className={`value status-${phase.toLowerCase().replace(/\s/g, '-')}`}>
                    {phase.toUpperCase()}
                </span>
            </div>

            {/* Debug helper: Remove after verifying */}
            {(phase === "Disconnected" || phase === "Unknown") && telemetry && (
                <div style={{ fontSize: '9px', color: '#ff3c3c', maxWidth: '150px', wordBreak: 'break-all' }}>
                    Keys: {Object.keys(telemetry).join(', ')}
                </div>
            )}

            {!telemetry && (
                <div style={{ fontSize: '9px', color: '#666' }}>No Data Stream</div>
            )}

            <div className="stats-grid">
                <div className="stat-item">
                    <span className="label">TIME</span>
                    <span className="value">{formatTime(elapsed)}</span>
                </div>
                <div className="stat-item">
                    <span className="label">DIST</span>
                    <span className="value">{formatDist(distance)}</span>
                </div>
            </div>
        </div>
    );
}

// Haversine formula to close enough approximation for small distances
function haversineDistance(lat1, lon1, lat2, lon2) {
    const R = 6371; // km
    const dLat = toRad(lat2 - lat1);
    const dLon = toRad(lon2 - lon1);
    const a =
        Math.sin(dLat / 2) * Math.sin(dLat / 2) +
        Math.cos(toRad(lat1)) * Math.cos(toRad(lat2)) *
        Math.sin(dLon / 2) * Math.sin(dLon / 2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    return R * c;
}

function toRad(Value) {
    return Value * Math.PI / 180;
}
