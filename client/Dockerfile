# Используем официальный образ Rust для сборки
FROM rust:latest as builder

# Создаем рабочую директорию
WORKDIR /usr/src/app

# Копируем файлы проекта
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Собираем релизную версию
RUN cargo build --release

# Используем легковесный образ для рантайма
FROM debian:bookworm-slim
RUN apt-get update && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/client /usr/local/bin/
ENV SERVER1_ADDRESS=host.docker.internal:7878
ENV SERVER2_ADDRESS=host.docker.internal:7979
CMD ["sh", "-c", "RUST_LOG=info /usr/local/bin/client"]
