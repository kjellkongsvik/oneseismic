import os
import uuid
import time
from urllib.parse import parse_qs, urlparse
import json

from hypothesis import given, settings, strategies as st
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


def obo_header():
    r = requests.post(
        AUTHSERVER + "/oauth2/v2.0/token" + "?client_id=" + AUDIENCE,
        headers={"content-type": "application/json"},
        allow_redirects=False,
    )
    t = json.loads(r.content)["access_token"]

    return {"Authorization": f"Bearer {t}"}


class client_auth:
    def __init__(self, auth):
        self.auth = auth

    def token(self):
        return self.auth


AUTH_HEADER = auth_header()
OBO_HEADER = obo_header()
AUTH_CLIENT = client_auth(auth_header())


def upload_cube(data):
    """ create segy of data and upload to azure blob

    return: guid of cube
    """
    fname = tempfile.mktemp("segy")
    segyio.tools.from_array(fname, data)

    with open(fname, "rb") as f:
        meta = scan.scan(f)

    credential = CustomTokenCredential()
    blob_service_client = BlobServiceClient(STORAGE_URL, credential)

    shape = [64, 64, 64]
    params = {"subcube-dims": shape}
    meta["guid"] = str(uuid.uuid4())
    print("XXXXXXXXXXXXXXX", meta)
    print("\n")
    with open(fname, "rb") as f:
        upload.upload(params, meta, f, blob_service_client)
    return meta["guid"]


# @pytest.fixture(scope="session")
# def cube():
#     """ Generate and upload simplest cube, no specific data needed

#     return: guid of cube
#     """
#     data = np.ndarray(shape=(2, 2, 2), dtype=np.float32)

#     return upload_cube(data)


# def test_no_auth():
#     r = requests.get(API_ADDR)
#     assert r.status_code == 401


# def test_auth():
#     r = requests.get(API_ADDR, headers=AUTH_HEADER)
#     assert r.status_code == 200


# def test_list_cubes(cube):
#     c = client.client(API_ADDR, AUTH_CLIENT)
#     assert cube in c.list_cubes()


# def test_cube_404(cube):
#     c = client.client(API_ADDR, AUTH_CLIENT)
#     with pytest.raises(RuntimeError) as e:
#         c.cube("not_found").dim0
#     assert "404" in str(e.value)


# @settings(deadline=None, max_examples=1)
# @given(
#     w=st.integers(min_value=2, max_value=2),
#     h=st.integers(min_value=2, max_value=2),
#     d=st.integers(min_value=2, max_value=2),
# )
def test_slices():
    w, h, d = 2, 2, 2
    data = np.ndarray(shape=(w, h, d), dtype=np.float32)
    for i in range(w):
        for j in range(h):
            for k in range(d):
                data[i, j, k] = (i * 1) + (j * 1000) + (k * 1000000)

    credential = CustomTokenCredential()
    blob_service_client = BlobServiceClient(STORAGE_URL, credential)

    guid = upload_cube(data)
    all_containers = blob_service_client.list_containers(include_metadata=True)
    for container in all_containers:
        print(container['name'], flush=True)

    time.sleep(1)

    c = client.client(API_ADDR, AUTH_CLIENT)
    cube = c.cube(guid)
    cube.slice(0, 0)
    assert 1 == 0
    # print("LIST", client.list_cubes())
    # time.sleep(2)
    # assert "CCCCCCCCCC" == cube.slice(0, 0)
    # tolerance = 1e-1

    # for i in range(w):
    #     assert np.allclose(cube.slice(0, i), data[i, :, :], atol=tolerance)
    # for i in range(h):
    #     assert np.allclose(cube.slice(1, i), data[:, i, :], atol=tolerance)
    # for i in range(d):
    #     assert np.allclose(cube.slice(2, i), data[:, :, i], atol=tolerance)
