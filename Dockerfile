FROM ghcr.io/galoisinc/cryptol-remote-api:2.13.0

ENTRYPOINT ["/usr/local/bin/cryptol-remote-api"]

CMD ["http", "--host", "0.0.0.0", "--port", "8080", "/", "--max-occupancy", "1000"]