// src/components/MapPanel.jsx
import { useEffect, useRef } from "react";
import {
    Viewer,
    ImageryLayer,
    OpenStreetMapImageryProvider,
    Cartesian3,
    Color,
    Entity,
    PolylineGlowMaterialProperty,
    Math as CesiumMath,
} from "cesium";
import "cesium/Build/Cesium/Widgets/widgets.css";

/**
 * MapPanel:
 * - Cesium 2D map
 * - blinking aircraft dot
 * - colored trajectory segments based on battery_power (W)
 *
 * Driven by props.telemetry (JSON from ws://localhost:9091/ws/stream)
 */
export default function MapPanel({ telemetry }) {
    const containerRef = useRef(null);
    const viewerRef = useRef(null);

    const planeEntityRef = useRef(null);
    const segmentEntitiesRef = useRef([]);
    const prevPointRef = useRef(null);
    const lastCamMoveRef = useRef(0);

    // blink
    const blinkOnRef = useRef(true);
    const blinkTimerRef = useRef(null);

    useEffect(() => {
        if (!containerRef.current) return;

        const osmProvider = new OpenStreetMapImageryProvider({
            url: "https://tile.openstreetmap.org/",
        });

        const viewer = new Viewer(containerRef.current, {
            baseLayer: new ImageryLayer(osmProvider),
            baseLayerPicker: false,
            geocoder: false,
            homeButton: false,
            sceneModePicker: false,
            navigationHelpButton: false,
            animation: false,
            timeline: false,
            fullscreenButton: false,
            infoBox: false,
            selectionIndicator: false,
            requestRenderMode: true,
            maximumRenderTimeChange: Infinity,
        });

        viewer.scene.morphTo2D(0);

        // Initial view (will update once telemetry arrives)
        viewer.camera.setView({
            destination: Cartesian3.fromDegrees(8.651235, 49.87285, 7000),
            orientation: {
                heading: 0,
                pitch: -CesiumMath.PI_OVER_TWO,
                roll: 0,
            },
        });

        const plane = viewer.entities.add(
            new Entity({
                name: "Plane",
                position: Cartesian3.fromDegrees(8.651235, 49.87285),
                point: {
                    pixelSize: 16,
                    color: Color.CYAN.withAlpha(1.0),
                    outlineColor: Color.WHITE.withAlpha(0.9),
                    outlineWidth: 3,
                },
            })
        );

        viewerRef.current = viewer;
        planeEntityRef.current = plane;

        // Blink timer (toggles alpha)
        blinkTimerRef.current = setInterval(() => {
            blinkOnRef.current = !blinkOnRef.current;
            if (planeEntityRef.current?.point) {
                const a = blinkOnRef.current ? 1.0 : 0.25;
                planeEntityRef.current.point.color = Color.CYAN.withAlpha(a);
                viewer.scene.requestRender();
            }
        }, 350);

        return () => {
            if (blinkTimerRef.current) clearInterval(blinkTimerRef.current);
            viewer.destroy();
            viewerRef.current = null;
            planeEntityRef.current = null;
            segmentEntitiesRef.current = [];
            prevPointRef.current = null;
        };
    }, []);

    // Update dot + trajectory whenever telemetry updates
    useEffect(() => {
        const viewer = viewerRef.current;
        const plane = planeEntityRef.current;
        if (!viewer || !plane || !telemetry) return;

        const lat = Number(telemetry.latitude);
        const lon = Number(telemetry.longitude);
        if (!Number.isFinite(lat) || !Number.isFinite(lon)) return;

        const powerW = Number(telemetry.battery_power);
        const currPos = Cartesian3.fromDegrees(lon, lat);

        // update plane point
        plane.position = currPos;

        // create colored segment from previous to current
        const prev = prevPointRef.current;
        if (prev) {
            const segEntity = viewer.entities.add(
                new Entity({
                    name: "TrackSegment",
                    polyline: {
                        positions: [prev.pos, currPos],
                        width: 15,
                        material: new PolylineGlowMaterialProperty({
                            glowPower: 0.12,
                            color: consumptionColor(powerW).withAlpha(0.95),
                        }),
                    },
                })
            );

            segmentEntitiesRef.current.push(segEntity);

            // keep last N segments
            const MAX_SEGS = 800;
            if (segmentEntitiesRef.current.length > MAX_SEGS) {
                const old = segmentEntitiesRef.current.shift();
                if (old) viewer.entities.remove(old);
            }
        }

        prevPointRef.current = { pos: currPos, powerW };

        // follow plane
        const now = Date.now();
        if (now - lastCamMoveRef.current > 1500) {
            viewer.camera.setView({
                destination: Cartesian3.fromDegrees(lon, lat, 1200),
            });
            lastCamMoveRef.current = now;
        }

        viewer.scene.requestRender();
    }, [telemetry]);

    return <div ref={containerRef} style={{ height: "100%", width: "100%" }} />;
}

// Smooth gradient green->yellow->red based on watts
function consumptionColor(powerW) {
    const lowW = 120;  // cruise
    const highW = 500; // takeoff / heavy load

    const t = clamp01((powerW - lowW) / (highW - lowW));
    if (t <= 0.5) return lerpColor(Color.LIME, Color.YELLOW, t / 0.5);
    return lerpColor(Color.YELLOW, Color.RED, (t - 0.5) / 0.5);
}

function clamp01(x) {
    return Math.max(0, Math.min(1, x));
}

function lerpColor(a, b, t) {
    const out = new Color();
    Color.lerp(a, b, t, out);
    return out;
}