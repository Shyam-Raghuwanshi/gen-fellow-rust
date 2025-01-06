# Stage 1: Build using AWS Lambda Node.js image
FROM public.ecr.aws/lambda/nodejs:20 AS builder

# Set the working directory
WORKDIR /app

# Install dependencies for Rust and NAPI build
RUN microdnf install -y \
    gcc \
    gcc-c++ \
    make \
    rust \
    cargo \
    libstdc++ \
    openssl-devel \
    pkg-config && \
    microdnf clean all

# Install Rust using rustup and set up the environment
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

# Copy source code and dependencies
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY package*.json ./

# Install Node.js dependencies and build the Rust-based NAPI module
RUN npm ci
RUN npm run build:napi
RUN ls -la
# Stage 2: Publish using tozny/npm image
FROM tozny/npm

# Set the working directory
WORKDIR /src/

# Copy built files from the previous stage
COPY ./pkg/package.json ./
COPY ./index.d.ts ./
COPY --from=builder /app/rust-lib.linux-x64-gnu.node ./

# Copy publish script
COPY publish.sh /bin/

# Set the entrypoint to the publish script
ENTRYPOINT ["/bin/publish.sh"]

# docker run --rm -v "$PWD:/" -e NPM_TOKEN=?? -e TAG_NEXT=true npm