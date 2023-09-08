FROM node:latest as frontend
WORKDIR /usr/src/app/
COPY ./frontend .
RUN rm -rf .parcel-cache && rm -rf ./dist
RUN npm install && npm run build:docker && cp -r ./lib ./dist

FROM rust:latest as backend
COPY ./deps/libvosk.so /usr/local/lib/libvosk.so

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/speech-rs ./speech-rs

# Runtime image
FROM rust:latest

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=backend /usr/src/app/speech-rs /app/speech-rs
COPY --from=backend /usr/src/app/model /app/model
COPY --from=backend /usr/local/lib/libvosk.so /usr/local/lib/libvosk.so
COPY --from=frontend /usr/src/app/dist /app/public

ENV LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Run the app
EXPOSE 3000
CMD ./speech-rs