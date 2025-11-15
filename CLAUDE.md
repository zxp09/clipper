# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri application called "clipper" that combines a Rust backend with a React + TypeScript frontend. The app uses Vite as the build tool and is configured for cross-platform desktop application development.

## Development Commands

### Frontend Development
- `npm run dev` - Start Vite development server (frontend only)
- `npm run build` - Build the React frontend for production
- `npm run preview` - Preview the built frontend

### Tauri Development
- `npm run tauri dev` - Run the full Tauri app in development mode (builds both frontend and backend)
- `npm run tauri build` - Build the complete Tauri application for distribution

### Rust Backend
- `cargo check` (from src-tauri directory) - Check Rust code for errors
- `cargo build` (from src-tauri directory) - Build Rust backend
- `cargo test` (from src-tauri directory) - Run Rust tests

## Project Architecture

### Frontend Structure
- **Entry point**: `src/main.tsx` - Standard React bootstrap
- **Main component**: `src/App.tsx` - Contains the main UI and Tauri command integration
- **Build tool**: Vite with React plugin (`vite.config.ts`)
- **Dev server**: Runs on port 1420 (fixed port required by Tauri)

### Backend Structure
- **Entry point**: `src-tauri/src/main.rs` - Calls the library's run function
- **Core logic**: `src-tauri/src/lib.rs` - Contains Tauri application setup and commands
- **Tauri commands**: Currently implements a `greet` command that takes a name parameter
- **Configuration**: `src-tauri/tauri.conf.json` - Main Tauri app configuration

### Communication Pattern
The frontend communicates with Rust backend through Tauri's `invoke()` function:
```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { param: value });
```

Rust commands are defined using `#[tauri::command]` macro and registered in the Builder.

## Key Configuration

### Tauri Configuration (`tauri.conf.json`)
- App identifier: `com.clipper.app`
- Development URL: `http://localhost:1420`
- Frontend dist directory: `../dist`
- Default window: 800x600px

### Vite Configuration
- React plugin enabled
- Fixed port 1420 for Tauri compatibility
- Ignores `src-tauri` directory in watch mode

### TypeScript Configuration
- Strict mode enabled
- Target: ES2020
- React JSX transform enabled
- Unused variables/parameters checking enabled

## Development Workflow

1. **Full development**: Use `npm run tauri dev` to run both frontend and backend with hot reload
2. **Frontend-only**: Use `npm run dev` for faster frontend iteration
3. **Backend changes**: Navigate to `src-tauri` and use Cargo commands for Rust development

## File Organization

- `/src/` - React frontend source code
- `/src-tauri/` - Rust backend source code and Tauri configuration
- `/public/` - Static assets for frontend
- `/dist/` - Built frontend output (generated)