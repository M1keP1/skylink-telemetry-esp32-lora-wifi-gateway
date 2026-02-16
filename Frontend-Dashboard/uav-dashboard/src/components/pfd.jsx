import { useEffect, useRef } from "react";

/**
 * PFD v2: Artificial horizon + roll scale + pitch ladder
 * + Airspeed tape (left) + Altitude tape (right)
 *
 * Props (from your websocket JSON):
 *  - rollDeg  <- roll
 *  - pitchDeg <- pitch
 *  - speedMps <- ground_speed
 *  - altM     <- altitude_baro (use this)
 *  - climbMps <- vertical_speed
 */
export default function Pfd({
    rollDeg = 0,
    pitchDeg = 0,
    speedMps = 0,
    altM = 0,
    climbMps = 0,
    headingDeg = 0,
    width = 340,
    height = 200,
}) {
    const canvasRef = useRef(null);

    useEffect(() => {
        const c = canvasRef.current;
        if (!c) return;
        const ctx = c.getContext("2d");
        if (!ctx) return;

        const dpr = window.devicePixelRatio || 1;
        c.width = Math.floor(width * dpr);
        c.height = Math.floor(height * dpr);
        c.style.width = `${width}px`;
        c.style.height = `${height}px`;
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

        drawPfd(ctx, width, height, {
            rollDeg,
            pitchDeg,
            speedMps,
            altM,
            climbMps,
            headingDeg,
        });
    }, [rollDeg, pitchDeg, speedMps, altM, climbMps, headingDeg, width, height]);

    return (
        <div style={{ height: "100%", display: "grid", placeItems: "center" }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

function drawPfd(ctx, w, h, s) {
    ctx.clearRect(0, 0, w, h);

    // Frame
    roundRect(ctx, 0, 0, w, h, 12);
    ctx.fillStyle = "#06122b";
    ctx.fill();
    ctx.strokeStyle = "#1c2a44";
    ctx.lineWidth = 2;
    ctx.stroke();


    const pad = 10;

    // Reserve side tapes
    const tapeW = 110;
    const gap = 8;

    const winX = pad + tapeW + gap;
    const winY = pad;
    const winW = w - (pad * 2 + tapeW * 2 + gap * 2);
    const winH = h - pad * 2;
    const headingH = 60;
    const hcX = winX + winW / 2;
    const hcY = winY + headingH + (winH - headingH) / 2;
    // --- Horizon window (center)
    ctx.save();
    ctx.beginPath();
    roundRect(ctx, winX, winY + headingH, winW, winH - headingH, 10);
    ctx.clip();

    const rollRad = (safeNum(s.rollDeg) * Math.PI) / 180;
    const pitchClamped = clamp(safeNum(s.pitchDeg), -90, 90);
    const pxPerDegPitch = 4.5;
    const pitchPx = pitchClamped * pxPerDegPitch;

    ctx.translate(hcX, hcY);
    ctx.rotate(-rollRad);
    ctx.translate(0, pitchPx);

    // Sky / Ground
    const skyGrad = ctx.createLinearGradient(0, -h, 0, 0);
    skyGrad.addColorStop(0, "#03abf3e9"); // Deep blue
    skyGrad.addColorStop(1, "#a8e0fc"); // Lighter/whiter at horizon

    ctx.fillStyle = skyGrad;
    ctx.fillRect(-w, -h * 2, w * 2, h * 2);
    ctx.fillStyle = "#87ea29";
    ctx.fillRect(-w, 0, w * 2, h * 2);

    // Horizon line
    ctx.strokeStyle = "#e6eefc";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(-w, 0);
    ctx.lineTo(w, 0);
    ctx.stroke();

    // Pitch ladder (dynamic: normally ±30, expands up to ±90)
    ctx.strokeStyle = "#e8070e";
    ctx.fillStyle = "#ea030a";
    ctx.lineWidth = 2;
    ctx.font = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

    const minorStep = 5;
    const majorEvery = 10;
    const baseRange = 30;
    const maxRange = 90;
    const dynamicRange = Math.min(
        maxRange,
        Math.max(baseRange, Math.ceil(Math.abs(pitchClamped) / 10) * 10)
    );

    for (let p = -dynamicRange; p <= dynamicRange; p += minorStep) {
        if (p === 0) continue;
        const y = -p * pxPerDegPitch;
        const isMajor = p % majorEvery === 0;
        const half = isMajor ? 60 : 35;

        ctx.globalAlpha = isMajor ? 0.95 : 0.65;
        ctx.beginPath();
        ctx.moveTo(-half, y);
        ctx.lineTo(half, y);
        ctx.stroke();

        if (isMajor) {
            const label = p.toString(); // shows -10, -20 below horizon
            ctx.fillText(label, -half - 28, y + 4);
            ctx.fillText(label, half + 10, y + 4);
        }
    }
    ctx.globalAlpha = 1;

    ctx.restore(); // end clip + transforms

    // --- Roll scale (top arc + labels)
    drawRollScale(ctx, hcX, hcY, winW, winH - headingH, safeNum(s.rollDeg));
    drawAircraftSymbol(ctx, hcX, hcY);

    // --- Side tapes
    drawSpeedTape(ctx, {
        x: pad,
        y: pad,
        w: tapeW,
        h: winH,
        speedMps: safeNum(s.speedMps),
    });

    drawAltTape(ctx, {
        x: w - pad - tapeW,
        y: pad,
        w: tapeW,
        h: winH,
        altM: safeNum(s.altM),
        climbMps: safeNum(s.climbMps),
    });
    drawHeadingTape(ctx, {
        x: winX,
        y: winY,
        w: winW,
        h: headingH,
        headingDeg: safeHeading(s.headingDeg ?? 0),
    });
}

/* -------------------- Drawing helpers -------------------- */

function drawRollScale(ctx, cx, cy, winW, horizonH, rollDeg) {
    ctx.save();
    ctx.translate(cx, cy);
    ctx.strokeStyle = "#f71e0aff";
    ctx.fillStyle = "#ed2405ff";
    ctx.lineWidth = 2;
    ctx.globalAlpha = 0.9;

    const r = Math.min(winW, horizonH) * 0.45;

    ctx.beginPath();
    ctx.arc(0, 0, r, (210 * Math.PI) / 180, (330 * Math.PI) / 180);
    ctx.stroke();

    const ticks = [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60];
    ctx.font = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

    ticks.forEach((t) => {
        const a = ((-t - 90) * Math.PI) / 180;
        const len = t % 30 === 0 ? 14 : 8;

        const x1 = Math.cos(a) * (r - len);
        const y1 = Math.sin(a) * (r - len);
        const x2 = Math.cos(a) * r;
        const y2 = Math.sin(a) * r;

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();

        const label = String(Math.abs(t));
        const tx = Math.cos(a) * (r + 16);
        const ty = Math.sin(a) * (r + 16);
        const tw = ctx.measureText(label).width;
        ctx.fillText(label, tx - tw / 2, ty + 4);
    });

    const pointerAng = (-rollDeg * Math.PI) / 180;
    ctx.rotate(pointerAng);
    ctx.beginPath();
    ctx.moveTo(0, -r - 2);
    ctx.lineTo(-8, -r + 12);
    ctx.lineTo(8, -r + 12);
    ctx.closePath();
    ctx.fill();

    ctx.restore();
    ctx.globalAlpha = 1;
}

function drawAircraftSymbol(ctx, cx, cy) {
    ctx.save();
    ctx.translate(cx, cy);
    ctx.strokeStyle = "#e6042d";
    ctx.lineWidth = 3;

    ctx.beginPath();
    ctx.moveTo(-55, 0);
    ctx.lineTo(-15, 0);
    ctx.moveTo(15, 0);
    ctx.lineTo(55, 0);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(-15, 0);
    ctx.lineTo(-6, 0);
    ctx.moveTo(6, 0);
    ctx.lineTo(15, 0);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(0, 12);
    ctx.stroke();

    ctx.restore();
}
function drawSpeedTape(ctx, { x, y, w, h, speedMps }) {
    // Background
    ctx.save();
    roundRect(ctx, x, y, w, h, 10);
    ctx.fillStyle = "#0c1526";
    ctx.fill();
    ctx.strokeStyle = "#1c2a44";
    ctx.lineWidth = 2;
    ctx.stroke();

    // Values
    const speed = Math.max(0, speedMps);
    const speedKmh = speed * 3.6;

    // Tape settings
    const pxPerUnit = 3;     // px per 1 km/h
    const minorStep = 5;     // km/h
    const majorStep = 10;    // km/h
    const centerY = y + h / 2;

    const current = speedKmh;
    const currentRounded = Math.round(current / minorStep) * minorStep;

    // Draw ticks around current
    ctx.beginPath();
    ctx.rect(x, y, w, h);
    ctx.clip();

    ctx.strokeStyle = "#3cff3c";
    ctx.fillStyle = "#3cff3c";
    ctx.lineWidth = 2;
    ctx.font = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

    const range = 60; // show +/- 60 km/h around
    for (let v = currentRounded - range; v <= currentRounded + range; v += minorStep) {
        if (v < 0) continue;
        const dy = (current - v) * pxPerUnit;
        const yy = centerY + dy;

        const isMajor = v % majorStep === 0;
        const tickLen = isMajor ? 22 : 12;

        ctx.globalAlpha = isMajor ? 0.95 : 0.6;

        ctx.beginPath();
        ctx.moveTo(x + w - tickLen - 10, yy);
        ctx.lineTo(x + w - 10, yy);
        ctx.stroke();

        if (isMajor) {
            const label = String(v);
            const tw = ctx.measureText(label).width;
            ctx.fillText(label, x + 10, yy + 4);
        }
    }
    ctx.globalAlpha = 1;

    // Pointer box
    drawSidePointer(ctx, {
        x: x + 6,
        y: centerY,
        w: w - 20,
        text: `${current.toFixed(0)}`,
        //sub: "km/h",

    });

    // label
    ctx.fillStyle = "#e6eefc";
    ctx.globalAlpha = 0.85;
    ctx.font = "11px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
    ctx.fillText("SPD", x + 10, y + 14);
    ctx.globalAlpha = 1;

    ctx.restore();
}

function drawAltTape(ctx, { x, y, w, h, altM, climbMps }) {
    ctx.save();
    roundRect(ctx, x, y, w, h, 10);
    ctx.fillStyle = "#0c1526";
    ctx.fill();
    ctx.strokeStyle = "#1c2a44";
    ctx.lineWidth = 2;
    ctx.stroke();

    const alt = altM;
    const centerY = y + h / 2;

    // Tape settings (meters)
    const pxPerM = 2.2;
    const minorStep = 5;
    const majorStep = 10;
    const current = alt;
    const currentRounded = Math.round(current / minorStep) * minorStep;

    ctx.beginPath();
    ctx.rect(x, y, w, h);
    ctx.clip();

    ctx.strokeStyle = "#3cff3c";
    ctx.fillStyle = "#3cff3c";
    ctx.lineWidth = 2;
    ctx.font = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

    const rangeM = 80; // show +/- 80m
    for (let v = currentRounded - rangeM; v <= currentRounded + rangeM; v += minorStep) {
        const dy = (current - v) * pxPerM;
        const yy = centerY + dy;

        const isMajor = v % majorStep === 0;
        const tickLen = isMajor ? 22 : 12;

        ctx.globalAlpha = isMajor ? 0.95 : 0.6;

        ctx.beginPath();
        ctx.moveTo(x + 10, yy);
        ctx.lineTo(x + 10 + tickLen, yy);
        ctx.stroke();

        if (isMajor) {
            ctx.save();
            ctx.textAlign = "right";      // ✅ align digits
            ctx.textBaseline = "middle";  // ✅ stable vertical alignment

            // right edge just inside the tape
            const labelX = x + w - 12;

            ctx.fillText(String(v), labelX, yy);
            ctx.restore();
        }
    }
    ctx.globalAlpha = 1;

    // Pointer box
    drawSidePointer(ctx, {
        x: x + 6,
        y: centerY,
        w: w - 6,
        text: `${alt.toFixed(1)}`,
        // sub: "m",
        alignRight: true,
    });

    // climb text
    ctx.fillStyle = "#e6eefc";
    ctx.globalAlpha = 0.85;
    ctx.font = "11px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
    ctx.fillText("ALT", x + 20, y + 16);
    ctx.fillText(
        `V/S ${(climbMps >= 0 ? "+" : "") + climbMps.toFixed(1)} m/s`,
        x + 10,
        y + 32
    );
    ctx.globalAlpha = 1;

    ctx.restore();
}

function drawSidePointer(ctx, { x, y, w, text, sub, alignRight = false }) {
    const boxH = 34;
    const boxY = y - boxH / 2;

    ctx.save();

    roundRect(ctx, x, boxY, w, boxH, 8);
    // darker + more contrast background
    ctx.fillStyle = "#081022";
    ctx.fill();

    // thicker, brighter border
    ctx.strokeStyle = "rgba(60,255,60,0.85)";
    ctx.lineWidth = 3;
    ctx.stroke();

    // subtle glow
    ctx.shadowColor = "rgba(60,255,60,0.35)";
    ctx.shadowBlur = 10;
    ctx.shadowOffsetX = 0;
    ctx.shadowOffsetY = 0;

    // draw border again so glow shows
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.fillStyle = "#3cff3c";
    ctx.font = "24px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
    ctx.textBaseline = "middle";

    const mainX = alignRight ? x + w - 10 : x + 10;
    ctx.textAlign = alignRight ? "right" : "left";
    ctx.fillText(text, mainX, y);

    ctx.font = "11px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
    ctx.fillStyle = "#e6eefc";
    ctx.globalAlpha = 0.85;
    // ctx.globalAlpha = 1;

    ctx.restore();
}

function safeNum(v) {
    return Number.isFinite(v) ? v : 0;
}
function clamp(v, a, b) {
    return Math.max(a, Math.min(b, v));
}

function roundRect(ctx, x, y, w, h, r) {
    const rr = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + rr, y);
    ctx.arcTo(x + w, y, x + w, y + h, rr);
    ctx.arcTo(x + w, y + h, x, y + h, rr);
    ctx.arcTo(x, y + h, x, y, rr);
    ctx.arcTo(x, y, x + w, y, rr);
    ctx.closePath();
}

function drawHeadingTape(ctx, { x, y, w, h, headingDeg }) {
    ctx.save();

    // background bar
    roundRect(ctx, x, y, w, h, 10);
    ctx.fillStyle = "rgba(12, 21, 38, 0.92)";
    ctx.fill();
    ctx.strokeStyle = "#1c2a44";
    ctx.lineWidth = 2;
    ctx.stroke();

    const cx = x + w / 2;
    const pxPerDeg = 4;      // spacing
    const minorStep = 5;     // tick every 5°
    const majorStep = 10;    // label every 10°
    const rangeDeg = Math.floor((w / pxPerDeg) / 2) + 10;

    // clip to the bar
    ctx.beginPath();
    roundRect(ctx, x, y, w, h, 10);
    ctx.clip();

    ctx.strokeStyle = "#3cff3c";
    ctx.fillStyle = "#3cff3c";
    ctx.lineWidth = 2;
    ctx.font = "16px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
    ctx.textBaseline = "middle";

    const start = Math.floor((headingDeg - rangeDeg) / minorStep) * minorStep;
    const end = Math.ceil((headingDeg + rangeDeg) / minorStep) * minorStep;

    for (let deg = start; deg <= end; deg += minorStep) {
        const d = wrap360(deg);
        const xx = cx + (deg - headingDeg) * pxPerDeg;

        const isMajor = d % majorStep === 0;
        const tickH = isMajor ? 10 : 6;

        ctx.globalAlpha = isMajor ? 0.95 : 0.6;

        // tick
        ctx.beginPath();
        ctx.moveTo(xx, y + h - 4);
        ctx.lineTo(xx, y + h - 4 - tickH);
        ctx.stroke();

        // label
        if (isMajor) {
            const label = headingLabel(d);
            const tw = ctx.measureText(label).width;
            ctx.fillText(label, xx - tw / 2, y + 12);
        }
    }
    ctx.globalAlpha = 1;
    ctx.restore();

    // center pointer
    ctx.save();
    ctx.strokeStyle = "#e6eefc";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(cx, y + h - 1);
    ctx.lineTo(cx, y + h - 2);
    ctx.stroke();

    ctx.fillStyle = "#e6eefc";
    ctx.beginPath();
    ctx.moveTo(cx, y + h);
    ctx.lineTo(cx - 7, y + h - 10);
    ctx.lineTo(cx + 7, y + h - 10);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
}

function headingLabel(deg) {
    if (deg === 0) return "N";
    if (deg === 90) return "E";
    if (deg === 180) return "S";
    if (deg === 270) return "W";
    return String(deg).padStart(3, "0");
}

function wrap360(deg) {
    let d = deg % 360;
    if (d < 0) d += 360;
    return d;
}

function safeHeading(v) {
    const n = Number(v);
    return Number.isFinite(n) ? wrap360(n) : 0;
}