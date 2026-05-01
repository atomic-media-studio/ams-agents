# Build docs with MkDocs Material and serve via nginx

FROM python:3.12-slim AS build
WORKDIR /docs

RUN pip install --no-cache-dir \
    mkdocs-material==9.* \
    mkdocstrings[python]==0.* \
    mkdocs-swagger-ui-tag==0.*

COPY docs/ .
COPY platform/src/ /platform/src/

RUN mkdocs build --config-file mkdocs.yml --site-dir /site

FROM nginx:alpine
COPY --from=build /site /usr/share/nginx/html
EXPOSE 80
