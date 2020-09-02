import os
import pytest
import segyio
import numpy as np
from upload import upload
from scan import scan

from urllib.parse import parse_qs, urlparse
from azure.storage.blob import BlobServiceClient
import requests

from oneseismic import client

HOST_ADDR = os.getenv("HOST_ADDR", "http://localhost:8080")
AUTH_ADDR = os.getenv("AUTH_ADDR", "http://localhost:8089")


with open("./small.sgy", "rb") as f:
    META = scan.scan(f)


def az_storage():
    protocol = "DefaultEndpointsProtocol=https;"
    account_name = "AccountName={};".format(os.getenv("AZURE_STORAGE_ACCOUNT"))
    account_key = "AccountKey={};".format(os.getenv("AZURE_STORAGE_ACCESS_KEY"))
    uri = os.getenv("AZURE_STORAGE_URL").format(os.getenv("AZURE_STORAGE_ACCOUNT"))
    blob_endpoint = "BlobEndpoint={};".format(uri)

    return protocol + account_name + account_key + blob_endpoint


def auth_header():
    r = requests.get(
        AUTH_ADDR
        + "/oauth2/v2.0/authorize"
        + "?redirect_uri=3"
        + "&client_id="
        + os.getenv("AUDIENCE")
        + "&grant_type=t"
        + "&code=c"
        + "&client_secret=s",
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


@pytest.fixture(scope="session")
def create_cubes():
    blob_service_client = BlobServiceClient.from_connection_string(az_storage())
    for c in blob_service_client.list_containers():
        blob_service_client.delete_container(c)

    shape = [64, 64, 64]
    params = {"subcube-dims": shape}
    with open("small.sgy", "rb") as f:
        upload.upload(params, META, f, blob_service_client)


def test_no_auth():
    r = requests.get(HOST_ADDR)
    assert r.status_code == 401


def test_auth():
    r = requests.get(HOST_ADDR, headers=AUTH_HEADER)
    assert r.status_code == 200


def test_list_cubes(create_cubes):
    r = requests.get(HOST_ADDR, headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == [META["guid"]]


def test_services(create_cubes):
    r = requests.get(HOST_ADDR + "/" + META["guid"], headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == ["slice"]


def test_cube_404(create_cubes):
    r = requests.get(HOST_ADDR + "/b", headers=AUTH_HEADER)
    assert r.status_code == 404


def test_dimensions(create_cubes):
    r = requests.get(HOST_ADDR + "/" + META["guid"] + "/slice", headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == [0, 1, 2]


def test_lines(create_cubes):
    r = requests.get(HOST_ADDR + "/" + META["guid"] + "/slice/1", headers=AUTH_HEADER)
    assert r.status_code == 200
    assert r.json() == META["dimensions"][1]


def test_lines_404(create_cubes):
    r = requests.get(HOST_ADDR + "/" + META["guid"] + "/slice/3", headers=AUTH_HEADER)
    assert r.status_code == 404


def tests_slices(create_cubes):
    c = client(HOST_ADDR, AUTH_CLIENT)
    cube_id = c.list_cubes()[0]
    cube = c.cube(cube_id)

    with segyio.open("small.sgy", "r") as f:
        expected = segyio.cube(f)

    for i in range(len(cube.dim0)):
        assert np.allclose(cube.slice(0, cube.dim0[i]), expected[i, :, :], atol=1e-5)
    for i in range(len(cube.dim1)):
        assert np.allclose(cube.slice(1, cube.dim1[i]), expected[:, i, :], atol=1e-5)
    for i in range(len(cube.dim2)):
        assert np.allclose(cube.slice(2, cube.dim2[i]), expected[:, :, i], atol=1e-5)
