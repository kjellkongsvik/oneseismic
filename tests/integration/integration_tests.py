import os
from urllib.parse import parse_qs, urlparse

import numpy as np
import pytest
import requests
import segyio
import tempfile
from azure.core.credentials import AccessToken
from azure.storage.blob import BlobServiceClient
from oneseismic import scan
from oneseismic import upload
from oneseismic import client

API_ADDR = os.getenv("API_ADDR", "http://localhost:8080")
AUTHSERVER = os.getenv("AUTHSERVER", "http://localhost:8089")
AUDIENCE = os.getenv("AUDIENCE")
STORAGE_URL = os.getenv("AZURE_STORAGE_URL")


class CustomTokenCredential(object):
    def get_token(self, *scopes, **kwargs):
        r = requests.post(AUTHSERVER + "/oauth2/v2.0/token")
        access_token = r.json()["access_token"]
        return AccessToken(access_token, 1)


def auth_header():
    r = requests.get(
        AUTHSERVER + "/oauth2/v2.0/authorize" + "?client_id=" + AUDIENCE,
        headers={"content-type": "application/json"},
        allow_redirects=False,
    )
    token = parse_qs(urlparse(r.headers["location"]).fragment)["access_token"]

    return {"Authorization": f"Bearer {token[0]}"}


class client_auth:
    def __init__(self, auth):
        self.auth = auth

    def token(self):
        return self.auth


AUTH_HEADER = auth_header()
AUTH_CLIENT = client_auth(auth_header())


def upload_cubes(data):
    fname = tempfile.mktemp("segy")
    segyio.tools.from_array(fname, data)

    with open(fname, "rb") as f:
        meta = scan.scan(f)

    credential = CustomTokenCredential()
    blob_service_client = BlobServiceClient(STORAGE_URL, credential)
    for c in requests.get(API_ADDR, headers=AUTH_HEADER).json():
        blob_service_client.get_container_client(c).delete_container()

    shape = [64, 64, 64]
    params = {"subcube-dims": shape}
    with open(fname, "rb") as f:
        upload.upload(params, meta, f, blob_service_client)

    return meta


@pytest.fixture(scope="session")
def meta():
    w, h, d = 2, 2, 2
    data = np.ndarray(shape=(w, h, d), dtype=np.float32)
    for i in range(w):
        for j in range(h):
            for k in range(d):
                data[i, j, k] = i * j * k

    return upload_cubes(data)


def test_no_auth():
    r = requests.get(API_ADDR)
    assert r.status_code == 401


def test_auth():
    r = requests.get(API_ADDR, headers=AUTH_HEADER)
    assert r.status_code == 200


def test_list_cubes(meta):
    r = requests.get(API_ADDR, headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == [meta["guid"]]


def test_services(meta):
    r = requests.get(API_ADDR + "/" + meta["guid"], headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == ["slice"]


def test_cube_404(meta):
    r = requests.get(API_ADDR + "/b", headers=AUTH_HEADER)
    assert r.status_code == 404


def test_dimensions(meta):
    r = requests.get(API_ADDR + "/" + meta["guid"] + "/slice", headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == [0, 1, 2]


def test_lines(meta):
    r = requests.get(API_ADDR + "/" + meta["guid"] + "/slice/1", headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == meta["dimensions"][1]


def test_lines_404(meta):
    r = requests.get(API_ADDR + "/" + meta["guid"] + "/slice/3", headers=AUTH_HEADER)
    assert r.status_code == 404


def test_slices():
    w, h, d = 3, 5, 7
    data = np.ndarray(shape=(w, h, d), dtype=np.float32)
    for i in range(w):
        for j in range(h):
            for k in range(d):
                data[i, j, k] = i * j * k

    upload_cubes(data)

    c = client.client(API_ADDR, AUTH_CLIENT)
    cube_id = c.list_cubes()[0]
    cube = c.cube(cube_id)

    for i in range(len(cube.dim0)):
        assert np.allclose(cube.slice(0, cube.dim0[i]), data[i, :, :], atol=1e-5)
    for i in range(len(cube.dim1)):
        assert np.allclose(cube.slice(1, cube.dim1[i]), data[:, i, :], atol=1e-5)
    for i in range(len(cube.dim2)):
        assert np.allclose(cube.slice(2, cube.dim2[i]), data[:, :, i], atol=1e-5)
