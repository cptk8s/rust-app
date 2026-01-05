# --- Etapa 1: Compilación ---
FROM rust:1.75-alpine AS builder

# Instalamos dependencias necesarias para compilar SQLx y otras librerías de C
RUN apk add --no-cache musl-dev gcc sqlite-dev

WORKDIR /workspaces/rust/app
COPY . .

# Compilamos para release para optimizar el tamaño y velocidad
RUN cargo build --release

# --- Etapa 2: Ejecución (Imagen final) ---
FROM alpine:3.19

# Instalamos las librerías mínimas necesarias para ejecutar el binario y certificados SSL
RUN apk add --no-cache libgcc libstdc++ sqlite-libs ca-certificates

# Copiamos el binario desde la etapa de compilación
COPY --from=builder /workspaces/rust/app/target/release/app /usr/local/bin/app

# Exponemos el puerto
EXPOSE 3000

# Variables de entorno por defecto
ENV RUST_LOG=info

# Ejecutamos la aplicación
CMD ["app"]