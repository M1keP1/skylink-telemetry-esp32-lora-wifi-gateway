% MATLAB WebSocket Client for SkyLink Telemetry
% Connects to the WebSocket endpoint and plots real-time telemetry data
%
% Requirements:
%   - MATLAB R2020b or later (WebSocket support)
%   - SkyLink server running: cargo run --example combined_receiver_api
%
% Usage:
%   1. Start the SkyLink server
%   2. Run this script in MATLAB
%   3. Press Ctrl+C to stop

clear; clc; close all;

%% Configuration
WS_URL = "ws://localhost:3000/ws/telemetry";
MAX_POINTS = 100;  % Maximum number of points to display

%% Initialize data storage
data.timestamp = [];
data.altitude = [];
data.speed = [];
data.voltage = [];
data.current = [];
data.roll = [];
data.pitch = [];
data.yaw = [];
data.rssi = [];

%% Create figure with subplots
fig = figure('Name', 'SkyLink Telemetry Monitor', 'NumberTitle', 'off', ...
             'Position', [100, 100, 1200, 800]);

% Altitude plot
subplot(3, 3, 1);
h_alt = animatedline('Color', 'b', 'LineWidth', 1.5);
ylabel('Altitude (m)');
title('Barometric Altitude');
grid on;

% Speed plot
subplot(3, 3, 2);
h_speed = animatedline('Color', 'r', 'LineWidth', 1.5);
ylabel('Speed (m/s)');
title('GPS Speed');
grid on;

% Battery voltage
subplot(3, 3, 3);
h_voltage = animatedline('Color', 'g', 'LineWidth', 1.5);
ylabel('Voltage (V)');
title('Battery Voltage');
grid on;

% Battery current
subplot(3, 3, 4);
h_current = animatedline('Color', 'm', 'LineWidth', 1.5);
ylabel('Current (A)');
title('Battery Current');
grid on;

% Roll
subplot(3, 3, 5);
h_roll = animatedline('Color', 'c', 'LineWidth', 1.5);
ylabel('Roll (deg)');
title('Roll Angle');
grid on;

% Pitch
subplot(3, 3, 6);
h_pitch = animatedline('Color', 'k', 'LineWidth', 1.5);
ylabel('Pitch (deg)');
title('Pitch Angle');
grid on;

% Yaw
subplot(3, 3, 7);
h_yaw = animatedline('Color', [0.5 0.5 0]);
ylabel('Yaw (deg)');
title('Yaw Angle');
xlabel('Time (s)');
grid on;

% RSSI
subplot(3, 3, 8);
h_rssi = animatedline('Color', [0.8 0.2 0.2], 'LineWidth', 1.5);
ylabel('RSSI (dBm)');
title('Link Quality (RSSI)');
xlabel('Time (s)');
grid on;

% Status text
subplot(3, 3, 9);
axis off;
h_status = text(0.1, 0.9, 'Connecting...', 'FontSize', 10, ...
                'VerticalAlignment', 'top', 'Interpreter', 'none');

%% Connect to WebSocket
fprintf('Connecting to %s...\n', WS_URL);
try
    ws = websocket(WS_URL);
    fprintf('Connected!\n');
    set(h_status, 'String', sprintf('Connected\nPackets: 0\nPhase: -'));
catch e
    fprintf('Error connecting: %s\n', e.message);
    fprintf('Make sure the server is running:\n');
    fprintf('  cargo run --example combined_receiver_api\n');
    return;
end

%% Main loop
packet_count = 0;
start_time = tic;

fprintf('Receiving telemetry data... (Press Ctrl+C to stop)\n');

while isvalid(ws) && ws.Status == "open"
    try
        % Read message from WebSocket
        if ws.NumMessagesAvailable > 0
            msg = read(ws);
            
            % Parse JSON
            packet = jsondecode(msg);
            packet_count = packet_count + 1;
            
            % Calculate relative time
            rel_time = toc(start_time);
            
            % Store data
            data.timestamp(end+1) = rel_time;
            data.altitude(end+1) = packet.baro.alt;
            data.speed(end+1) = packet.gps.speed;
            data.voltage(end+1) = packet.battery.voltage;
            data.current(end+1) = packet.battery.current;
            data.roll(end+1) = packet.imu.roll;
            data.pitch(end+1) = packet.imu.pitch;
            data.yaw(end+1) = packet.imu.yaw;
            data.rssi(end+1) = packet.link.rssi;
            
            % Limit data points
            if length(data.timestamp) > MAX_POINTS
                data.timestamp = data.timestamp(end-MAX_POINTS+1:end);
                data.altitude = data.altitude(end-MAX_POINTS+1:end);
                data.speed = data.speed(end-MAX_POINTS+1:end);
                data.voltage = data.voltage(end-MAX_POINTS+1:end);
                data.current = data.current(end-MAX_POINTS+1:end);
                data.roll = data.roll(end-MAX_POINTS+1:end);
                data.pitch = data.pitch(end-MAX_POINTS+1:end);
                data.yaw = data.yaw(end-MAX_POINTS+1:end);
                data.rssi = data.rssi(end-MAX_POINTS+1:end);
            end
            
            % Update plots
            addpoints(h_alt, rel_time, packet.baro.alt);
            addpoints(h_speed, rel_time, packet.gps.speed);
            addpoints(h_voltage, rel_time, packet.battery.voltage);
            addpoints(h_current, rel_time, packet.battery.current);
            addpoints(h_roll, rel_time, packet.imu.roll);
            addpoints(h_pitch, rel_time, packet.imu.pitch);
            addpoints(h_yaw, rel_time, packet.imu.yaw);
            addpoints(h_rssi, rel_time, packet.link.rssi);
            
            % Update status
            status_str = sprintf('Connected\nPackets: %d\nPhase: %s\nSeq: %d\nAlt: %.1f m', ...
                                 packet_count, packet.phase, packet.seq, packet.baro.alt);
            set(h_status, 'String', status_str);
            
            % Update display every 10 packets
            if mod(packet_count, 10) == 0
                drawnow limitrate;
                fprintf('[%d] Alt: %.1fm, Speed: %.1fm/s, Voltage: %.2fV, Phase: %s\n', ...
                        packet_count, packet.baro.alt, packet.gps.speed, ...
                        packet.battery.voltage, packet.phase);
            end
        else
            % No messages available, pause briefly
            pause(0.01);
        end
        
    catch e
        fprintf('Error processing packet: %s\n', e.message);
        break;
    end
end

%% Cleanup
fprintf('\nDisconnected. Total packets received: %d\n', packet_count);
if isvalid(ws)
    close(ws);
end

% Save data to workspace
assignin('base', 'telemetry_data', data);
fprintf('Data saved to workspace variable: telemetry_data\n');
