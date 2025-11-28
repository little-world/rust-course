# Angular WebSocket Chat Client

This is an Angular client for the Rust WebSocket chat server (Project 21: Chat Server with Broadcast and Backpressure).

## Prerequisites

- Node.js (v18 or higher)
- Angular CLI (v17 or higher)

## Installation

```bash
# Install Angular CLI globally (if not already installed)
npm install -g @angular/cli

# Navigate to the project directory
cd angular-chat-client

# Install dependencies
npm install
```

## Development Server

```bash
# Start the development server
ng serve

# Or with specific port
ng serve --port 4200
```

Navigate to `http://localhost:4200/`. The application will automatically reload if you change any of the source files.

## Build

```bash
# Build for production
ng build

# The build artifacts will be stored in the `dist/` directory
```

## Usage

1. Start the Rust WebSocket chat server (from Milestone 6)
2. Run the Angular development server
3. Open your browser to `http://localhost:4200`
4. Set your username and start chatting!

## Features

- Real-time WebSocket communication
- Auto-reconnect on disconnect
- Username management with duplicate prevention
- User list display
- Private messaging (whisper)
- Server statistics
- Message timestamps
- Responsive design
- TypeScript type safety
- RxJS-based reactive programming
