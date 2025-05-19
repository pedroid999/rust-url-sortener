#!/bin/bash

# Función para detener los procesos al salir
cleanup() {
    echo "Stopping servers..."
    kill $BACKEND_PID 2>/dev/null
    kill $FRONTEND_PID 2>/dev/null
    exit 0
}

# Capturar señales para cerrar correctamente
trap cleanup SIGINT SIGTERM

# Asegurarse de que no hay instancias antiguas ejecutándose
echo "Stopping any existing servers..."
killall rust-url-shortener 2>/dev/null || true
killall trunk 2>/dev/null || true

# Crear directorios necesarios
mkdir -p data

echo "Starting backend server on http://localhost:8081"
cargo run &
BACKEND_PID=$!

# Esperar a que el backend esté listo
sleep 2

echo "Starting frontend server on http://localhost:8080"
cd frontend && trunk serve &
FRONTEND_PID=$!

echo "==========================================="
echo "🚀 URL Shortener is running!"
echo "- Frontend: http://localhost:8080"
echo "- Backend: http://localhost:8081"
echo "- Press Ctrl+C to stop all servers"
echo "==========================================="

# Mantener el script en ejecución
wait $FRONTEND_PID
cleanup 