% Simple MATLAB WebSocket Client for SkyLink Telemetry
% Minimal example showing just altitude and speed
%
% Requirements: MATLAB R2020b or later

clear; clc; close all;

%% Configuration
WS_URL = "ws://localhost:3000/ws/telemetry";

%% Create figure
figure('Name', 'SkyLink Telemetry', 'Position', [100, 100, 800, 400]);

subplot(1, 2, 1);
h_alt = animatedline('Color', 'b', 'LineWidth', 2);
ylabel('Altitude (m)');
xlabel('Time (s)');
title('Altitude');
grid on;

subplot(1, 2, 2);
h_speed = animatedline('Color', 'r', 'LineWidth', 2);
ylabel('Speed (m/s)');
xlabel('Time (s)');
title('Speed');
grid on;

%% Connect
fprintf('Connecting to %s...\n', WS_URL);
ws = websocket(WS_URL);
fprintf('Connected!\n');

%% Receive and plot
start_time = tic;
packet_count = 0;

fprintf('Receiving data... (Press Ctrl+C to stop)\n');

while isvalid(ws) && ws.Status == "open"
    if ws.NumMessagesAvailable > 0
        msg = read(ws);
        packet = jsondecode(msg);
        packet_count = packet_count + 1;
        
        t = toc(start_time);
        
        addpoints(h_alt, t, packet.baro.alt);
        addpoints(h_speed, t, packet.gps.speed);
        
        if mod(packet_count, 10) == 0
            drawnow limitrate;
            fprintf('[%d] Alt: %.1fm, Speed: %.1fm/s\n', ...
                    packet_count, packet.baro.alt, packet.gps.speed);
        end
    else
        pause(0.01);
    end
end

fprintf('Disconnected. Packets: %d\n', packet_count);
close(ws);
