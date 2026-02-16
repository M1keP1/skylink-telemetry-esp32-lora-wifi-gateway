# UAV Dashboard

A real-time dashboard for monitoring Unmanned Aerial Vehicle (UAV) telemetry data. This application provides a comprehensive interface for visualizing flight metrics, location, and status updates.

## Features

- **Primary Flight Display (PFD):** Visualizes critical flight data including:
  - Roll and Pitch attitude
  - Heading (Yaw)
  - Airspeed and Ground Speed
  - Altitude (Barometric)
  - Vertical Speed (Climb Rate)
- **Interactive Map:** Real-time 3D map visualization using [Cesium](https://cesium.com/platform/cesiumjs/) to track the UAV's position.
- **Telemetry Panel:** Detailed view of raw telemetry data.
- **Flight Status Overlay:** key flight status information.
- **Live Connectivity:** Connects to a backend telemetry server via WebSocket (`ws://localhost:9091/ws/stream`) for real-time updates.

## Tech Stack

- **Frontend Framework:** [React](https://react.dev/)
- **Build Tool:** [Vite](https://vitejs.dev/)
- **Mapping Library:** [Cesium](https://cesium.com/platform/cesiumjs/)
- **Styling:** CSS

## Prerequisites
- npm (Node Package Manager)
- A running instance of the telemetry backend server (expected at `ws://localhost:9091`)

## Installation

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd uav-dashboard
    ```

2.  Install dependencies:
    ```bash
    npm install
    ```

## Usage

1.  Start the development server:
    ```bash
    npm run dev
    ```

2.  Open your browser and navigate to `http://localhost:5173` (or the URL provided in the terminal).

3.  Click the **Connect** button to establish a WebSocket connection to the telemetry server.

## Project Structure

- `src/components/`: Reusable UI components (PFD, MapPanel, TelemetryPanel, etc.)
- `src/hooks/`: Custom React hooks (e.g., `useTelemetry` for WebSocket management).
- `src/App.jsx`: Main application component layout.

## Scripts

- `npm run dev`: Starts the development server.
- `npm run build`: Builds the application for production.
- `npm run lint`: Runs ESLint to check for code quality issues.
- `npm run preview`: Previews the production build locally.

## Team & Responsibilities

### Backend System (SkyLink Gateway) - Mihir Kumar Patel (1123669)

- Core telemetry gateway implementation
- KV store design and optimization
- Flight detection state machine
- REST/WebSocket API architecture

### Frontend System (UAV Dashboard) - Vedant Sorout

- **Dashboard Architecture:** Designed and implemented the real-time React application using Vite.
- **Primary Flight Display (PFD):** Developed the canvas-based flight instrument visualization.
- **3D Map Integration:** Integrated CesiumJS for real-time UAV tracking and geospatial visualization.
- **Telemetry Integration:** Implemented WebSocket client hooks for low-latency data streaming and state management.

