FROM python:3.12-slim

LABEL org.opencontainers.image.title="Porkbun DDNS" \
      org.opencontainers.image.description="Dynamic DNS updater for Porkbun domains" \
      org.opencontainers.image.version="1.0.0" \
      org.opencontainers.image.authors="skriptzip" \
      org.opencontainers.image.source="https://github.com/skriptzip/docker_porkbun-ddns" \
      org.opencontainers.image.licenses="MIT" \
      maintainer="skriptzip"

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY ddns.py .

RUN useradd -r -u 1001 appuser && \
    mkdir -p /data && \
    touch /data/last_ip.txt && \
    chown -R appuser:appuser /data

USER appuser

CMD ["python", "ddns.py"]