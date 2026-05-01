FROM python:3.12-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

COPY platform/pyproject.toml ./pyproject.toml
RUN uv sync --no-dev

COPY platform/src ./src

ENV ARP_PLATFORM_HOST=0.0.0.0
ENV ARP_PLATFORM_PORT=8080

CMD ["uv", "run", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8080"]
