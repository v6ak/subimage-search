import { defineConfig } from 'vite';
import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs';

// Custom plugin to watch and compile Rust code
const rustWatcher = () => {
  let rustProcess = null;

  return {
    name: 'rust-watcher',
    configureServer(server) {
      // Watch for changes in Rust files
      server.watcher.add(path.resolve(__dirname, 'src-rust/**/*.rs'), path.resolve(__dirname, 'Cargo.*'));
      
      // Compile Rust code initially
      compileRust();
      
      // Recompile when .rs files change
      server.watcher.on('change', (filePath) => {
        if (filePath.endsWith('.rs') || filePath.startsWith(path.resolve(__dirname, 'Cargo.'))) {
          console.log('\nRust file changed. Recompiling...');
          compileRust();
        }
      });

      function compileRust() {
        // Kill any existing process
        if (rustProcess) {
          rustProcess.kill('SIGTERM');
        }

        // Run wasm-pack build command
        rustProcess = spawn('wasm-pack', ['build', '--target', 'web', '--release'], {
          stdio: 'inherit',
          shell: true
        });

        rustProcess.on('close', (code) => {
          if (code === 0) {
            console.log('Rust compilation finished successfully!');
            // Optionally trigger a page reload
            server.ws.send({ type: 'full-reload' });
          } else {
            console.error(`Rust compilation failed with code ${code}`);
          }
        });
      }
    }
  };
};

export default defineConfig({
  plugins: [rustWatcher()],
  build: {
    target: 'esnext',
    outDir: 'dist',
  },
  server: {
    open: true,
  },
  base: './',
  publicDir: 'pkg',
});
