from src.main import app
from fastapi.testclient import TestClient


def test_platform_health() -> None:
    client = TestClient(app)
    response = client.get("/api/health")
    assert response.status_code == 200
    assert response.json()["service"] == "arp-platform"


def test_rust_health_reports_bridge_error_when_unavailable() -> None:
    client = TestClient(app)
    response = client.get("/api/rust/health")
    assert response.status_code == 502
    assert "rust-health-error" in response.json()["detail"]
