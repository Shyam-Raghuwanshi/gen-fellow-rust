FROM rust:1.79

WORKDIR /var/task

# Install Node.js 18.x and update npm to latest
RUN apt-get update && apt-get install -y curl \
    && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

COPY package*.json ./

RUN npm install

COPY . .

RUN npm run build

CMD ["node", "test.mjs"]
