## RustyPlanet GUI

A TypeScript/React GUI for observing and controlling RustyPlanet simulations.

### Tech stack

- React + Vite + TypeScript
- MUI (Material UI) for layout and components
- Zustand for local state management
- TanStack React Query (ready for real API integration)

### Features

- **Simulation controls** (`Simulation` tab):
  - Start / Pause / Step buttons.
  - Speed slider with live value.
  - State, tick and speed are stored in `simulationStore` and used across the app.
- **Simulation status overview** (`Overview` tab):
  - Shows current run state, tick, speed and number of planets.
- **Planet list and details** (`Overview` + `Planet` tabs):
  - List of planets with type, energy and error counts.
  - Detailed view for the selected planet: state, energy, resources, and Skycartel‑specific stats.
- **Energy cells visualization** (`Planet` tab):
  - Visual representation of resource and defense energy cells and their charge state.
- **Logs console** (`Logs` tab):
  - Streaming log view with level filter and start/pause control (mocked stream for now).
- **Trajectory visualization** (`Visualization` tab):
  - SVG orbital view with mocked trajectories for planets and an asteroid stream.

### Running the GUI

```bash
cd rustyplanet-gui
npm install
npm run dev
```

Then open the URL printed by Vite (by default `http://localhost:5173`).

### Backend integration

Right now the GUI uses mock services (`simulationService`, `planetService`, `logStore` mock stream).
To connect it to a real Rust backend, replace the mock implementations with HTTP/WebSocket calls that match:

- `/api/simulation/*` for simulation status and controls.
- `/api/planets` and `/api/planets/:id` for planet summaries and details.
- `ws://.../ws/logs` for the log stream.

# React + TypeScript + Vite

This template provides a minimal setup to get React working in Vite with HMR and some ESLint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Babel](https://babeljs.io/) (or [oxc](https://oxc.rs) when used in [rolldown-vite](https://vite.dev/guide/rolldown)) for Fast Refresh
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/) for Fast Refresh

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```
