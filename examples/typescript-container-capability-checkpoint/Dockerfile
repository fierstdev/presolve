FROM node:24-alpine AS build

WORKDIR /workspace

RUN npm install --global pnpm@12.8.1

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages/sdk/package.json packages/sdk/package.json
COPY examples/typescript-container-capability-checkpoint/package.json examples/typescript-container-capability-checkpoint/package.json

RUN pnpm install \
  --frozen-lockfile \
  --filter @presolve/sdk \
  --filter @presolve/example-typescript-container-checkpoint

COPY packages/sdk packages/sdk
COPY examples/typescript-container-capability-checkpoint examples/typescript-container-capability-checkpoint

RUN pnpm --filter @presolve/sdk build
RUN pnpm --filter @presolve/example-typescript-container-checkpoint build

FROM node:24-alpine

WORKDIR /app

COPY --from=build /workspace/packages/sdk/package.json /app/node_modules/@presolve/sdk/package.json
COPY --from=build /workspace/packages/sdk/dist /app/node_modules/@presolve/sdk/dist
COPY --from=build /workspace/examples/typescript-container-capability-checkpoint/dist /app/dist

CMD ["node", "/app/dist/main.js"]
