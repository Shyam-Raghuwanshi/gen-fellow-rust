#! /bin/sh

# setup for npm credentials
echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" > ~/.npmrc

npm install --unsafe-perm --no-optional \
&& npm publish --tag next