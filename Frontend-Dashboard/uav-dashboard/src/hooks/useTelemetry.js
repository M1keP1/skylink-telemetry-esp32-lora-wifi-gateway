import { useEffect, useRef, useState } from "react";

function lerp(a, b, t) {
    return a + (b - a) * t;
}
function safe(n, fallback = 0) {
    return Number.isFinite(n) ? n : fallback;
}

// optional smoothing for display (PFD feels better)
function smoothTelemetry(prev, msg) {
    if (!prev) return msg;

    return {
        ...msg,

        // attitude: slightly smoother
        roll: lerp(safe(prev.roll), safe(msg.roll), 0.25),
        pitch: lerp(safe(prev.pitch), safe(msg.pitch), 0.25),

        // speed/alt: a bit smoother
        ground_speed: lerp(safe(prev.ground_speed), safe(msg.ground_speed), 0.2),
        altitude_baro: lerp(safe(prev.altitude_baro), safe(msg.altitude_baro), 0.2),
        vertical_speed: lerp(safe(prev.vertical_speed), safe(msg.vertical_speed), 0.2),

        // heading: usually keep raw (2Hz), but you can lerp if you want
        heading: safe(msg.heading, safe(prev.heading)),
        yaw: safe(msg.yaw, safe(prev.yaw)),

        latitude: safe(msg.latitude, safe(prev.latitude)),
        longitude: safe(msg.longitude, safe(prev.longitude)),
    };
}

export default function useTelemetry(url = "ws://localhost:9091/ws/stream") {
    const [telemetry, setTelemetry] = useState(null);
    const prevRef = useRef(null);

    useEffect(() => {
        if (!url) return;

        const ws = new WebSocket(url);

        ws.onmessage = (ev) => {
            try {
                const msg = JSON.parse(ev.data);
                const next = smoothTelemetry(prevRef.current, msg);
                prevRef.current = next;
                setTelemetry(next);
            } catch (e) {
                console.warn("WS JSON parse error:", e);
            }
        };

        ws.onerror = (e) => console.warn("WS error:", e);
        ws.onclose = () => console.warn("WS closed");

        return () => ws.close();
    }, [url]);

    return telemetry;
}