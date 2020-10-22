import os
from urllib.parse import parse_qs, urlparse

import numpy as np
import pytest
import requests
import segyio
import tempfile
from azure.core.credentials import AccessToken
from azure.core.exceptions import ResourceExistsError
from azure.core.exceptions import ResourceNotFoundError
from azure.storage.blob import BlobServiceClient
from oneseismic import scan
from oneseismic import upload
from oneseismic import client
from oneseismic import login

API_ADDR = os.getenv("API_ADDR", "http://localhost:8080")
AUTHSERVER = os.getenv("AUTHSERVER", "http://localhost:8089")
AUDIENCE = os.getenv("AUDIENCE")
STORAGE_URL = os.getenv("STORAGE_URL")
SCOPE = os.getenv("SCOPE")


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
    print("YYYYYYYY", token)
    return {"Authorization": f"Bearer {token[0]}"}


class client_auth:
    def __init__(self, auth):
        self.auth = auth

    def token(self):
        return self.auth


AUTH_HEADER = auth_header()
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

    try:
        blob_service_client.create_container("results")
    except ResourceExistsError as error:
        pass
    try:
        blob_service_client.delete_container(meta['guid'])
    except ResourceNotFoundError as error:
        pass

    shape = [64, 64, 64]
    params = {"subcube-dims": shape}
    with open(fname, "rb") as f:
        upload.upload(params, meta, f, blob_service_client)

    return meta["guid"]


@pytest.fixture(scope="session")
def cl():
    cache_dir = tempfile.mkdtemp()
    login.login(AUDIENCE, AUTHSERVER, SCOPE, cache_dir)
    return cache_dir


@pytest.fixture(scope="session")
def cube():
    """ Generate and upload simplest cube, no specific data needed

    return: guid of cube
    """
    data = np.ndarray(shape=(2, 2, 2), dtype=np.float32)

    return upload_cube(data)


@pytest.mark.skip()
def test_cube_404(cube):
    c = client.client(API_ADDR, AUTH_CLIENT)
    with pytest.raises(RuntimeError) as e:
        c.cube("not_found").slice(0, 1)
    assert "Request timed out" in str(e.value)


def test_pass(cl):
    for l in os.listdir(cl):
        ll = os.path.join(cl, l)
        print(ll)
        with open(ll) as f:
            print(f.read())
        print(ll)
    print("ZZZZZZZZZZZZZZZZZ", AUTH_HEADER)


def test_slices(cl):
    w, h, d = 100, 100, 100
    data = np.ndarray(shape=(w, h, d), dtype=np.float32)
    for i in range(w):
        for j in range(h):
            for k in range(d):
                data[i, j, k] = (i * 1) + (j * 1000) + (k * 1000000)
    
    guid = upload_cube(data)

#    c = client.client(API_ADDR, AUTH_DIR=cl)
    c = client.client(API_ADDR, AUTH_CLIENT)
    cube = c.cube(guid)

    tolerance = 1e-1

    assert np.allclose(cube.slice(0, 1), data[0, :, :], atol=tolerance)
    assert np.allclose(cube.slice(1, 1), data[:, 0, :], atol=tolerance)
    assert np.allclose(cube.slice(2, 0), data[:, :, 0], atol=tolerance)
